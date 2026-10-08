//! Makes the index match the disk: new files are added, changed ones re-hashed, missing ones
//! marked, and items missing longer than `GRACE_MS` purged.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use rusqlite::{params, Connection, Transaction};

use super::{file_path, vault_path, EXT, GRACE_MS, MIME, PREFIX};
use crate::error::AppResult;
use crate::files;
use crate::names;
use crate::util::{new_id, now_ms};

struct DiskNote {
    folder: String,
    /// File name without `.md`.
    name: String,
    mtime: i64,
}

#[derive(Default)]
struct Disk {
    folders: Vec<String>,
    notes: Vec<DiskNote>,
    skipped: Vec<String>,
}

/// A note body the index knows, keyed by its attachment path.
struct Indexed {
    attachment_id: String,
    sha256: String,
    mtime: Option<i64>,
    missing: bool,
}

/// An indexed body that is no longer at its path.
struct Vanished {
    attachment_id: String,
    sha256: String,
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
            add_file(&path, rel, &file, out);
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

fn add_file(path: &Path, rel: &str, file: &str, out: &mut Disk) {
    let name = file.strip_suffix(EXT).filter(|n| names::is_valid(n));
    let Some(name) = name else {
        out.skipped.push(names::join(rel, file));
        return;
    };
    out.notes.push(DiskNote {
        folder: rel.to_string(),
        name: name.to_string(),
        mtime: files::mtime(path),
    });
}

/// Folders on disk are present; indexed folders that are gone get `missing_at`.
fn sync_folders(tx: &Transaction, folders: &[String], now: i64) -> AppResult<()> {
    for f in folders {
        tx.execute(
            "INSERT INTO note_folders (id, path, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)
             ON CONFLICT(path) DO UPDATE SET missing_at = NULL WHERE missing_at IS NOT NULL",
            params![new_id(), f, now],
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
        let path = vault_path(&n.folder, &n.name);
        match indexed.get(&path) {
            Some(ix) if ix.mtime == Some(n.mtime) && !ix.missing => {}
            Some(ix) => refresh_note(tx, dir, n, ix, now)?,
            None => add_note(tx, dir, n, &mut vanished, now)?,
        }
    }
    for v in &vanished {
        tx.execute(
            "UPDATE attachments SET missing_at = ?1 WHERE id = ?2",
            params![now, v.attachment_id],
        )?;
    }
    Ok(())
}

fn load_indexed(tx: &Transaction) -> AppResult<HashMap<String, Indexed>> {
    let mut stmt = tx.prepare(
        "SELECT a.path, a.id, a.sha256, a.mtime, a.missing_at
         FROM attachments a JOIN notes n ON n.attachment_id = a.id",
    )?;
    let rows = stmt.query_map([], |r| {
        let indexed = Indexed {
            attachment_id: r.get(1)?,
            sha256: r.get(2)?,
            mtime: r.get(3)?,
            missing: r.get::<_, Option<i64>>(4)?.is_some(),
        };
        Ok((r.get(0)?, indexed))
    })?;
    let indexed = rows.collect::<Result<_, _>>()?;
    Ok(indexed)
}

/// Present bodies the disk no longer has at their indexed path.
fn find_vanished(indexed: &HashMap<String, Indexed>, notes: &[DiskNote]) -> Vec<Vanished> {
    let present: HashSet<String> = notes
        .iter()
        .map(|n| vault_path(&n.folder, &n.name))
        .collect();
    indexed
        .iter()
        .filter(|(path, ix)| !ix.missing && !present.contains(*path))
        .map(|(_, ix)| Vanished {
            attachment_id: ix.attachment_id.clone(),
            sha256: ix.sha256.clone(),
        })
        .collect()
}

/// The file's mtime moved (or it came back): re-hash it, and bump `updated_at` only when the
/// content really changed.
fn refresh_note(
    tx: &Transaction,
    dir: &Path,
    n: &DiskNote,
    ix: &Indexed,
    now: i64,
) -> AppResult<()> {
    let facts = files::facts(&file_path(dir, &n.folder, &n.name))?;
    let changed = facts.sha256 != ix.sha256;
    tx.execute(
        "UPDATE attachments SET size_bytes = ?1, sha256 = ?2, mtime = ?3, missing_at = NULL,
           updated_at = CASE WHEN ?4 THEN ?5 ELSE updated_at END
         WHERE id = ?6",
        params![
            facts.size,
            facts.sha256,
            facts.mtime,
            changed,
            now,
            ix.attachment_id
        ],
    )?;
    Ok(())
}

/// A file the index doesn't know. If a vanished body has the same hash, the file is that note
/// renamed or moved outside Tend, so the note keeps its id (and its title, tags and links).
fn add_note(
    tx: &Transaction,
    dir: &Path,
    n: &DiskNote,
    vanished: &mut Vec<Vanished>,
    now: i64,
) -> AppResult<()> {
    let facts = files::facts(&file_path(dir, &n.folder, &n.name))?;
    let path = vault_path(&n.folder, &n.name);
    let file = format!("{}{EXT}", n.name);
    let moved = vanished.iter().position(|v| v.sha256 == facts.sha256);
    if let Some(i) = moved {
        let body = vanished.swap_remove(i);
        tx.execute(
            "UPDATE attachments SET path = ?1, name = ?2, mtime = ?3, missing_at = NULL,
               updated_at = ?4
             WHERE id = ?5",
            params![path, file, facts.mtime, now, body.attachment_id],
        )?;
        return Ok(());
    }
    let attachment_id = new_id();
    tx.execute(
        "INSERT INTO attachments
           (id, path, name, mime_type, size_bytes, sha256, created_at, updated_at, mtime)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, ?8)",
        params![
            attachment_id,
            path,
            file,
            MIME,
            facts.size,
            facts.sha256,
            now,
            facts.mtime
        ],
    )?;
    tx.execute(
        "INSERT INTO notes (id, attachment_id, title, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?4)",
        params![new_id(), attachment_id, n.name, now],
    )?;
    Ok(())
}

fn purge(tx: &Transaction, now: i64) -> AppResult<()> {
    let cutoff = now - GRACE_MS;
    tx.execute(
        "DELETE FROM attachments WHERE missing_at < ?1 AND substr(path, 1, length(?2)) = ?2",
        params![cutoff, PREFIX],
    )?;
    tx.execute(
        "DELETE FROM note_folders WHERE missing_at < ?1
         AND NOT EXISTS (SELECT 1 FROM attachments a
                         WHERE substr(a.path, 1, length(?2 || note_folders.path || '/'))
                               = ?2 || note_folders.path || '/')",
        params![cutoff, PREFIX],
    )?;
    Ok(())
}
