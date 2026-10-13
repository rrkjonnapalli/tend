//! The files side of applying: which note bodies and attached files must be written, moved or
//! removed so the vault matches the merged rows. Everything incoming is copied into
//! `.tend/staging/` first; nothing in the vault changes until the database commit succeeds.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::db::{columns, tables};
use crate::error::{errors, AppResult};
use crate::sync::shards::{Row, Snapshot};

/// The vault folder staged files wait in, relative to the vault.
const STAGING_DIR: &str = ".tend/staging";
/// The staged apply's `Pending`, in `STAGING_DIR`.
const PENDING_FILE: &str = "pending.json";

/// File contents in the fetched remote commit, by vault-relative path.
pub trait Blobs {
    fn read(&self, path: &str) -> AppResult<Option<Vec<u8>>>;
}

/// Where a file the merged rows need comes from.
enum Source {
    /// Already on this device, at this path (the same bytes; maybe another path).
    Local(String),
    /// In the remote commit, at this path.
    Remote(String),
}

struct Install {
    target: String,
    source: Source,
}

/// What the vault's files must become.
pub struct Plan {
    installs: Vec<Install>,
    /// Paths this device has that no merged row keeps.
    removals: Vec<String>,
    /// Note folders to make sure exist, and ones to remove if empty.
    folders: Vec<String>,
    gone_folders: Vec<String>,
    /// Each path this device's rows point at, with the hash it should hold.
    expected: BTreeMap<String, String>,
}

fn text<'a>(row: &'a Row, column: &str) -> &'a str {
    row.get(column).and_then(Value::as_str).unwrap_or_default()
}

fn paths(s: &Snapshot, table: &str) -> BTreeSet<String> {
    s.get(table)
        .map(|rows| {
            rows.values()
                .map(|r| text(r, columns::PATH).to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// A path from a row may only point inside the vault; a row from the remote is not trusted.
pub fn check_path(path: &str) -> AppResult<()> {
    if crate::files::stays_in_vault(path) {
        return Ok(());
    }
    Err(errors::SYNC_APPLY_FAILED.with(format!("Refused a path outside the vault: {path}")))
}

pub fn plan(local: &Snapshot, merged: &Snapshot, remote: &Snapshot) -> AppResult<Plan> {
    for path in paths(merged, tables::ATTACHMENTS)
        .iter()
        .chain(&paths(merged, tables::NOTE_FOLDERS))
    {
        check_path(path)?;
    }
    let empty = Default::default();
    let local_files = local.get(tables::ATTACHMENTS).unwrap_or(&empty);
    let remote_files = remote.get(tables::ATTACHMENTS).unwrap_or(&empty);
    let merged_files = merged.get(tables::ATTACHMENTS).unwrap_or(&empty);
    let mut installs = Vec::new();
    for (id, want) in merged_files {
        let have = local_files.get(id);
        let same_bytes =
            have.is_some_and(|h| text(h, columns::SHA256) == text(want, columns::SHA256));
        let in_place =
            same_bytes && have.is_some_and(|h| text(h, columns::PATH) == text(want, columns::PATH));
        if in_place {
            continue;
        }
        let source = match have {
            Some(h) if same_bytes => Source::Local(text(h, columns::PATH).to_string()),
            _ => match remote_path(id, want, remote_files) {
                Some(path) => Source::Remote(path),
                // The remote has the row but not the file (too big, or gone from disk there):
                // the row comes in, the file stays missing until it arrives.
                None => continue,
            },
        };
        installs.push(Install {
            target: text(want, columns::PATH).to_string(),
            source,
        });
    }
    let targets = paths(merged, tables::ATTACHMENTS);
    let removals = paths(local, tables::ATTACHMENTS)
        .difference(&targets)
        .cloned()
        .collect();
    let folders: Vec<String> = paths(merged, tables::NOTE_FOLDERS).into_iter().collect();
    let gone_folders = paths(local, tables::NOTE_FOLDERS)
        .difference(&paths(merged, tables::NOTE_FOLDERS))
        .cloned()
        .collect();
    let expected = local_files
        .values()
        .map(|row| {
            (
                text(row, columns::PATH).to_string(),
                text(row, columns::SHA256).to_string(),
            )
        })
        .collect();
    Ok(Plan {
        installs,
        removals,
        folders,
        gone_folders,
        expected,
    })
}

/// Where the remote commit holds the bytes a merged row wants, if it holds them at all.
fn remote_path(
    id: &str,
    want: &Row,
    remote: &std::collections::BTreeMap<String, Row>,
) -> Option<String> {
    remote
        .get(id)
        .filter(|r| text(r, columns::SHA256) == text(want, columns::SHA256))
        .map(|r| text(r, columns::PATH).to_string())
}

/// The file side of an apply that has been staged: written to `PENDING_FILE` too, so an apply cut
/// off after its commit can be finished on the next run (`resume`).
#[derive(Serialize, Deserialize, Default)]
pub struct Pending {
    /// Staged file → vault path.
    moves: Vec<(PathBuf, String)>,
    removals: Vec<String>,
    gone_folders: Vec<String>,
    /// What each indexed path held when the merge was worked out. A file that no longer holds it
    /// was edited since (an autosave, another editor) and is left alone; the next scan picks the
    /// edit up and the next sync sends it.
    expected: BTreeMap<String, String>,
}

/// Copies every incoming file into staging and makes every folder the files need, all before the
/// commit, so a clash (a file where a folder must go) rolls the whole apply back. An install the
/// remote has no copy of is left out.
pub fn stage(root: &Path, plan: &Plan, blobs: &dyn Blobs) -> AppResult<Pending> {
    let dir = root.join(STAGING_DIR);
    discard(root)?;
    fs::create_dir_all(&dir)?;
    for folder in &plan.folders {
        fs::create_dir_all(root.join(folder))?;
    }
    let mut moves = Vec::new();
    for (n, install) in plan.installs.iter().enumerate() {
        if let Some(parent) = root.join(&install.target).parent() {
            fs::create_dir_all(parent)?;
        }
        let bytes = match &install.source {
            Source::Local(from) => Some(fs::read(root.join(from))?),
            Source::Remote(from) => blobs.read(from)?,
        };
        let Some(bytes) = bytes else { continue };
        let path = dir.join(n.to_string());
        fs::write(&path, bytes)?;
        moves.push((path, install.target.clone()));
    }
    let pending = Pending {
        moves,
        removals: plan.removals.clone(),
        gone_folders: plan.gone_folders.clone(),
        expected: plan.expected.clone(),
    };
    fs::write(dir.join(PENDING_FILE), serde_json::to_vec(&pending)?)?;
    Ok(pending)
}

/// After the commit: removes what is gone and moves staged files into place (same volume, so only
/// renames are left to fail). A file edited since the merge is skipped, never overwritten.
pub fn install(root: &Path, pending: &Pending) -> AppResult<()> {
    for path in &pending.removals {
        if untouched(root, pending, path) {
            remove_if_there(&root.join(path))?;
        }
    }
    for (from, target) in &pending.moves {
        if untouched(root, pending, target) {
            fs::rename(from, root.join(target))?;
        }
    }
    remove_empty_folders(root, &pending.gone_folders);
    discard(root)
}

/// Nothing is at `path`, or it holds what the merge expected there.
fn untouched(root: &Path, pending: &Pending, path: &str) -> bool {
    let full = root.join(path);
    if !full.exists() {
        return true;
    }
    let now = crate::files::sha256(&full).ok();
    now.is_some() && now.as_ref() == pending.expected.get(path)
}

/// Finishes an apply whose commit happened but whose files didn't all move (the app quit or
/// crashed in between). A staged file moves only when the database now holds its bytes at its
/// path; a removal happens only when no row is left there. A staged apply that never committed
/// is just discarded.
pub fn resume(conn: &Connection, root: &Path) -> AppResult<()> {
    let file = root.join(STAGING_DIR).join(PENDING_FILE);
    let Ok(text) = fs::read(&file) else {
        return Ok(());
    };
    let Ok(mut pending) = serde_json::from_slice::<Pending>(&text) else {
        return discard(root);
    };
    let indexed = indexed_shas(conn)?;
    pending.moves.retain(|(from, target)| {
        let staged = crate::files::sha256(from).ok();
        staged.is_some() && staged.as_ref() == indexed.get(target)
    });
    pending.removals.retain(|path| !indexed.contains_key(path));
    install(root, &pending)
}

fn indexed_shas(conn: &Connection) -> AppResult<BTreeMap<String, String>> {
    let mut stmt = conn.prepare("SELECT path, sha256 FROM attachments")?;
    let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

fn remove_if_there(path: &Path) -> AppResult<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

/// Deepest first; a folder that still holds anything (a file Tend doesn't index) stays.
fn remove_empty_folders(root: &Path, folders: &[String]) {
    let mut deepest_first = folders.to_vec();
    deepest_first.sort_by_key(|f| std::cmp::Reverse(f.matches('/').count()));
    for folder in deepest_first {
        // Not empty, or already gone: either way there is nothing to do.
        let _ = fs::remove_dir(root.join(folder));
    }
}

/// Empties the staging folder (before staging, and after a failed or finished apply).
pub fn discard(root: &Path) -> AppResult<()> {
    let dir = root.join(STAGING_DIR);
    match fs::remove_dir_all(&dir) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}
