//! Task folders, keyed by path under one root (''). Renaming or moving a folder rewrites the
//! paths below it; tasks follow through ON UPDATE CASCADE.

use rusqlite::{params, Connection, OptionalExtension, Transaction};

use crate::error::{errors, AppResult};
use crate::names;
use crate::util::now_ms;

fn exists(conn: &Connection, path: &str) -> AppResult<bool> {
    let found = conn
        .query_row("SELECT 1 FROM task_folders WHERE path = ?1", [path], |_| {
            Ok(())
        })
        .optional()?;
    Ok(found.is_some())
}

fn check_new(conn: &Connection, parent: &str, name: &str) -> AppResult<()> {
    names::check(name)?;
    if exists(conn, &names::join(parent, name))? {
        return Err(errors::NAME_TAKEN.err());
    }
    Ok(())
}

/// Creates `path` and any missing parents.
pub fn create(conn: &mut Connection, path: &str) -> AppResult<()> {
    names::check_path(path)?;
    if path.is_empty() || exists(conn, path)? {
        return Err(errors::NAME_TAKEN.err());
    }
    let tx = conn.transaction()?;
    ensure(&tx, path)?;
    tx.commit()?;
    Ok(())
}

/// Adds `path` and its parents when they're missing; existing ones are left alone.
pub(super) fn ensure(tx: &Transaction, path: &str) -> AppResult<()> {
    names::check_path(path)?;
    let now = now_ms();
    let mut current = String::new();
    for segment in path.split('/').filter(|s| !s.is_empty()) {
        current = names::join(&current, segment);
        tx.execute(
            "INSERT OR IGNORE INTO task_folders (path, created_at) VALUES (?1, ?2)",
            params![current, now],
        )?;
    }
    Ok(())
}

pub fn rename(conn: &mut Connection, path: &str, name: &str) -> AppResult<String> {
    if names::base(path) == name {
        return Ok(path.to_string());
    }
    let parent = names::parent(path);
    check_new(conn, parent, name)?;
    let new = names::join(parent, name);
    let tx = conn.transaction()?;
    relocate(&tx, path, &new)?;
    tx.commit()?;
    Ok(new)
}

pub fn move_to(conn: &mut Connection, path: &str, target: &str) -> AppResult<String> {
    if names::is_under(target, path) {
        return Err(errors::INTO_SELF.err());
    }
    if names::parent(path) == target {
        return Err(errors::SAME_FOLDER.err());
    }
    let name = names::base(path);
    check_new(conn, target, name)?;
    let new = names::join(target, name);
    let tx = conn.transaction()?;
    ensure(&tx, target)?;
    relocate(&tx, path, &new)?;
    tx.commit()?;
    Ok(new)
}

fn relocate(tx: &Transaction, old: &str, new: &str) -> AppResult<()> {
    if old.is_empty() || !exists(tx, old)? {
        return Err(errors::NOT_FOUND.with("Folder was not found"));
    }
    tx.execute(
        "UPDATE task_folders SET path = ?2 || substr(path, length(?1) + 1)
         WHERE path = ?1 OR substr(path, 1, length(?1) + 1) = ?1 || '/'",
        params![old, new],
    )?;
    Ok(())
}

/// Deletes the folder, its subfolders and every task in them.
pub fn delete(conn: &mut Connection, path: &str) -> AppResult<()> {
    if path.is_empty() {
        return Err(errors::ROOT_FOLDER.err());
    }
    let tx = conn.transaction()?;
    tx.execute(
        "DELETE FROM tasks WHERE path = ?1 OR substr(path, 1, length(?1) + 1) = ?1 || '/'",
        [path],
    )?;
    tx.execute(
        "DELETE FROM task_folders WHERE path = ?1 OR substr(path, 1, length(?1) + 1) = ?1 || '/'",
        [path],
    )?;
    tx.commit()?;
    Ok(())
}
