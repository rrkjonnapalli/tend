//! Opening a vault: check or create `notes/`, `.attachments/` and `db/app.sqlite`, in that order.

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

/// The step-by-step result the start page shows as a checklist.
#[derive(Debug, Serialize)]
pub struct Check {
    pub path: String,
    pub steps: Vec<Step>,
    pub ok: bool,
}

const DB_ITEM: &str = "db/app.sqlite";
const NO_PERMISSION: &str = "no permission to access this folder";
const INCOMPATIBLE: &str = "incompatible schema";

const NO_PERMISSION_MESSAGE: &str = "Tend doesn’t have permission to open this folder.";
const GONE_MESSAGE: &str = "The folder isn’t there any more. It may have been moved or renamed, \
                            or it's on a drive that isn’t connected.";
const INCOMPATIBLE_MESSAGE: &str = "An incompatible database exists in db/app.sqlite.";

fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        return format!("{n} {word}");
    }
    format!("{n} {word}s")
}

/// Files under `dir`, in subfolders too. With `ext`, only files with that extension.
fn count_files(dir: &Path, ext: Option<&str>) -> usize {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    let mut count = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            count += count_files(&path, ext);
            continue;
        }
        let matches = ext.is_none_or(|x| path.extension().is_some_and(|e| e == x));
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
    let (check, opened) = open(root);
    match opened {
        Some(vault) => Ok(vault),
        None => Err(failure_message(&check)),
    }
}

/// Why `open` failed, from its first failed step.
fn failure_message(check: &Check) -> String {
    let Some(failed) = check.steps.iter().find(|s| s.status == StepStatus::Error) else {
        return "The vault couldn’t be opened.".into();
    };
    match failed.detail.as_str() {
        INCOMPATIBLE => INCOMPATIBLE_MESSAGE.into(),
        NO_PERMISSION => NO_PERMISSION_MESSAGE.into(),
        detail => format!("{}: {detail}", failed.item),
    }
}

pub fn open(root: &Path) -> (Check, Option<Vault>) {
    let mut steps = Vec::new();
    if !check_dirs(root, &mut steps) {
        return (finish(root, steps, false), None);
    }
    let (db_step, conn) = open_db(root);
    let fresh = db_step.status == StepStatus::Created && is_empty(&root.join("notes"));
    steps.push(db_step);
    if let (true, Some(conn)) = (fresh, &conn) {
        // A new, empty vault has no names of its own to protect: files added later get fixed.
        let allowed = crate::settings::set(
            conn,
            crate::notes::FIX_NAMES,
            &serde_json::Value::Bool(true),
        );
        if let Err(e) = allowed {
            steps.push(step(DB_ITEM, StepStatus::Error, e.to_string()));
            return (finish(root, steps, false), None);
        }
    }
    match conn {
        Some(conn) => (
            finish(root, steps, true),
            Some(Vault::new(root.to_path_buf(), conn)),
        ),
        None => (finish(root, steps, false), None),
    }
}

/// Checks or creates `notes/` then `.attachments/`, stopping at the first that fails.
fn check_dirs(root: &Path, steps: &mut Vec<Step>) -> bool {
    let notes = root.join("notes");
    let notes_step = ensure_dir(&notes, "notes/", || {
        plural(count_files(&notes, Some("md")), "note")
    });
    let notes_ok = notes_step.status != StepStatus::Error;
    steps.push(notes_step);
    if !notes_ok {
        return false;
    }
    let attachments = root.join(".attachments");
    let attachments_step = ensure_dir(&attachments, ".attachments/", || {
        plural(count_files(&attachments, None), "file")
    });
    let attachments_ok = attachments_step.status != StepStatus::Error;
    steps.push(attachments_step);
    attachments_ok
}

/// No entries other than hidden ones (`.DS_Store`).
fn is_empty(dir: &Path) -> bool {
    fs::read_dir(dir).map_or(true, |entries| {
        entries
            .flatten()
            .all(|e| e.file_name().to_string_lossy().starts_with('.'))
    })
}

fn open_db(root: &Path) -> (Step, Option<Connection>) {
    let db_dir = root.join("db");
    let opened = fs::create_dir_all(&db_dir)
        .map_err(AppError::from)
        .and_then(|_| db::open(&db_dir.join("app.sqlite")));
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
    }
}
