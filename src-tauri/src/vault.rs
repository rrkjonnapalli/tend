//! Opening a vault: check or create `notes/`, `.attachments/` and `db/app.sqlite`, in that order.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use serde::Serialize;

use crate::db::{self, Opened};
use crate::error::AppError;
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

fn plural(n: usize, word: &str) -> String {
    format!("{n} {word}{}", if n == 1 { "" } else { "s" })
}

fn count_files(dir: &Path, ext: Option<&str>) -> usize {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    entries
        .flatten()
        .map(|e| {
            let path = e.path();
            if path.is_dir() {
                count_files(&path, ext)
            } else {
                let matches = ext.is_none_or(|x| path.extension().is_some_and(|e| e == x));
                usize::from(matches)
            }
        })
        .sum()
}

/// Makes sure a directory exists. Returns a step saying whether it was found or created.
fn ensure_dir(dir: &Path, item: &'static str, found: impl Fn() -> String) -> Result<Step, Step> {
    if dir.is_dir() {
        return Ok(Step {
            item,
            status: StepStatus::Found,
            detail: format!("found · {}", found()),
        });
    }
    match fs::create_dir_all(dir) {
        Ok(()) => Ok(Step {
            item,
            status: StepStatus::Created,
            detail: "created".into(),
        }),
        Err(e) => Err(Step {
            item,
            status: StepStatus::Error,
            detail: io_detail(&e),
        }),
    }
}

/// File errors in words the start page can show.
fn io_detail(e: &std::io::Error) -> String {
    match e.kind() {
        ErrorKind::PermissionDenied => NO_PERMISSION.to_string(),
        _ => e.to_string(),
    }
}

const NO_PERMISSION: &str = "no permission to access this folder";

/// Reopens the remembered vault on launch, or says in plain words why it can't.
pub fn reopen(root: &Path) -> Result<Vault, String> {
    match fs::metadata(root) {
        Err(e) if e.kind() == ErrorKind::PermissionDenied => {
            return Err("Tend doesn’t have permission to open this folder.".into())
        }
        Ok(m) if m.is_dir() => {}
        // Never recreate a vault that has gone (moved, renamed, or on a drive that isn't attached).
        _ => return Err("The folder isn’t there any more. It may have been moved or renamed, or it's on a drive that isn’t connected.".into()),
    }
    let (check, opened) = open(root);
    opened.ok_or_else(|| {
        let failed = check.steps.iter().find(|s| s.status == StepStatus::Error);
        match failed.map(|s| s.detail.as_str()) {
            Some("incompatible schema") => {
                "An incompatible database exists in db/app.sqlite.".into()
            }
            Some(NO_PERMISSION) => "Tend doesn’t have permission to open this folder.".into(),
            Some(detail) => format!("{}: {detail}", failed.map_or("", |s| s.item)),
            None => "The vault couldn’t be opened.".into(),
        }
    })
}

pub fn open(root: &Path) -> (Check, Option<Vault>) {
    let mut steps = Vec::new();
    let notes = root.join("notes");
    let attachments = root.join(".attachments");
    let dirs = [
        ensure_dir(&notes, "notes/", || {
            plural(count_files(&notes, Some("md")), "note")
        }),
        ensure_dir(&attachments, ".attachments/", || {
            plural(count_files(&attachments, None), "file")
        }),
    ];
    for result in dirs {
        let failed = result.is_err();
        steps.push(result.unwrap_or_else(|s| s));
        if failed {
            return (finish(root, steps, false), None);
        }
    }

    let db_dir = root.join("db");
    let opened = fs::create_dir_all(&db_dir)
        .map_err(AppError::from)
        .and_then(|_| db::open(&db_dir.join("app.sqlite")));
    match opened {
        Ok((conn, how)) => {
            let (status, detail) = match how {
                Opened::Created => (StepStatus::Created, "created".to_string()),
                Opened::Current => (StepStatus::Found, "schema matches".to_string()),
                Opened::Migrated { to, .. } => (StepStatus::Found, format!("migrated to v{to}")),
            };
            steps.push(Step {
                item: "db/app.sqlite",
                status,
                detail,
            });
            (
                finish(root, steps, true),
                Some(Vault::new(root.to_path_buf(), conn)),
            )
        }
        Err(e) => {
            let detail = match e {
                AppError::IncompatibleDb => "incompatible schema".to_string(),
                AppError::Io(io) => io_detail(&io),
                other => other.to_string(),
            };
            steps.push(Step {
                item: "db/app.sqlite",
                status: StepStatus::Error,
                detail,
            });
            (finish(root, steps, false), None)
        }
    }
}

fn finish(root: &Path, steps: Vec<Step>, ok: bool) -> Check {
    Check {
        path: root.to_string_lossy().into_owned(),
        steps,
        ok,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn statuses(c: &Check) -> Vec<StepStatus> {
        c.steps.iter().map(|s| s.status).collect()
    }

    #[test]
    fn empty_folder_creates_everything() {
        let dir = tempfile::tempdir().unwrap();
        let (check, vault) = open(dir.path());
        assert!(check.ok);
        assert!(vault.is_some());
        assert_eq!(statuses(&check), vec![StepStatus::Created; 3]);
        assert!(dir.path().join("notes").is_dir());
        assert!(dir.path().join(".attachments").is_dir());
        assert!(dir.path().join("db/app.sqlite").is_file());
    }

    #[test]
    fn existing_vault_opens_and_counts() {
        let dir = tempfile::tempdir().unwrap();
        drop(open(dir.path()));
        fs::create_dir_all(dir.path().join("notes/work")).unwrap();
        fs::write(dir.path().join("notes/a.md"), "# A").unwrap();
        fs::write(dir.path().join("notes/work/b.md"), "# B").unwrap();
        fs::write(dir.path().join("notes/skip.txt"), "x").unwrap();
        let (check, vault) = open(dir.path());
        assert!(check.ok && vault.is_some());
        assert_eq!(statuses(&check), vec![StepStatus::Found; 3]);
        assert_eq!(check.steps[0].detail, "found · 2 notes");
        assert_eq!(check.steps[2].detail, "schema matches");
    }

    fn incompatible(setup: &str) {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("db")).unwrap();
        let path = dir.path().join("db/app.sqlite");
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(setup).unwrap();
        drop(conn);
        let before = fs::read(&path).unwrap();

        let (check, vault) = open(dir.path());
        assert!(!check.ok);
        assert!(vault.is_none());
        assert_eq!(check.steps[2].status, StepStatus::Error);
        assert_eq!(check.steps[2].detail, "incompatible schema");
        assert_eq!(
            fs::read(&path).unwrap(),
            before,
            "incompatible db must not change"
        );
    }

    #[test]
    fn wrong_app_id_is_incompatible() {
        incompatible(
            "CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO meta VALUES ('app_id','other'),('schema_version','1');",
        );
    }

    #[test]
    fn higher_version_is_incompatible() {
        incompatible(
            "CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO meta VALUES ('app_id','tend'),('schema_version','99');",
        );
    }

    #[test]
    fn missing_meta_is_incompatible() {
        incompatible("CREATE TABLE things (id INTEGER PRIMARY KEY);");
    }

    fn reopen_err(root: &Path) -> String {
        match reopen(root) {
            Err(e) => e,
            Ok(_) => panic!("expected {} to fail", root.display()),
        }
    }

    #[test]
    fn reopen_explains_what_went_wrong() {
        let dir = tempfile::tempdir().unwrap();
        let gone = dir.path().join("gone");
        assert!(reopen_err(&gone).contains("isn’t there"));
        assert!(!gone.exists(), "a missing vault is never recreated");

        assert!(reopen(dir.path()).is_ok());
        let bad = tempfile::tempdir().unwrap();
        fs::create_dir_all(bad.path().join("db")).unwrap();
        fs::write(bad.path().join("db/app.sqlite"), "junk").unwrap();
        assert_eq!(
            reopen_err(bad.path()),
            "An incompatible database exists in db/app.sqlite."
        );
    }

    #[test]
    fn reopen_reports_missing_permission() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let locked = dir.path().join("locked");
        fs::create_dir_all(&locked).unwrap();
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
        let err = reopen_err(&locked);
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(err.contains("permission"), "{err}");
    }

    #[test]
    fn non_sqlite_file_is_incompatible() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("db")).unwrap();
        fs::write(dir.path().join("db/app.sqlite"), "not a database").unwrap();
        let (check, vault) = open(dir.path());
        assert!(!check.ok && vault.is_none());
        assert_eq!(
            fs::read_to_string(dir.path().join("db/app.sqlite")).unwrap(),
            "not a database"
        );
    }
}
