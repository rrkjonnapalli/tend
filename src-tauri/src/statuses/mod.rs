//! Task statuses: `TASK_STATUS` rows of `tend_enums` (app-wide). NOT_STARTED is always first and
//! DONE always last.

mod checks;

#[cfg(test)]
mod tests;

use rusqlite::Connection;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::enums::rows::{self, EnumRow, Key, MetaEntry, NewRow, COLOR, POSITION};
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
    let position = rows::next_position(conn, KIND, Some(DONE))?;
    let mut meta = Map::new();
    meta.insert(COLOR.into(), Value::from(new.color));
    meta.insert(POSITION.into(), Value::from(position));
    let row = NewRow {
        kind: KIND,
        value,
        label: &label,
        meta,
    };
    let added = rows::insert(conn, &row)?;
    // Taken between the check and the insert.
    let added = added.ok_or_else(|| errors::STATUS_VALUE_TAKEN.err())?;
    Ok(to_status(added))
}

/// Refuses a status that is blocked: no task can be given it.
pub fn check_active(conn: &Connection, value: &str) -> AppResult<()> {
    rows::check_active(conn, KIND, value)
}

fn key(value: &str) -> Key<'_> {
    Key { kind: KIND, value }
}

fn get(conn: &Connection, value: &str) -> AppResult<Status> {
    rows::get(conn, KIND, value)?
        .map(to_status)
        .ok_or_else(|| KIND.not_found())
}

/// A status and the label it gets.
pub struct Rename<'a> {
    pub value: &'a str,
    pub label: &'a str,
}

pub fn rename(conn: &Connection, change: &Rename) -> AppResult<Status> {
    let label = checks::label(conn, change.label, Some(change.value))?;
    rows::set_label(conn, key(change.value), &label).map(to_status)
}

/// A status and the color it gets.
pub struct Recolor<'a> {
    pub value: &'a str,
    pub color: &'a str,
}

pub fn recolor(conn: &Connection, change: &Recolor) -> AppResult<Status> {
    crate::colors::check(change.color)?;
    let entry = MetaEntry {
        key: COLOR,
        to: Value::from(change.color),
    };
    rows::set_meta(conn, key(change.value), &entry).map(to_status)
}

/// Sets the order. Not started stays first and Done stays last.
pub fn reorder(conn: &mut Connection, values: &[String]) -> AppResult<()> {
    checks::order(&list(conn)?, values)?;
    // Done keeps its position, so only the others are renumbered.
    crate::db::in_transaction(conn, |tx| {
        for (i, v) in values.iter().enumerate().take(values.len() - 1) {
            let entry = MetaEntry {
                key: POSITION,
                to: Value::from(i as i64),
            };
            rows::set_meta(tx, key(v), &entry)?;
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
