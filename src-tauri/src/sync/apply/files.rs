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

pub fn plan(local: &Snapshot, merged: &Snapshot, remote: &Snapshot) -> AppResult<Plan> {
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
            _ => Source::Remote(remote_path(id, want, remote_files)?),
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

/// Where the remote commit holds the bytes a merged row wants.
fn remote_path(
    id: &str,
    want: &Row,
    remote: &std::collections::BTreeMap<String, Row>,
) -> AppResult<String> {
    let found = remote
        .get(id)
        .filter(|r| text(r, "sha256") == text(want, "sha256"));
    match found {
        Some(r) => Ok(text(r, "path").to_string()),
        None => {
            Err(errors::SYNC_APPLY_FAILED
                .with(format!("No copy of {} to apply", text(want, "path"))))
        }
    }
}

/// Copies every incoming file into staging. Returns the staged path of each install, in order.
pub fn stage(root: &Path, plan: &Plan, blobs: &dyn Blobs) -> AppResult<Vec<PathBuf>> {
    let dir = root.join(STAGING_DIR);
    discard(root)?;
    fs::create_dir_all(&dir)?;
    let mut staged = Vec::new();
    for (n, install) in plan.installs.iter().enumerate() {
        let path = dir.join(n.to_string());
        match &install.source {
            Source::Local(from) => {
                fs::copy(root.join(from), &path)?;
            }
            Source::Remote(from) => {
                let Some(bytes) = blobs.read(from)? else {
                    return Err(errors::SYNC_APPLY_FAILED
                        .with(format!("{from} is missing from the remote")));
                };
                fs::write(&path, bytes)?;
            }
        }
        staged.push(path);
    }
    Ok(staged)
}

/// After the commit: removes what is gone, moves staged files into place, and makes the note
/// folders match.
pub fn install(root: &Path, plan: &Plan, staged: &[PathBuf]) -> AppResult<()> {
    for path in &plan.removals {
        remove_if_there(&root.join(path))?;
    }
    for folder in &plan.folders {
        fs::create_dir_all(root.join(folder))?;
    }
    for (install, from) in plan.installs.iter().zip(staged) {
        let to = root.join(&install.target);
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(from, to)?;
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
