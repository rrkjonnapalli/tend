//! Task labels: a fixed list kept in Settings → Tasks, in a set order. A task carries any number
//! of them; deleting a label takes it off every task.

#[cfg(test)]
mod tests;

use rusqlite::{params, Connection, ErrorCode, OptionalExtension};
use serde::Serialize;

use crate::colors;
use crate::db::in_transaction;
use crate::error::{errors, AppError, AppResult};
use crate::names;
use crate::util::{new_id, now_ms};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    pub id: String,
    pub name: String,
    pub color: String,
    pub position: i64,
}

const ROW_SQL: &str = "SELECT id, name, color, position FROM task_labels";

fn to_label(r: &rusqlite::Row) -> rusqlite::Result<Label> {
    Ok(Label {
        id: r.get(0)?,
        name: r.get(1)?,
        color: r.get(2)?,
        position: r.get(3)?,
    })
}

pub fn list(conn: &Connection) -> AppResult<Vec<Label>> {
    let labels = conn
        .prepare(&format!("{ROW_SQL} ORDER BY position"))?
        .query_map([], to_label)?
        .collect::<Result<_, _>>()?;
    Ok(labels)
}

fn get(conn: &Connection, id: &str) -> AppResult<Label> {
    conn.query_row(&format!("{ROW_SQL} WHERE id = ?1"), [id], to_label)
        .optional()?
        .ok_or_else(|| errors::NOT_FOUND.with("Label was not found"))
}

/// The trimmed name, which must not be empty.
fn check_name(name: &str) -> AppResult<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(errors::LABEL_NAME_EMPTY.err());
    }
    Ok(name.to_string())
}

/// A unique-key conflict on insert or rename means another label has this name or id; the
/// database is the one check for it.
fn taken_on_conflict(e: rusqlite::Error) -> AppError {
    match e.sqlite_error_code() {
        Some(ErrorCode::ConstraintViolation) => errors::LABEL_NAME_TAKEN.err(),
        _ => e.into(),
    }
}

/// A label's value (`DB_SCRIPT`) as its id (`db_script`): the value is the id shown in capitals,
/// so it follows the name rule. Refuses a value another label already has.
fn id_of(conn: &Connection, value: &str) -> AppResult<String> {
    let value = value.trim();
    let id = value.to_lowercase();
    let capitals = value == value.to_uppercase();
    if !capitals || !names::is_valid(&id) {
        return Err(errors::LABEL_VALUE.err());
    }
    let taken = conn
        .query_row("SELECT 1 FROM task_labels WHERE id = ?1", [&id], |_| Ok(()))
        .optional()?;
    if taken.is_some() {
        return Err(errors::LABEL_VALUE_TAKEN.with(format!(
            "{value} is already used by another label. Change the value."
        )));
    }
    Ok(id)
}

/// A label to add: its name, its value (picked once, never changed) and its color.
pub struct NewLabel<'a> {
    pub name: &'a str,
    pub value: &'a str,
    pub color: &'a str,
}

/// Adds a label at the end of the list. Its value (shown in capitals, stored as its id) is picked
/// when it is added and never changes: tasks and other devices refer to the label by it.
pub fn add(conn: &Connection, new: &NewLabel) -> AppResult<Label> {
    let name = check_name(new.name)?;
    let id = id_of(conn, new.value)?;
    let color = new.color;
    colors::check(color)?;
    let position: i64 = conn.query_row(
        "SELECT coalesce(max(position) + 1, 0) FROM task_labels",
        [],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO task_labels (id, name, color, position, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
        params![id, name, color, position, now_ms()],
    )
    .map_err(taken_on_conflict)?;
    get(conn, &id)
}

pub fn rename(conn: &Connection, id: &str, name: &str) -> AppResult<Label> {
    let name = check_name(name)?;
    conn.execute(
        "UPDATE task_labels SET name = ?1, updated_at = ?2 WHERE id = ?3",
        params![name, now_ms(), id],
    )
    .map_err(taken_on_conflict)?;
    get(conn, id)
}

pub fn recolor(conn: &Connection, id: &str, color: &str) -> AppResult<Label> {
    colors::check(color)?;
    conn.execute(
        "UPDATE task_labels SET color = ?1, updated_at = ?2 WHERE id = ?3",
        params![color, now_ms(), id],
    )?;
    get(conn, id)
}

/// Sets the order; `ids` must be exactly the current labels.
pub fn reorder(conn: &mut Connection, ids: &[String]) -> AppResult<()> {
    let mut proposed: Vec<&str> = ids.iter().map(String::as_str).collect();
    proposed.sort_unstable();
    let current = list(conn)?;
    let mut existing: Vec<&str> = current.iter().map(|l| l.id.as_str()).collect();
    existing.sort_unstable();
    if proposed != existing {
        return Err(errors::LABEL_ORDER.err());
    }
    let now = now_ms();
    in_transaction(conn, |tx| {
        for (position, id) in ids.iter().enumerate() {
            tx.execute(
                "UPDATE task_labels SET position = ?1, updated_at = ?2 WHERE id = ?3 AND position <> ?1",
                params![position as i64, now, id],
            )?;
        }
        Ok(())
    })
}

/// How many tasks carry the label (shown before deleting it).
pub fn usage(conn: &Connection, id: &str) -> AppResult<i64> {
    let count = conn.query_row(
        "SELECT count(*) FROM task_label_links WHERE label_id = ?1",
        [id],
        |r| r.get(0),
    )?;
    Ok(count)
}

/// Deletes the label and takes it off every task (ON DELETE CASCADE).
pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM task_labels WHERE id = ?1", [id])?;
    Ok(())
}

/// The task's label ids, in the list's order.
pub fn of_task(conn: &Connection, task_id: &str) -> AppResult<Vec<String>> {
    let ids = conn
        .prepare(
            "SELECT l.id FROM task_label_links x JOIN task_labels l ON l.id = x.label_id
             WHERE x.task_id = ?1 ORDER BY l.position",
        )?
        .query_map([task_id], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    Ok(ids)
}

/// Every (task id, label id) pair, for the task lists and the board.
pub fn links(conn: &Connection) -> AppResult<Vec<(String, String)>> {
    let pairs = conn
        .prepare("SELECT task_id, label_id FROM task_label_links")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<_, _>>()?;
    Ok(pairs)
}

/// Replaces the task's labels. Every id must be a configured label.
pub fn set_for_task(
    conn: &mut Connection,
    task_id: &str,
    ids: &[String],
) -> AppResult<Vec<String>> {
    for id in ids {
        get(conn, id)?;
    }
    let current = of_task(conn, task_id)?;
    let now = now_ms();
    // Only links that change are written, so unchanged ones keep their ids.
    in_transaction(conn, |tx| {
        for id in current.iter().filter(|id| !ids.contains(id)) {
            tx.execute(
                "DELETE FROM task_label_links WHERE task_id = ?1 AND label_id = ?2",
                params![task_id, id],
            )?;
        }
        for id in ids.iter().filter(|id| !current.contains(id)) {
            tx.execute(
                "INSERT OR IGNORE INTO task_label_links (id, task_id, label_id, updated_at)
                 VALUES (?1, ?2, ?3, ?4)",
                params![new_id(), task_id, id, now],
            )?;
        }
        Ok(())
    })?;
    of_task(conn, task_id)
}
