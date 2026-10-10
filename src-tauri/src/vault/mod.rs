//! Opening a vault. The chosen folder is the notes root; Tend's own files live in `.tend/`
//! (`.tend/attachments/`, `.tend/tend.db`), which are checked or created in that order.
//! Before writing anything, folders that look like a mistake ask first (`Confirm`).

#[cfg(test)]
mod tests;

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use rusqlite::Connection;
use serde::Serialize;

use crate::db::{self, Opened};
use crate::error::{errors, AppError};
use crate::state::Vault;
use crate::{names, notes};

#[derive(Debug, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    Found,
    Created,
    Error,
}

#[derive(Debug, Serialize)]
pub struct Step {
    pub item: &'static str,
    pub status: StepStatus,
    pub detail: String,
}

/// Something the user agrees to before Tend writes into the folder.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Confirm {
    /// Not a vault and not empty: Tend would use it as the notes root, holding `notes` notes among
    /// `items` visible entries, and rename folders and notes to fit the name rule.
    NotAVault { items: usize, notes: usize },
}

/// The step-by-step result the start page shows as a checklist. With `confirm`, nothing was done
/// yet: the folder is opened only after the user agrees.
#[derive(Debug, Serialize)]
pub struct Check {
    pub path: String,
    pub steps: Vec<Step>,
    pub ok: bool,
    pub confirm: Option<Confirm>,
}

/// Tend's own folder in the vault: attachments and the database now, sync files later.
pub const TEND_DIR: &str = ".tend";
/// Attached files, relative to the vault (also the prefix of their `attachments.path`).
pub const ATTACHMENTS_DIR: &str = ".tend/attachments";
/// The vault's database file.
pub fn db_path(root: &Path) -> std::path::PathBuf {
    root.join(TEND_DIR).join(DB_FILE)
}

/// Sync snapshots, relative to the vault (see `sync::shards`).
pub const SYNC_DIR: &str = ".tend/sync";
const DB_FILE: &str = "tend.db";
// Labels of the start page checklist.
const NOTES_ITEM: &str = "notes";
const ATTACHMENTS_ITEM: &str = ".tend/attachments/";
const DB_ITEM: &str = ".tend/tend.db";
const NO_PERMISSION: &str = "no permission to access this folder";
const INCOMPATIBLE: &str = "incompatible schema";

const NO_PERMISSION_MESSAGE: &str = "Tend doesn’t have permission to open this folder.";
const GONE_MESSAGE: &str = "The folder isn’t there any more. It may have been moved or renamed, \
                            or it's on a drive that isn’t connected.";
const NOT_A_VAULT_MESSAGE: &str = "This folder isn’t a Tend vault.";

fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        return format!("{n} {word}");
    }
    format!("{n} {word}s")
}

/// Files under `dir`, in subfolders too, leaving out hidden ones (`.tend/`, `.git/`) and symlinks
/// (as the scan does; a link loop would never end). With `ext` (`.md`), only files whose names end
/// with it.
fn count_files(dir: &Path, ext: Option<&str>) -> usize {
    let mut count = 0;
    for (name, kind) in crate::files::shown_entries(dir) {
        if kind.is_dir() {
            count += count_files(&dir.join(&name), ext);
            continue;
        }
        let matches = ext.is_none_or(|x| name.ends_with(x));
        if matches {
            count += 1;
        }
    }
    count
}

fn step(item: &'static str, status: StepStatus, detail: impl Into<String>) -> Step {
    Step {
        item,
        status,
        detail: detail.into(),
    }
}

/// Makes sure a directory exists. Returns a step saying whether it was found or created.
fn ensure_dir(dir: &Path, item: &'static str, found: impl Fn() -> String) -> Step {
    if dir.is_dir() {
        return step(item, StepStatus::Found, format!("found · {}", found()));
    }
    match fs::create_dir_all(dir) {
        Ok(()) => step(item, StepStatus::Created, "created"),
        Err(e) => step(item, StepStatus::Error, io_detail(&e)),
    }
}

/// File errors in words the start page can show.
fn io_detail(e: &std::io::Error) -> String {
    match e.kind() {
        ErrorKind::PermissionDenied => NO_PERMISSION.to_string(),
        _ => e.to_string(),
    }
}

/// Reopens the remembered vault on launch, or says in plain words why it can't.
pub fn reopen(root: &Path) -> Result<Vault, String> {
    match fs::metadata(root) {
        Err(e) if e.kind() == ErrorKind::PermissionDenied => {
            return Err(NO_PERMISSION_MESSAGE.into())
        }
        Ok(m) if m.is_dir() => {}
        // Never recreate a vault that has gone (moved, renamed, or on a drive that isn't attached).
        _ => return Err(GONE_MESSAGE.into()),
    }
    let (check, opened) = open(root, false);
    match opened {
        Some(vault) => Ok(vault),
        None => Err(failure_message(&check)),
    }
}

/// What to ask before opening `root`, if anything. Reads only; never writes.
pub fn inspect(root: &Path) -> Option<Confirm> {
    if root.join(TEND_DIR).join(DB_FILE).is_file() {
        return None;
    }
    let items = visible_entries(root);
    if items == 0 {
        return None;
    }
    Some(Confirm::NotAVault {
        items,
        notes: count_files(root, Some(notes::EXT)),
    })
}

/// Entries in `root` that aren't hidden.
pub fn visible_entries(root: &Path) -> usize {
    let Ok(entries) = fs::read_dir(root) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|e| !names::is_hidden(&e.file_name().to_string_lossy()))
        .count()
}

/// Why `open` failed: what it would have asked, or its first failed step.
fn failure_message(check: &Check) -> String {
    if check.confirm.is_some() {
        return NOT_A_VAULT_MESSAGE.into();
    }
    let Some(failed) = check.steps.iter().find(|s| s.status == StepStatus::Error) else {
        return "The vault couldn’t be opened.".into();
    };
    match failed.detail.as_str() {
        INCOMPATIBLE => errors::INCOMPATIBLE_DB.message.into(),
        NO_PERMISSION => NO_PERMISSION_MESSAGE.into(),
        detail => format!("{}: {detail}", failed.item),
    }
}

/// Opens `root`, creating what is missing. Unless `confirmed`, a folder that needs the user's say
/// (`inspect`) is left untouched and the check carries the question.
pub fn open(root: &Path, confirmed: bool) -> (Check, Option<Vault>) {
    if !confirmed {
        if let Some(confirm) = inspect(root) {
            let mut check = finish(root, Vec::new(), false);
            check.confirm = Some(confirm);
            return (check, None);
        }
    }
    let mut steps = Vec::new();
    if !check_dirs(root, &mut steps) {
        return (finish(root, steps, false), None);
    }
    let (db_step, conn) = open_db(root);
    let new_db = db_step.status == StepStatus::Created;
    steps.push(db_step);
    let Some(conn) = conn else {
        return (finish(root, steps, false), None);
    };
    // A new, empty vault has no names of its own to protect, and the user who confirmed a folder
    // agreed to its names being fixed: from now on scans fix names.
    let allow_fix = new_db && (confirmed || visible_entries(root) == 0);
    if allow_fix {
        if let Err(e) = notes::allow_fix(&conn) {
            steps.push(step(DB_ITEM, StepStatus::Error, e.to_string()));
            return (finish(root, steps, false), None);
        }
    }
    (
        finish(root, steps, true),
        Some(Vault::new(root.to_path_buf(), conn)),
    )
}

/// Counts the notes in `root`, then checks or creates `.tend/attachments/`.
fn check_dirs(root: &Path, steps: &mut Vec<Step>) -> bool {
    let found = count_files(root, Some(notes::EXT));
    steps.push(step(NOTES_ITEM, StepStatus::Found, plural(found, "note")));
    let attachments = root.join(ATTACHMENTS_DIR);
    let attachments_step = ensure_dir(&attachments, ATTACHMENTS_ITEM, || {
        plural(count_files(&attachments, None), "file")
    });
    let attachments_ok = attachments_step.status != StepStatus::Error;
    steps.push(attachments_step);
    attachments_ok
}

fn open_db(root: &Path) -> (Step, Option<Connection>) {
    let db_dir = root.join(TEND_DIR);
    let opened = fs::create_dir_all(&db_dir)
        .map_err(AppError::from)
        .and_then(|_| db::open(&db_dir.join(DB_FILE)));
    match opened {
        Ok((conn, how)) => (opened_step(how), Some(conn)),
        Err(e) => (step(DB_ITEM, StepStatus::Error, db_error_detail(e)), None),
    }
}

fn opened_step(how: Opened) -> Step {
    match how {
        Opened::Created => step(DB_ITEM, StepStatus::Created, "created"),
        Opened::Current => step(DB_ITEM, StepStatus::Found, "schema matches"),
        Opened::Migrated { to, .. } => {
            step(DB_ITEM, StepStatus::Found, format!("migrated to v{to}"))
        }
    }
}

fn db_error_detail(e: AppError) -> String {
    match e {
        e if e.is(errors::INCOMPATIBLE_DB) => INCOMPATIBLE.to_string(),
        e if e.is(errors::NO_PERMISSION) => NO_PERMISSION.to_string(),
        other => other.to_string(),
    }
}

fn finish(root: &Path, steps: Vec<Step>, ok: bool) -> Check {
    Check {
        path: root.to_string_lossy().into_owned(),
        steps,
        ok,
        confirm: None,
    }
}
