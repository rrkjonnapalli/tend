//! Writes the merged rows into SQLite inside the apply's transaction. Ordered so the database's
//! own rules never get in the way halfway through:
//!
//! 1. Foreign keys are checked at commit (`defer_foreign_keys`), so rows can point at parents that
//!    arrive later in the same transaction.
//! 2. Tasks are written first: a status or folder that is going away can only be deleted once no
//!    kept task points at it (those keys are `ON DELETE RESTRICT`, checked at once).
//! 3. Gone rows are deleted, children first.
//! 4. Changed and new rows are written, parents first. A unique column that changes (a label's
//!    name, a path) first gets a placeholder, so two rows swapping values never collide.

use rusqlite::types::Value as SqlValue;
use rusqlite::{params_from_iter, Transaction};
use serde_json::Value;

use crate::error::AppResult;
use crate::sync::shards::{table_names, Row, Snapshot};

/// Written before anything is deleted (see step 2 above), and only then.
const FIRST: &str = "tasks";

/// Columns that must stay unique and can change on an existing row.
const UNIQUE_COLUMNS: &[(&str, &str)] = &[
    ("task_labels", "name"),
    ("statuses", "label"),
    ("attachments", "path"),
    ("note_folders", "path"),
    ("task_folders", "path"),
];

/// Makes the database's synced rows equal `merged`, given they are `local` now.
pub fn write(tx: &Transaction, local: &Snapshot, merged: &Snapshot) -> AppResult<()> {
    tx.pragma_update(None, "defer_foreign_keys", "ON")?;
    upsert(tx, FIRST, local, merged)?;
    for table in table_names().into_iter().rev() {
        delete_gone(tx, table, local, merged)?;
    }
    for table in table_names().into_iter().filter(|t| *t != FIRST) {
        upsert(tx, table, local, merged)?;
    }
    Ok(())
}

fn rows_of<'a>(s: &'a Snapshot, table: &str) -> impl Iterator<Item = (&'a String, &'a Row)> {
    s.get(table).into_iter().flat_map(|rows| rows.iter())
}

fn has(s: &Snapshot, table: &str, id: &str) -> bool {
    s.get(table).is_some_and(|rows| rows.contains_key(id))
}

fn delete_gone(
    tx: &Transaction,
    table: &str,
    local: &Snapshot,
    merged: &Snapshot,
) -> AppResult<()> {
    let sql = format!("DELETE FROM {table} WHERE id = ?1");
    for (id, _) in rows_of(local, table) {
        if !has(merged, table, id) {
            tx.execute(&sql, [id])?;
        }
    }
    Ok(())
}

fn upsert(tx: &Transaction, table: &str, local: &Snapshot, merged: &Snapshot) -> AppResult<()> {
    let old = |id: &str| local.get(table).and_then(|rows| rows.get(id));
    let changed: Vec<(&String, &Row)> = rows_of(merged, table)
        .filter(|(id, row)| old(id).is_none_or(|was| was != *row))
        .collect();
    let unique = UNIQUE_COLUMNS
        .iter()
        .find(|(t, _)| *t == table)
        .map(|(_, column)| *column);
    if let Some(column) = unique {
        park_unique(tx, table, column, &changed, local)?;
    }
    for (id, row) in changed {
        match old(id) {
            Some(_) => update(tx, table, row)?,
            None => insert(tx, table, row)?,
        }
    }
    Ok(())
}

/// Gives every existing row whose unique `column` changes a placeholder value first.
fn park_unique(
    tx: &Transaction,
    table: &str,
    column: &str,
    changed: &[(&String, &Row)],
    local: &Snapshot,
) -> AppResult<()> {
    let sql = format!("UPDATE {table} SET {column} = ?1 WHERE id = ?2");
    for (id, row) in changed {
        let Some(was) = local.get(table).and_then(|rows| rows.get(*id)) else {
            continue;
        };
        if was.get(column) == row.get(column) {
            continue;
        }
        // A control character can't be typed, so this never matches a real value.
        let placeholder = format!("\u{1}{id}");
        tx.execute(&sql, rusqlite::params![placeholder, id])?;
    }
    Ok(())
}

fn update(tx: &Transaction, table: &str, row: &Row) -> AppResult<()> {
    let columns: Vec<&String> = row.keys().filter(|c| *c != "id").collect();
    let sets: Vec<String> = columns
        .iter()
        .enumerate()
        .map(|(i, c)| format!("{c} = ?{}", i + 1))
        .collect();
    let sql = format!(
        "UPDATE {table} SET {} WHERE id = ?{}",
        sets.join(", "),
        columns.len() + 1
    );
    let mut values: Vec<SqlValue> = columns.iter().map(|c| to_sql(&row[*c])).collect();
    values.push(to_sql(&row["id"]));
    tx.execute(&sql, params_from_iter(values))?;
    Ok(())
}

fn insert(tx: &Transaction, table: &str, row: &Row) -> AppResult<()> {
    let columns: Vec<&String> = row.keys().collect();
    let names: Vec<&str> = columns.iter().map(|c| c.as_str()).collect();
    let marks: Vec<String> = (1..=columns.len()).map(|i| format!("?{i}")).collect();
    let sql = format!(
        "INSERT INTO {table} ({}) VALUES ({})",
        names.join(", "),
        marks.join(", ")
    );
    let values: Vec<SqlValue> = columns.iter().map(|c| to_sql(&row[*c])).collect();
    tx.execute(&sql, params_from_iter(values))?;
    Ok(())
}

fn to_sql(value: &Value) -> SqlValue {
    match value {
        Value::Null => SqlValue::Null,
        Value::Bool(b) => SqlValue::Integer(i64::from(*b)),
        Value::Number(n) => match n.as_i64() {
            Some(i) => SqlValue::Integer(i),
            None => SqlValue::Real(n.as_f64().unwrap_or_default()),
        },
        Value::String(s) => SqlValue::Text(s.clone()),
        other => SqlValue::Text(other.to_string()),
    }
}
