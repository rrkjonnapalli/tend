//! Task statuses (app-wide). NOT_STARTED is always first and DONE always last.

mod checks;

#[cfg(test)]
mod tests;

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

use crate::error::{errors, AppResult};
use crate::util::now_ms;

/// Always the first status; new tasks start here.
pub const FIRST: &str = "NOT_STARTED";
/// Always the last status; a task here is complete.
pub const DONE: &str = "DONE";
const RESERVED: [&str; 3] = [FIRST, "IN_PROGRESS", DONE];

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub value: String,
    pub label: String,
    pub color: String,
    pub position: i64,
    pub is_default: bool,
}

const ROW_SQL: &str = "SELECT value, label, color, position, is_default FROM statuses";

fn to_status(r: &rusqlite::Row) -> rusqlite::Result<Status> {
    Ok(Status {
        value: r.get(0)?,
        label: r.get(1)?,
        color: r.get(2)?,
        position: r.get(3)?,
        is_default: r.get(4)?,
    })
}

pub fn list(conn: &Connection) -> AppResult<Vec<Status>> {
    let rows = conn
        .prepare(&format!("{ROW_SQL} ORDER BY position"))?
        .query_map([], to_status)?
        .collect::<Result<_, _>>()?;
    Ok(rows)
}

/// Adds a custom status just before Done.
pub fn add(conn: &Connection, label: &str, value: &str, color: &str) -> AppResult<Status> {
    let value = checks::new_value(conn, value)?;
    let label = checks::label(conn, label, None)?;
    crate::colors::check(color)?;
    let position: i64 = conn.query_row(
        "SELECT max(position) + 1 FROM statuses WHERE value <> ?1",
        [DONE],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO statuses (id, value, label, color, position, is_default, updated_at)
         VALUES (?1, ?1, ?2, ?3, ?4, 0, ?5)",
        params![value, label, color, position, now_ms()],
    )?;
    get(conn, value)
}

fn get(conn: &Connection, value: &str) -> AppResult<Status> {
    conn.query_row(&format!("{ROW_SQL} WHERE value = ?1"), [value], to_status)
        .optional()?
        .ok_or_else(|| errors::NOT_FOUND.with("Status was not found"))
}

pub fn rename(conn: &Connection, value: &str, label: &str) -> AppResult<Status> {
    let label = checks::label(conn, label, Some(value))?;
    conn.execute(
        "UPDATE statuses SET label = ?1, updated_at = ?2 WHERE value = ?3",
        params![label, now_ms(), value],
    )?;
    get(conn, value)
}

pub fn recolor(conn: &Connection, value: &str, color: &str) -> AppResult<Status> {
    crate::colors::check(color)?;
    conn.execute(
        "UPDATE statuses SET color = ?1, updated_at = ?2 WHERE value = ?3",
        params![color, now_ms(), value],
    )?;
    get(conn, value)
}

/// Sets the order. Not started stays first and Done stays last.
pub fn reorder(conn: &mut Connection, values: &[String]) -> AppResult<()> {
    checks::order(&list(conn)?, values)?;
    // Done keeps its position, so only the others are renumbered.
    let now = now_ms();
    let tx = conn.transaction()?;
    for (i, v) in values.iter().enumerate().take(values.len() - 1) {
        tx.execute(
            "UPDATE statuses SET position = ?1, updated_at = ?2 WHERE value = ?3 AND position <> ?1",
            params![i as i64, now, v],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Deletes a custom status that no task uses.
pub fn delete(conn: &Connection, value: &str) -> AppResult<()> {
    let s = get(conn, value)?;
    if s.is_default {
        return Err(errors::STATUS_DEFAULT.err());
    }
    let used: i64 = conn.query_row(
        "SELECT count(*) FROM tasks WHERE status = ?1",
        [value],
        |r| r.get(0),
    )?;
    if used > 0 {
        return Err(errors::STATUS_IN_USE.with(format!("Can’t delete “{}”: some tasks still have this status. Move them to another status first.", s.label)));
    }
    conn.execute("DELETE FROM statuses WHERE value = ?1", [value])?;
    Ok(())
}
