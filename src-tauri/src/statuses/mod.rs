//! Task statuses: `TASK_STATUS` rows of `tend_enums` (app-wide). NOT_STARTED is always first and
//! DONE always last.

mod checks;

#[cfg(test)]
mod tests;

use rusqlite::Connection;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::enums::rows::{self, EnumRow, NewRow, COLOR, POSITION};
use crate::enums::EnumKind;
use crate::error::{errors, AppResult};

const KIND: EnumKind = EnumKind::TaskStatus;
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

fn to_status(row: EnumRow) -> Status {
    Status {
        color: row.text(COLOR),
        position: row.position(),
        is_default: row.is_default,
        value: row.value,
        label: row.label,
    }
}

pub fn list(conn: &Connection) -> AppResult<Vec<Status>> {
    let statuses = rows::list(conn, KIND)?.into_iter().map(to_status).collect();
    Ok(statuses)
}

/// A status to add: its label, its value (picked once, never changed) and its color.
pub struct NewStatus<'a> {
    pub label: &'a str,
    pub value: &'a str,
    pub color: &'a str,
}

/// Adds a custom status just before Done.
pub fn add(conn: &Connection, new: &NewStatus) -> AppResult<Status> {
    let value = checks::new_value(conn, new.value)?;
    let label = checks::label(conn, new.label, None)?;
    crate::colors::check(new.color)?;
    let before_done = list(conn)?.into_iter().filter(|s| s.value != DONE);
    let position = before_done.map(|s| s.position + 1).max().unwrap_or(0);
    let mut meta = Map::new();
    meta.insert(COLOR.into(), Value::from(new.color));
    meta.insert(POSITION.into(), Value::from(position));
    rows::insert(
        conn,
        &NewRow {
            kind: KIND,
            value,
            label: &label,
            meta,
        },
    )?;
    get(conn, value)
}

fn get(conn: &Connection, value: &str) -> AppResult<Status> {
    rows::get(conn, KIND, value)?
        .map(to_status)
        .ok_or_else(|| errors::NOT_FOUND.with("Status was not found"))
}

pub fn rename(conn: &Connection, value: &str, label: &str) -> AppResult<Status> {
    let label = checks::label(conn, label, Some(value))?;
    rows::set_label(conn, KIND, value, &label)?;
    get(conn, value)
}

pub fn recolor(conn: &Connection, value: &str, color: &str) -> AppResult<Status> {
    crate::colors::check(color)?;
    rows::set_meta(conn, KIND, value, COLOR, Value::from(color))?;
    get(conn, value)
}

/// Sets the order. Not started stays first and Done stays last.
pub fn reorder(conn: &mut Connection, values: &[String]) -> AppResult<()> {
    checks::order(&list(conn)?, values)?;
    // Done keeps its position, so only the others are renumbered.
    crate::db::in_transaction(conn, |tx| {
        for (i, v) in values.iter().enumerate().take(values.len() - 1) {
            rows::set_meta(tx, KIND, v, POSITION, Value::from(i as i64))?;
        }
        Ok(())
    })
}

/// Deletes a custom status that no task uses. Not atomic with the check: a task given this status
/// at the same moment falls back to the first status on the next sync.
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
    rows::delete(conn, KIND, value)
}
