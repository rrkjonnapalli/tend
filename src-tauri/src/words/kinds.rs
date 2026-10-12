//! Word kinds: `WORD_KIND` rows of `tend_enums`, in their set order. `lookup` says whether the
//! dictionary is asked for words of that kind.

use rusqlite::Connection;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::db::in_transaction;
use crate::enums::rows::{self, EnumRow, NewRow, POSITION};
use crate::enums::EnumKind;
use crate::error::{errors, AppResult};
use crate::names;

pub const KIND: EnumKind = EnumKind::WordKind;
/// `meta` key: the dictionary is asked for this kind.
const LOOKUP: &str = "lookup";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Kind {
    pub value: String,
    pub label: String,
    pub position: i64,
    pub lookup: bool,
    pub is_default: bool,
}

fn to_kind(row: EnumRow) -> Kind {
    Kind {
        position: row.position(),
        lookup: row
            .meta
            .get(LOOKUP)
            .and_then(Value::as_bool)
            .unwrap_or(false),
        is_default: row.is_default,
        value: row.value,
        label: row.label,
    }
}

pub fn list(conn: &Connection) -> AppResult<Vec<Kind>> {
    Ok(rows::list(conn, KIND)?.into_iter().map(to_kind).collect())
}

pub fn exists(conn: &Connection, value: &str) -> AppResult<bool> {
    Ok(rows::get(conn, KIND, value)?.is_some())
}

fn get(conn: &Connection, value: &str) -> AppResult<Kind> {
    rows::get(conn, KIND, value)?
        .map(to_kind)
        .ok_or_else(|| errors::NOT_FOUND.with("Word kind was not found"))
}

/// The trimmed name, not empty and not another kind's (ignoring case).
fn check_label(conn: &Connection, label: &str, except: Option<&str>) -> AppResult<String> {
    let label = label.trim();
    if label.is_empty() {
        return Err(errors::WORD_KIND_LABEL_EMPTY.err());
    }
    if rows::label_taken(conn, KIND, label, except)? {
        return Err(errors::WORD_KIND_LABEL_TAKEN.err());
    }
    Ok(label.to_string())
}

/// A new kind's value (`SLANG`): the name rule in capitals, not taken.
fn check_value(conn: &Connection, value: &str) -> AppResult<String> {
    let value = value.trim();
    let capitals = value == value.to_uppercase();
    if !capitals || !names::is_valid(&value.to_lowercase()) {
        return Err(errors::WORD_KIND_VALUE.err());
    }
    if exists(conn, value)? {
        return Err(errors::WORD_KIND_VALUE_TAKEN.err());
    }
    Ok(value.to_string())
}

/// Adds a kind at the end. Kinds added here aren't looked up in the dictionary.
pub fn add(conn: &Connection, label: &str, value: &str) -> AppResult<Kind> {
    let label = check_label(conn, label, None)?;
    let value = check_value(conn, value)?;
    let position = list(conn)?
        .iter()
        .map(|k| k.position + 1)
        .max()
        .unwrap_or(0);
    let mut meta = Map::new();
    meta.insert(LOOKUP.into(), Value::Bool(false));
    meta.insert(POSITION.into(), Value::from(position));
    let new = NewRow {
        kind: KIND,
        value: &value,
        label: &label,
        meta,
    };
    rows::insert(conn, &new)?;
    get(conn, &value)
}

pub fn rename(conn: &Connection, value: &str, label: &str) -> AppResult<Kind> {
    let label = check_label(conn, label, Some(value))?;
    rows::set_label(conn, KIND, value, &label)?;
    get(conn, value)
}

/// Sets the order; `values` must be exactly the current kinds.
pub fn reorder(conn: &mut Connection, values: &[String]) -> AppResult<()> {
    let mut proposed: Vec<&str> = values.iter().map(String::as_str).collect();
    proposed.sort_unstable();
    let current = list(conn)?;
    let mut existing: Vec<&str> = current.iter().map(|k| k.value.as_str()).collect();
    existing.sort_unstable();
    if proposed != existing {
        return Err(errors::WORD_KIND_ORDER.err());
    }
    in_transaction(conn, |tx| {
        for (position, value) in values.iter().enumerate() {
            rows::set_meta(tx, KIND, value, POSITION, Value::from(position as i64))?;
        }
        Ok(())
    })
}

/// How many words have the kind.
pub fn usage(conn: &Connection, value: &str) -> AppResult<i64> {
    let count = conn.query_row("SELECT count(*) FROM words WHERE kind = ?1", [value], |r| {
        r.get(0)
    })?;
    Ok(count)
}

/// Deletes a kind that came with no words and isn't one of Tend's own. Not atomic with the check:
/// a word given this kind at the same moment becomes `OTHER` on the next sync.
pub fn delete(conn: &Connection, value: &str) -> AppResult<()> {
    let kind = get(conn, value)?;
    if kind.is_default {
        return Err(errors::WORD_KIND_DEFAULT.err());
    }
    let used = usage(conn, value)?;
    if used > 0 {
        let message = format!(
            "{used} words have “{}”. Give them another kind first.",
            kind.label
        );
        return Err(errors::WORD_KIND_IN_USE.with(message));
    }
    rows::delete(conn, KIND, value)
}
