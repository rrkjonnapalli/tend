//! Makes the index match the disk: new files are added, changed ones re-read, missing ones
//! marked, and items missing longer than `GRACE_MS` purged.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use rusqlite::{params, Connection, Transaction};

use super::{file_path, mtime, title_of, EXT, GRACE_MS};
use crate::error::AppResult;
use crate::names;
use crate::util::{new_id, now_ms};

struct DiskNote {
    folder: String,
    file_name: String,
    mtime: i64,
}

#[derive(Default)]
struct Disk {
    folders: Vec<String>,
    notes: Vec<DiskNote>,
    skipped: Vec<String>,
}

/// How a note is found on disk and in the index: (folder, file name).
type NoteKey = (String, String);

struct Indexed {
    id: String,
    modified_at: i64,
    missing: bool,
    title: String,
}

/// An indexed note that is no longer where the index says.
struct Vanished {
    id: String,
    modified_at: i64,
    title: String,
}

/// Returns the skipped items (names that break the rule).
pub fn scan(conn: &mut Connection, dir: &Path) -> AppResult<Vec<String>> {
    let mut disk = Disk::default();
    walk(dir, "", &mut disk);
    let now = now_ms();
    let tx = conn.transaction()?;
    sync_folders(&tx, &disk.folders, now)?;
    sync_notes(&tx, dir, &disk.notes, now)?;
    purge(&tx, now)?;
    tx.commit()?;
    disk.skipped.sort();
    Ok(disk.skipped)
}

fn walk(dir: &Path, rel: &str, out: &mut Disk) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let file = entry.file_name().to_string_lossy().into_owned();
        if file.starts_with('.') {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            add_folder(&path, rel, &file, out);
        } else {
            add_file(&path, rel, file, out);
        }
    }
}

fn add_folder(path: &Path, rel: &str, name: &str, out: &mut Disk) {
    let shown = names::join(rel, name);
    if !names::is_valid(name) {
        out.skipped.push(format!("{shown}/"));
        return;
    }
    out.folders.push(shown.clone());
    walk(path, &shown, out);
}

fn add_file(path: &Path, rel: &str, file: String, out: &mut Disk) {
    let is_note = file.strip_suffix(EXT).is_some_and(names::is_valid);
    if !is_note {
        out.skipped.push(names::join(rel, &file));
        return;
    }
    out.notes.push(DiskNote {
        folder: rel.to_string(),
        file_name: file,
        mtime: mtime(path),
    });
}

/// Folders on disk are present; indexed folders that are gone get `missing_at`.
fn sync_folders(tx: &Transaction, folders: &[String], now: i64) -> AppResult<()> {
    for f in folders {
        tx.execute(
            "INSERT INTO note_folders (path, created_at) VALUES (?1, ?2)
             ON CONFLICT(path) DO UPDATE SET missing_at = NULL",
            params![f, now],
        )?;
    }
    let on_disk: HashSet<&str> = folders.iter().map(String::as_str).collect();
    let known: Vec<String> = tx
        .prepare("SELECT path FROM note_folders WHERE path <> '' AND missing_at IS NULL")?
        .query_map([], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    for f in known.iter().filter(|f| !on_disk.contains(f.as_str())) {
        tx.execute(
            "UPDATE note_folders SET missing_at = ?1 WHERE path = ?2",
            params![now, f],
        )?;
    }
    Ok(())
}

fn sync_notes(tx: &Transaction, dir: &Path, notes: &[DiskNote], now: i64) -> AppResult<()> {
    let indexed = load_indexed(tx)?;
    let mut vanished = find_vanished(&indexed, notes);
    for n in notes {
        let key = (n.folder.clone(), n.file_name.clone());
        match indexed.get(&key) {
            Some(ix) if ix.modified_at == n.mtime && !ix.missing => {}
            Some(ix) => refresh_note(tx, dir, n, &ix.id)?,
            None => add_note(tx, dir, n, &mut vanished, now)?,
        }
    }
    for v in &vanished {
        tx.execute(
            "UPDATE notes SET missing_at = ?1 WHERE id = ?2",
            params![now, v.id],
        )?;
    }
    Ok(())
}

fn load_indexed(tx: &Transaction) -> AppResult<HashMap<NoteKey, Indexed>> {
    let mut stmt =
        tx.prepare("SELECT folder, file_name, id, modified_at, missing_at, title FROM notes")?;
    let rows = stmt.query_map([], |r| {
        let key = (r.get(0)?, r.get(1)?);
        let indexed = Indexed {
            id: r.get(2)?,
            modified_at: r.get(3)?,
            missing: r.get::<_, Option<i64>>(4)?.is_some(),
            title: r.get(5)?,
        };
        Ok((key, indexed))
    })?;
    let indexed = rows.collect::<Result<_, _>>()?;
    Ok(indexed)
}

/// Present notes the disk no longer has at their indexed place.
fn find_vanished(indexed: &HashMap<NoteKey, Indexed>, notes: &[DiskNote]) -> Vec<Vanished> {
    let present: HashSet<NoteKey> = notes
        .iter()
        .map(|n| (n.folder.clone(), n.file_name.clone()))
        .collect();
    indexed
        .iter()
        .filter(|(key, ix)| !ix.missing && !present.contains(*key))
        .map(|(_, ix)| Vanished {
            id: ix.id.clone(),
            modified_at: ix.modified_at,
            title: ix.title.clone(),
        })
        .collect()
}

/// The file changed (or came back): re-read its title.
fn refresh_note(tx: &Transaction, dir: &Path, n: &DiskNote, id: &str) -> AppResult<()> {
    let title = read_title(dir, n);
    tx.execute(
        "UPDATE notes SET title = ?1, modified_at = ?2, missing_at = NULL WHERE id = ?3",
        params![title, n.mtime, id],
    )?;
    Ok(())
}

/// A file the index doesn't know. If a vanished note has the same mtime and title, the file is
/// that note renamed or moved outside Tend, so it keeps its id (and its tags and links).
fn add_note(
    tx: &Transaction,
    dir: &Path,
    n: &DiskNote,
    vanished: &mut Vec<Vanished>,
    now: i64,
) -> AppResult<()> {
    let title = read_title(dir, n);
    let moved = vanished
        .iter()
        .position(|v| v.modified_at == n.mtime && v.title == title);
    if let Some(i) = moved {
        let note = vanished.swap_remove(i);
        tx.execute(
            "UPDATE notes SET folder = ?1, file_name = ?2, missing_at = NULL WHERE id = ?3",
            params![n.folder, n.file_name, note.id],
        )?;
        return Ok(());
    }
    tx.execute(
        "INSERT INTO notes (id, folder, file_name, title, created_at, modified_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![new_id(), n.folder, n.file_name, title, now, n.mtime],
    )?;
    Ok(())
}

fn read_title(dir: &Path, n: &DiskNote) -> String {
    let path = file_path(dir, &n.folder, &n.file_name);
    let text = fs::read_to_string(path).unwrap_or_default();
    title_of(&text, n.file_name.trim_end_matches(EXT))
}

fn purge(tx: &Transaction, now: i64) -> AppResult<()> {
    let cutoff = now - GRACE_MS;
    tx.execute("DELETE FROM notes WHERE missing_at < ?1", [cutoff])?;
    tx.execute(
        "DELETE FROM note_folders WHERE missing_at < ?1
         AND NOT EXISTS (SELECT 1 FROM notes WHERE notes.folder = note_folders.path)",
        [cutoff],
    )?;
    Ok(())
}
