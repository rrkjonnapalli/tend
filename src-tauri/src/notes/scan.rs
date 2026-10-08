//! Makes the index match the disk: new files are added, changed ones re-hashed, missing ones
//! marked, and items missing longer than `GRACE_MS` purged. The vault folder is the notes root:
//! hidden entries (`.tend/`, `.git/`) and files other than `.md` are ignored, and so are folders
//! with no notes in them unless Tend made them. Folders and notes whose names break the rule get
//! their slug (logged in `path_log`) when the vault allows it (`fix`), else they are reported.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, Transaction};
use serde::Serialize;

use super::{file_path, log_path, vault_path, PathKind, Reason, EXT, GRACE_MS, MIME};
use crate::error::AppResult;
use crate::files;
use crate::names;
use crate::util::{new_id, now_ms};

struct DiskNote {
    folder: String,
    /// File name without `.md`.
    name: String,
    /// The title a new note gets: the file name as found, before any fix.
    title: String,
    mtime: i64,
}

/// An item the scan left out of the index because its name breaks the rule and the vault doesn't
/// allow fixing yet.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Skipped {
    /// Relative to the vault; folders end in `/`.
    pub path: String,
    /// The path it would be renamed to.
    pub fix: String,
}

#[derive(Default)]
struct Disk {
    folders: Vec<String>,
    notes: Vec<DiskNote>,
    skipped: Vec<Skipped>,
    /// (kind, old, new) paths renamed during this scan.
    renamed: Vec<(PathKind, String, String)>,
}

/// One directory being walked: its place, whether names may be fixed, and the names in it.
struct Walk<'a> {
    dir: &'a Path,
    rel: &'a str,
    fix: bool,
    /// Folders the index already has: they stay even when they hold no notes.
    known: &'a HashSet<String>,
    /// Every entry name in `dir`, lowercased (the disk may ignore case).
    taken: HashSet<String>,
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

/// Returns the skipped items. With `fix`, names that can be made valid are renamed on disk first.
pub fn scan(conn: &mut Connection, dir: &Path, fix: bool) -> AppResult<Vec<Skipped>> {
    let known = known_folders(conn)?;
    let mut disk = Disk::default();
    walk(dir, "", fix, &known, &mut disk);
    let now = now_ms();
    let tx = conn.transaction()?;
    log_renames(&tx, &disk.renamed, now)?;
    sync_folders(&tx, &disk.folders, now)?;
    sync_notes(&tx, dir, &disk.notes, now)?;
    purge(&tx, now)?;
    tx.commit()?;
    disk.skipped.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(disk.skipped)
}

fn known_folders(conn: &Connection) -> AppResult<HashSet<String>> {
    let known = conn
        .prepare("SELECT path FROM note_folders WHERE missing_at IS NULL")?
        .query_map([], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    Ok(known)
}

fn log_renames(
    tx: &Transaction,
    renamed: &[(PathKind, String, String)],
    now: i64,
) -> AppResult<()> {
    for (kind, old, new) in renamed {
        log_path(tx, *kind, Reason::Fix, old, new, now)?;
    }
    Ok(())
}

fn walk(dir: &Path, rel: &str, fix: bool, known: &HashSet<String>, out: &mut Disk) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let entries: Vec<(String, PathBuf)> = entries
        .flatten()
        .map(|e| (e.file_name().to_string_lossy().into_owned(), e.path()))
        .filter(|(file, _)| !file.starts_with('.'))
        .collect();
    let taken = entries.iter().map(|(f, _)| f.to_lowercase()).collect();
    let mut here = Walk {
        dir,
        rel,
        fix,
        known,
        taken,
    };
    for (file, path) in entries {
        if path.is_dir() {
            add_folder(&mut here, &path, &file, out);
        } else if let Some(stem) = file.strip_suffix(EXT) {
            add_disk_note(&mut here, stem, out);
        }
    }
}

fn add_folder(here: &mut Walk, path: &Path, raw: &str, out: &mut Disk) {
    let known = here.known.contains(&names::join(here.rel, raw));
    if !known && !has_notes(path) {
        return;
    }
    let Some(name) = settle(here, raw, "", PathKind::Folder, out) else {
        return;
    };
    let shown = names::join(here.rel, &name);
    out.folders.push(shown.clone());
    walk(&here.dir.join(&name), &shown, here.fix, here.known, out);
}

/// A `.md` file somewhere under `dir`, leaving out hidden entries.
fn has_notes(dir: &Path) -> bool {
    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };
    entries.flatten().any(|e| {
        let file = e.file_name().to_string_lossy().into_owned();
        if file.starts_with('.') {
            return false;
        }
        let path = e.path();
        if path.is_dir() {
            has_notes(&path)
        } else {
            file.ends_with(EXT)
        }
    })
}

fn add_disk_note(here: &mut Walk, raw: &str, out: &mut Disk) {
    let Some(name) = settle(here, raw, EXT, PathKind::Note, out) else {
        return;
    };
    out.notes.push(DiskNote {
        folder: here.rel.to_string(),
        mtime: files::mtime(&here.dir.join(format!("{name}{EXT}"))),
        title: raw.to_string(),
        name,
    });
}

/// The name an item is indexed under: its own when valid; otherwise its slug (with a suffix when
/// taken), after renaming it on disk when fixing is allowed. `None` when it is skipped, which is
/// recorded in `out`.
fn settle(here: &mut Walk, raw: &str, ext: &str, kind: PathKind, out: &mut Disk) -> Option<String> {
    if names::is_valid(raw) {
        return Some(raw.to_string());
    }
    let fallback = match kind {
        PathKind::Folder => "folder",
        PathKind::Note => "note",
    };
    let base = names::slug(raw).unwrap_or_else(|| fallback.to_string());
    let taken = |n: &str| Ok(here.taken.contains(&format!("{n}{ext}")));
    let name = names::free(&base, taken).ok()?;
    let old = names::join(here.rel, &format!("{raw}{ext}"));
    let new = names::join(here.rel, &format!("{name}{ext}"));
    let renamed = here.fix
        && fs::rename(
            here.dir.join(format!("{raw}{ext}")),
            here.dir.join(format!("{name}{ext}")),
        )
        .is_ok();
    if renamed {
        here.taken.insert(format!("{name}{ext}"));
        out.renamed.push((kind, old, new));
        return Some(name);
    }
    let slash = if ext.is_empty() { "/" } else { "" };
    out.skipped.push(Skipped {
        path: format!("{old}{slash}"),
        fix: format!("{new}{slash}"),
    });
    None
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
/// renamed or moved outside Tend, so the note keeps its id (and its tags and links).
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
        params![new_id(), attachment_id, n.title, now],
    )?;
    Ok(())
}

fn purge(tx: &Transaction, now: i64) -> AppResult<()> {
    let cutoff = now - GRACE_MS;
    // Note bodies only: attached files under .tend/ are never scanned.
    tx.execute(
        "DELETE FROM attachments WHERE missing_at < ?1
         AND id IN (SELECT attachment_id FROM notes)",
        [cutoff],
    )?;
    tx.execute(
        "DELETE FROM note_folders WHERE missing_at < ?1
         AND NOT EXISTS (SELECT 1 FROM attachments a
                         WHERE substr(a.path, 1, length(note_folders.path) + 1)
                               = note_folders.path || '/')",
        [cutoff],
    )?;
    Ok(())
}
