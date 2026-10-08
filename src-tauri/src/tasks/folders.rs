//! Task folders, keyed by path under one root (''). Renaming or moving a folder rewrites the
//! paths below it; tasks follow through ON UPDATE CASCADE.

use rusqlite::{params, Connection, OptionalExtension, Transaction};

use crate::error::{errors, AppResult};
use crate::names;
use crate::util::{new_id, now_ms};

fn exists(conn: &Connection, path: &str) -> AppResult<bool> {
    let found = conn
        .query_row("SELECT 1 FROM task_folders WHERE path = ?1", [path], |_| {
            Ok(())
        })
        .optional()?;
    Ok(found.is_some())
}

/// The cleaned folder path typed by the user, spelled as existing folders are spelled.
pub(super) fn target_path(conn: &Connection, path: &str) -> AppResult<String> {
    let path = names::accept_path(path)?;
    names::canonical(&path, |p| {
        let found = conn
            .query_row("SELECT path FROM task_folders WHERE path = ?1", [p], |r| {
                r.get(0)
            })
            .optional()?;
        Ok(found)
    })
}

/// Another folder in `parent` already uses `name` (ignoring case). `current` is the folder's own
/// name when renaming it, which counts as free.
fn taken(conn: &Connection, parent: &str, name: &str, current: Option<&str>) -> AppResult<bool> {
    if current.is_some_and(|c| names::same(c, name)) {
        return Ok(false);
    }
    exists(conn, &names::join(parent, name))
}

/// Creates the folder `path` typed by the user, and any missing parents. When the last segment is
/// taken it gets a suffix. Returns the path that was created.
pub fn create(conn: &mut Connection, path: &str) -> AppResult<String> {
    let path = target_path(conn, path)?;
    if path.is_empty() {
        return Err(errors::NAME_EMPTY.err());
    }
    let parent = names::parent(&path);
    let name = names::free(names::base(&path), |n| taken(conn, parent, n, None))?;
    let path = names::join(parent, &name);
    let tx = conn.transaction()?;
    ensure(&tx, &path)?;
    tx.commit()?;
    Ok(path)
}

/// Adds the cleaned `path` and its parents when they're missing; existing ones are left alone.
pub(super) fn ensure(tx: &Transaction, path: &str) -> AppResult<()> {
    let now = now_ms();
    let mut current = String::new();
    for segment in path.split('/').filter(|s| !s.is_empty()) {
        current = names::join(&current, segment);
        tx.execute(
            "INSERT OR IGNORE INTO task_folders (id, path, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
            params![new_id(), current, now],
        )?;
    }
    Ok(())
}

pub fn rename(conn: &mut Connection, path: &str, name: &str) -> AppResult<String> {
    let typed = names::accept(name)?;
    let current = names::base(path);
    if current == typed {
        return Ok(path.to_string());
    }
    let parent = names::parent(path);
    let name = names::free(&typed, |n| taken(conn, parent, n, Some(current)))?;
    let new = names::join(parent, &name);
    let tx = conn.transaction()?;
    relocate(&tx, path, &new)?;
    tx.commit()?;
    Ok(new)
}

/// Moves the folder into `target`; a name taken there gets a suffix.
pub fn move_to(conn: &mut Connection, path: &str, target: &str) -> AppResult<String> {
    let target = target_path(conn, target)?;
    if names::is_under(&target, path) {
        return Err(errors::INTO_SELF.err());
    }
    if names::parent(path) == target {
        return Err(errors::SAME_FOLDER.err());
    }
    let name = names::free(names::base(path), |n| taken(conn, &target, n, None))?;
    let new = names::join(&target, &name);
    let tx = conn.transaction()?;
    ensure(&tx, &target)?;
    relocate(&tx, path, &new)?;
    tx.commit()?;
    Ok(new)
}

fn relocate(tx: &Transaction, old: &str, new: &str) -> AppResult<()> {
    if old.is_empty() || !exists(tx, old)? {
        return Err(errors::NOT_FOUND.with("Folder was not found"));
    }
    let now = now_ms();
    tx.execute(
        "UPDATE task_folders SET path = ?2 || substr(path, length(?1) + 1), updated_at = ?3
         WHERE path = ?1 OR substr(path, 1, length(?1) + 1) = ?1 || '/'",
        params![old, new, now],
    )?;
    // Tasks followed through ON UPDATE CASCADE; mark them changed so sync carries the new paths.
    tx.execute(
        "UPDATE tasks SET updated_at = ?2
         WHERE path = ?1 OR substr(path, 1, length(?1) + 1) = ?1 || '/'",
        params![new, now],
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
