//! The files side of applying: which note bodies and attached files must be written, moved or
//! removed so the vault matches the merged rows. Everything incoming is copied into
//! `.tend/staging/` first; nothing in the vault changes until the database commit succeeds.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::error::{errors, AppResult};
use crate::sync::shards::{Row, Snapshot};

/// The vault folder staged files wait in, relative to the vault.
const STAGING_DIR: &str = ".tend/staging";

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
}

fn text<'a>(row: &'a Row, column: &str) -> &'a str {
    row.get(column).and_then(Value::as_str).unwrap_or_default()
}

fn paths(s: &Snapshot, table: &str) -> BTreeSet<String> {
    s.get(table)
        .map(|rows| rows.values().map(|r| text(r, "path").to_string()).collect())
        .unwrap_or_default()
}

/// A path from a row may only point inside the vault: relative, no `..`, and under `.tend/` only
/// in the attachment store. A row from the remote is not trusted to say otherwise.
pub fn check_path(path: &str) -> AppResult<()> {
    use std::path::Component;
    let p = Path::new(path);
    // The empty path is the root folder itself.
    let plain = p.components().all(|c| matches!(c, Component::Normal(_)));
    let in_tend = p.starts_with(crate::vault::TEND_DIR);
    let in_store = p.starts_with(crate::vault::ATTACHMENTS_DIR);
    if plain && (!in_tend || in_store) {
        return Ok(());
    }
    Err(errors::SYNC_APPLY_FAILED.with(format!("Refused a path outside the vault: {path}")))
}

pub fn plan(local: &Snapshot, merged: &Snapshot, remote: &Snapshot) -> AppResult<Plan> {
    for path in paths(merged, "attachments")
        .iter()
        .chain(&paths(merged, "note_folders"))
    {
        check_path(path)?;
    }
    let empty = Default::default();
    let local_files = local.get("attachments").unwrap_or(&empty);
    let remote_files = remote.get("attachments").unwrap_or(&empty);
    let merged_files = merged.get("attachments").unwrap_or(&empty);
    let mut installs = Vec::new();
    for (id, want) in merged_files {
        let have = local_files.get(id);
        let same_bytes = have.is_some_and(|h| text(h, "sha256") == text(want, "sha256"));
        let in_place = same_bytes && have.is_some_and(|h| text(h, "path") == text(want, "path"));
        if in_place {
            continue;
        }
        let source = match have {
            Some(h) if same_bytes => Source::Local(text(h, "path").to_string()),
            _ => match remote_path(id, want, remote_files) {
                Some(path) => Source::Remote(path),
                // The remote has the row but not the file (too big, or gone from disk there):
                // the row comes in, the file stays missing until it arrives.
                None => continue,
            },
        };
        installs.push(Install {
            target: text(want, "path").to_string(),
            source,
        });
    }
    let targets = paths(merged, "attachments");
    let removals = paths(local, "attachments")
        .difference(&targets)
        .cloned()
        .collect();
    let folders: Vec<String> = paths(merged, "note_folders").into_iter().collect();
    let gone_folders = paths(local, "note_folders")
        .difference(&paths(merged, "note_folders"))
        .cloned()
        .collect();
    Ok(Plan {
        installs,
        removals,
        folders,
        gone_folders,
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
        .filter(|r| text(r, "sha256") == text(want, "sha256"))
        .map(|r| text(r, "path").to_string())
}

/// Copies every incoming file into staging and makes every folder the files need, all before the
/// commit, so a clash (a file where a folder must go) rolls the whole apply back. Returns the staged
/// path of each install, in order; `None` where the remote has no copy of the file.
pub fn stage(root: &Path, plan: &Plan, blobs: &dyn Blobs) -> AppResult<Vec<Option<PathBuf>>> {
    let dir = root.join(STAGING_DIR);
    discard(root)?;
    fs::create_dir_all(&dir)?;
    for folder in &plan.folders {
        fs::create_dir_all(root.join(folder))?;
    }
    let mut staged = Vec::new();
    for (n, install) in plan.installs.iter().enumerate() {
        if let Some(parent) = root.join(&install.target).parent() {
            fs::create_dir_all(parent)?;
        }
        let path = dir.join(n.to_string());
        let bytes = match &install.source {
            Source::Local(from) => Some(fs::read(root.join(from))?),
            Source::Remote(from) => blobs.read(from)?,
        };
        match bytes {
            Some(bytes) => {
                fs::write(&path, bytes)?;
                staged.push(Some(path));
            }
            None => staged.push(None),
        }
    }
    Ok(staged)
}

/// After the commit: removes what is gone and moves staged files into place (same volume, so only
/// renames are left to fail).
pub fn install(root: &Path, plan: &Plan, staged: &[Option<PathBuf>]) -> AppResult<()> {
    for path in &plan.removals {
        remove_if_there(&root.join(path))?;
    }
    for (install, from) in plan.installs.iter().zip(staged) {
        let Some(from) = from else { continue };
        fs::rename(from, root.join(&install.target))?;
    }
    remove_empty_folders(root, &plan.gone_folders);
    discard(root)
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
