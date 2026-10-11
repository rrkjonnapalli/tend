//! Task labels: `TASK_LABEL` rows of `tend_enums`, kept in Settings → Tasks in a set order. A task
//! carries any number of them through `tend_links`; deleting a label takes it off every task.

#[cfg(test)]
mod tests;

use rusqlite::Connection;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::colors;
use crate::db::in_transaction;
use crate::enums::rows::{self, EnumRow, NewRow, COLOR, POSITION};
use crate::enums::{EnumKind, SourceType};
use crate::error::{errors, AppResult};
use crate::links::{self, Source};
use crate::names;

const KIND: EnumKind = EnumKind::TaskLabel;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    /// The label's value (`DB_SCRIPT`): picked once, never changed.
    pub id: String,
    pub name: String,
    pub color: String,
    pub position: i64,
}

fn to_label(row: EnumRow) -> Label {
    Label {
        color: row.text(COLOR),
        position: row.position(),
        id: row.value,
        name: row.label,
    }
}

pub fn list(conn: &Connection) -> AppResult<Vec<Label>> {
    let labels = rows::list(conn, KIND)?.into_iter().map(to_label).collect();
    Ok(labels)
}

fn get(conn: &Connection, id: &str) -> AppResult<Label> {
    rows::get(conn, KIND, id)?
        .map(to_label)
        .ok_or_else(|| errors::NOT_FOUND.with("Label was not found"))
}

/// The trimmed name, which must not be empty or another label's (ignoring case).
fn check_name(conn: &Connection, name: &str, except: Option<&str>) -> AppResult<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(errors::LABEL_NAME_EMPTY.err());
    }
    if rows::label_taken(conn, KIND, name, except)? {
        return Err(errors::LABEL_NAME_TAKEN.err());
    }
    Ok(name.to_string())
}

/// A label's value (`DB_SCRIPT`): the name rule in capitals. Refuses a value another label has.
fn check_value(conn: &Connection, value: &str) -> AppResult<String> {
    let value = value.trim();
    let capitals = value == value.to_uppercase();
    if !capitals || !names::is_valid(&value.to_lowercase()) {
        return Err(errors::LABEL_VALUE.err());
    }
    if rows::get(conn, KIND, value)?.is_some() {
        return Err(errors::LABEL_VALUE_TAKEN.with(format!(
            "{value} is already used by another label. Change the value."
        )));
    }
    Ok(value.to_string())
}

/// A label to add: its name, its value (picked once, never changed) and its color.
pub struct NewLabel<'a> {
    pub name: &'a str,
    pub value: &'a str,
    pub color: &'a str,
}

/// Adds a label at the end of the list. Its value is picked when it is added and never changes:
/// tasks and other devices refer to the label by it.
pub fn add(conn: &Connection, new: &NewLabel) -> AppResult<Label> {
    let name = check_name(conn, new.name, None)?;
    let value = check_value(conn, new.value)?;
    colors::check(new.color)?;
    let position = list(conn)?
        .iter()
        .map(|l| l.position + 1)
        .max()
        .unwrap_or(0);
    let mut meta = Map::new();
    meta.insert(COLOR.into(), Value::from(new.color));
    meta.insert(POSITION.into(), Value::from(position));
    rows::insert(
        conn,
        &NewRow {
            kind: KIND,
            value: &value,
            label: &name,
            meta,
        },
    )?;
    get(conn, &value)
}

pub fn rename(conn: &Connection, id: &str, name: &str) -> AppResult<Label> {
    let name = check_name(conn, name, Some(id))?;
    rows::set_label(conn, KIND, id, &name)?;
    get(conn, id)
}

pub fn recolor(conn: &Connection, id: &str, color: &str) -> AppResult<Label> {
    colors::check(color)?;
    rows::set_meta(conn, KIND, id, COLOR, Value::from(color))?;
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
    in_transaction(conn, |tx| {
        for (position, id) in ids.iter().enumerate() {
            rows::set_meta(tx, KIND, id, POSITION, Value::from(position as i64))?;
        }
        Ok(())
    })
}

/// How many tasks carry the label (shown before deleting it).
pub fn usage(conn: &Connection, id: &str) -> AppResult<i64> {
    links::enum_usage(conn, KIND, id)
}

/// Deletes the label and takes it off every task (trigger `tend_enums_drop_links`).
pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    rows::delete(conn, KIND, id)
}

fn task(id: &str) -> Source<'_> {
    Source {
        kind: SourceType::Task,
        id,
    }
}

/// The task's label ids, in the list's order.
pub fn of_task(conn: &Connection, task_id: &str) -> AppResult<Vec<String>> {
    let carried = links::enum_values(conn, task(task_id), KIND)?;
    let ordered = list(conn)?
        .into_iter()
        .map(|l| l.id)
        .filter(|id| carried.contains(id))
        .collect();
    Ok(ordered)
}

/// Every (task id, label id) pair, for the task lists and the board.
pub fn links(conn: &Connection) -> AppResult<Vec<(String, String)>> {
    links::enum_pairs(conn, SourceType::Task, KIND)
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
    in_transaction(conn, |tx| {
        links::set_enum_values(tx, task(task_id), KIND, ids)
    })?;
    of_task(conn, task_id)
}
