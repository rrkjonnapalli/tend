//! Makes the index match the disk: new files are added, changed ones re-hashed, missing ones
//! marked, and items missing longer than `GRACE_MS` purged. The vault folder is the notes root:
//! hidden entries (`.tend/`, `.git/`) and files other than `.md` are ignored, and so are folders
//! with no notes in them unless Tend made them. Folders and notes whose names break the rule get
//! their slug (logged in `path_log`) when the vault allows it (`fix`), else they are reported.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use rusqlite::{params, Connection, Transaction};
use serde::Serialize;

use super::{file_path, log_path, vault_path, Moved, PathKind, Reason, EXT, GRACE_MS, MIME};
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

/// How deep the scan goes below the vault: a guard against pathological trees.
const MAX_DEPTH: usize = 32;

/// An item the scan left out of the index because its name breaks the rule and the vault doesn't
/// allow fixing yet (or, rarely, no free name was left for it).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Skipped {
    /// Relative to the vault; folders end in `/`.
    pub path: String,
    /// The path it would be renamed to; empty when no free name was left.
    pub fix: String,
}

#[derive(Default)]
struct Disk {
    folders: Vec<String>,
    notes: Vec<DiskNote>,
    skipped: Vec<Skipped>,
    /// Renames made during this scan.
    renamed: Vec<Moved>,
    /// Notes left alone for being over the size limit. Their rows stay as they are: dropping them
    /// would sync as a deletion and remove the file from the other devices.
    too_big: HashSet<String>,
}

/// What stays the same for the whole walk.
struct Scope<'a> {
    /// Names that break the rule may be fixed on disk.
    fix: bool,
    /// Folders the index already has: they stay even when they hold no notes.
    known: &'a HashSet<String>,
    /// Notes this size or larger are not indexed.
    limit: files::Limit,
}

/// One directory being walked: its place, its depth, and the names in it.
struct Walk<'a> {
    scope: &'a Scope<'a>,
    dir: &'a Path,
    rel: &'a str,
    depth: usize,
    /// How many entries in `dir` have each name, lowercased (the disk may ignore case).
    taken: HashMap<String, usize>,
}

/// An item found on disk: its name as found and what it is.
struct Found<'a> {
    raw: &'a str,
    kind: PathKind,
}

impl Found<'_> {
    /// `.md` for notes; folders have none.
    fn ext(&self) -> &'static str {
        match self.kind {
            PathKind::Folder => "",
            PathKind::Note => EXT,
        }
    }
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
    let scope = Scope {
        fix,
        known: &known,
        limit: files::Limit::of(conn)?,
    };
    let mut disk = Disk::default();
    walk(&scope, dir, "", 0, &mut disk);
    let now = now_ms();
    let tx = conn.transaction()?;
    log_renames(&tx, &disk.renamed, now)?;
    sync_folders(&tx, &disk.folders, now)?;
    sync_notes(&tx, dir, &disk, now)?;
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

fn log_renames(tx: &Transaction, renamed: &[Moved], now: i64) -> AppResult<()> {
    for moved in renamed {
        log_path(tx, moved, now)?;
    }
    Ok(())
}

/// Indexes `dir` and below. Returns true when a note was found anywhere under it. Hidden entries
/// and symlinks are skipped (a symlinked folder could loop back on itself).
fn walk(scope: &Scope, dir: &Path, rel: &str, depth: usize, out: &mut Disk) -> bool {
    if depth > MAX_DEPTH {
        return false;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };
    let entries: Vec<(String, fs::FileType)> = entries
        .flatten()
        .filter_map(|e| {
            Some((
                e.file_name().to_string_lossy().into_owned(),
                e.file_type().ok()?,
            ))
        })
        .filter(|(name, kind)| !names::is_hidden(name) && !kind.is_symlink())
        .collect();
    let mut taken = HashMap::new();
    for (name, _) in &entries {
        *taken.entry(name.to_lowercase()).or_insert(0) += 1;
    }
    let mut here = Walk {
        scope,
        dir,
        rel,
        depth,
        taken,
    };
    let mut found_notes = false;
    for (name, kind) in &entries {
        let found = if kind.is_dir() {
            add_folder(&mut here, name, out)
        } else if let Some(stem) = name.strip_suffix(EXT) {
            add_disk_note(&mut here, stem, out)
        } else {
            false
        };
        found_notes |= found;
    }
    found_notes
}

/// Walks a folder and keeps it when it holds notes or Tend already knows it. A folder whose name
/// breaks the rule is only renamed when it holds notes.
fn add_folder(here: &mut Walk, raw: &str, out: &mut Disk) -> bool {
    let known = here.scope.known.contains(&names::join(here.rel, raw));
    let valid = names::is_valid(raw);
    if !valid && !known && !has_notes(&here.dir.join(raw), here.depth + 1) {
        return false;
    }
    let found = Found {
        raw,
        kind: PathKind::Folder,
    };
    let Some(name) = settle(here, &found, out) else {
        return false;
    };
    let shown = names::join(here.rel, &name);
    let at = out.folders.len();
    let found_notes = walk(
        here.scope,
        &here.dir.join(&name),
        &shown,
        here.depth + 1,
        out,
    );
    if found_notes || here.scope.known.contains(&shown) {
        // Parents go before their children.
        out.folders.insert(at, shown);
    }
    found_notes
}

/// A `.md` file somewhere under `dir` (hidden entries and symlinks left out). Only asked for a
/// folder whose name would be fixed, so the extra walk is rare.
fn has_notes(dir: &Path, depth: usize) -> bool {
    if depth > MAX_DEPTH {
        return false;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if names::is_hidden(&name) || kind.is_symlink() {
            continue;
        }
        let found = if kind.is_dir() {
            has_notes(&entry.path(), depth + 1)
        } else {
            name.ends_with(EXT)
        };
        if found {
            return true;
        }
    }
    false
}

fn add_disk_note(here: &mut Walk, raw: &str, out: &mut Disk) -> bool {
    // A note over the size limit is never indexed.
    let size = fs::metadata(here.dir.join(format!("{raw}{EXT}"))).map_or(0, |m| m.len());
    if !here.scope.limit.allows(size) {
        out.too_big.insert(vault_path(here.rel, raw));
        return false;
    }
    let found = Found {
        raw,
        kind: PathKind::Note,
    };
    let Some(name) = settle(here, &found, out) else {
        return false;
    };
    out.notes.push(DiskNote {
        folder: here.rel.to_string(),
        mtime: files::mtime(&here.dir.join(format!("{name}{EXT}"))),
        name,
    });
    true
}

/// The name an item is indexed under: its own when valid; otherwise its fixed name, after renaming
/// it on disk when fixing is allowed. `None` when it is skipped (recorded in `out`).
fn settle(here: &mut Walk, found: &Found, out: &mut Disk) -> Option<String> {
    if names::is_valid(found.raw) {
        return Some(found.raw.to_string());
    }
    let old = names::join(here.rel, &format!("{}{}", found.raw, found.ext()));
    let Some(name) = fixed_name(here, found) else {
        record_skip(out, found.kind, &old, None);
        return None;
    };
    let file = format!("{name}{}", found.ext());
    let new = names::join(here.rel, &file);
    let renamed = here.scope.fix && rename_on_disk(here, found, &file);
    if !renamed {
        record_skip(out, found.kind, &old, Some(&new));
        return None;
    }
    *here.taken.entry(file).or_insert(0) += 1;
    out.renamed.push(Moved {
        kind: found.kind,
        reason: Reason::Fix,
        old,
        new,
    });
    Some(name)
}

/// The slug of a found name (or `note` / `folder` when nothing is left), with a suffix when taken.
/// `None` only when every suffix is taken.
fn fixed_name(here: &Walk, found: &Found) -> Option<String> {
    let ext = found.ext();
    let base = names::slug(found.raw).unwrap_or_else(|| found.kind.as_str().to_string());
    // The item's own name, in another case, is free: `Ideas.md` becomes `ideas.md`, not `ideas_1.md`.
    let own = format!("{}{ext}", found.raw).to_lowercase();
    let taken = |n: &str| {
        let name = format!("{n}{ext}");
        let others = here
            .taken
            .get(&name)
            .copied()
            .unwrap_or(0)
            .saturating_sub(usize::from(name == own));
        Ok(others > 0)
    };
    names::free(&base, taken).ok()
}

/// Renames `found` to `file` in its folder. A failure (permissions, a file that vanished) leaves
/// it to be reported as skipped.
fn rename_on_disk(here: &Walk, found: &Found, file: &str) -> bool {
    let from = here.dir.join(format!("{}{}", found.raw, found.ext()));
    let to = here.dir.join(file);
    match fs::rename(from, to) {
        Ok(()) => true,
        Err(_) => false,
    }
}

/// Lists an item the scan left out, with the path it would get (none when no free name was left).
fn record_skip(out: &mut Disk, kind: PathKind, old: &str, new: Option<&str>) {
    let slash = match kind {
        PathKind::Folder => "/",
        PathKind::Note => "",
    };
    let fix = match new {
        Some(path) => format!("{path}{slash}"),
        None => String::new(),
    };
    out.skipped.push(Skipped {
        path: format!("{old}{slash}"),
        fix,
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

fn sync_notes(tx: &Transaction, dir: &Path, disk: &Disk, now: i64) -> AppResult<()> {
    let indexed = load_indexed(tx)?;
    let mut vanished = find_vanished(&indexed, disk);
    for n in &disk.notes {
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
fn find_vanished(indexed: &HashMap<String, Indexed>, disk: &Disk) -> Vec<Vanished> {
    let present: HashSet<String> = disk
        .notes
        .iter()
        .map(|n| vault_path(&n.folder, &n.name))
        .collect();
    let gone = |path: &String| !present.contains(path) && !disk.too_big.contains(path);
    indexed
        .iter()
        .filter(|(path, ix)| !ix.missing && gone(path))
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
        "INSERT INTO notes (id, attachment_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
        params![new_id(), attachment_id, now],
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
