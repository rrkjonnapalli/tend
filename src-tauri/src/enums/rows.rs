//! Reads and writes `tend_enums`, one kind at a time. The rules of each list (which values are
//! allowed, which labels clash, what can be deleted) live in that list's module.

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{Map, Value};

use super::EnumKind;
use crate::error::AppResult;
use crate::util::now_ms;

/// `meta` keys.
pub const COLOR: &str = "color";
pub const POSITION: &str = "position";

/// One row of a list.
#[derive(Debug, Clone, PartialEq)]
pub struct EnumRow {
    pub value: String,
    pub label: String,
    pub is_default: bool,
    pub meta: Map<String, Value>,
}

impl EnumRow {
    pub fn position(&self) -> i64 {
        self.meta.get(POSITION).and_then(Value::as_i64).unwrap_or(0)
    }

    pub fn text(&self, key: &str) -> String {
        let value = self.meta.get(key).and_then(Value::as_str);
        value.unwrap_or_default().to_string()
    }
}

/// A row as stored: `meta` still JSON text.
type Stored = (String, String, bool, String);

fn to_stored(r: &rusqlite::Row) -> rusqlite::Result<Stored> {
    Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
}

fn to_row((value, label, is_default, meta): Stored) -> AppResult<EnumRow> {
    let meta = serde_json::from_str(&meta)?;
    Ok(EnumRow {
        value,
        label,
        is_default,
        meta,
    })
}

const ROW_SQL: &str = "SELECT value, label, is_default, meta FROM tend_enums WHERE kind = ?1";

/// The kind's rows in their order (`position`, then value).
pub fn list(conn: &Connection, kind: EnumKind) -> AppResult<Vec<EnumRow>> {
    let stored: Vec<Stored> = conn
        .prepare(ROW_SQL)?
        .query_map([kind.as_str()], to_stored)?
        .collect::<Result<_, _>>()?;
    let mut rows = stored
        .into_iter()
        .map(to_row)
        .collect::<AppResult<Vec<_>>>()?;
    rows.sort_by(|a, b| (a.position(), &a.value).cmp(&(b.position(), &b.value)));
    Ok(rows)
}

pub fn get(conn: &Connection, kind: EnumKind, value: &str) -> AppResult<Option<EnumRow>> {
    let stored = conn
        .query_row(
            &format!("{ROW_SQL} AND value = ?2"),
            params![kind.as_str(), value],
            to_stored,
        )
        .optional()?;
    stored.map(to_row).transpose()
}

/// Whether another row of the kind (not `except`) has this label, ignoring case.
pub fn label_taken(
    conn: &Connection,
    kind: EnumKind,
    label: &str,
    except: Option<&str>,
) -> AppResult<bool> {
    let taken = conn
        .query_row(
            "SELECT 1 FROM tend_enums
             WHERE kind = ?1 AND label = ?2 COLLATE NOCASE AND value IS NOT ?3",
            params![kind.as_str(), label, except],
            |_| Ok(()),
        )
        .optional()?;
    Ok(taken.is_some())
}

/// A row to add.
pub struct NewRow<'a> {
    pub kind: EnumKind,
    pub value: &'a str,
    pub label: &'a str,
    pub meta: Map<String, Value>,
}

/// A row's id: the same on every device (and as the seeded defaults), so a tag made on two
/// devices syncs as one row.
fn row_id(kind: EnumKind, value: &str) -> String {
    format!("{}:{value}", kind.as_str())
}

/// Adds the row; a row with the same kind and value already there is left as it is.
pub fn insert(conn: &Connection, new: &NewRow) -> AppResult<()> {
    let meta = serde_json::to_string(&new.meta)?;
    conn.execute(
        "INSERT OR IGNORE INTO tend_enums (id, kind, value, label, meta, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        params![
            row_id(new.kind, new.value),
            new.kind.as_str(),
            new.value,
            new.label,
            meta,
            now_ms()
        ],
    )?;
    Ok(())
}

pub fn set_label(conn: &Connection, kind: EnumKind, value: &str, label: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE tend_enums SET label = ?1, updated_at = ?2 WHERE kind = ?3 AND value = ?4",
        params![label, now_ms(), kind.as_str(), value],
    )?;
    Ok(())
}

/// Sets one `meta` key. Writes only when the value changes, so an unchanged row keeps its time.
pub fn set_meta(
    conn: &Connection,
    kind: EnumKind,
    value: &str,
    key: &str,
    to: Value,
) -> AppResult<()> {
    let Some(row) = get(conn, kind, value)? else {
        return Ok(());
    };
    if row.meta.get(key) == Some(&to) {
        return Ok(());
    }
    let mut meta = row.meta;
    meta.insert(key.to_string(), to);
    conn.execute(
        "UPDATE tend_enums SET meta = ?1, updated_at = ?2 WHERE kind = ?3 AND value = ?4",
        params![
            serde_json::to_string(&meta)?,
            now_ms(),
            kind.as_str(),
            value
        ],
    )?;
    Ok(())
}

/// Deletes the row; its links go with it (trigger `tend_enums_drop_links`).
pub fn delete(conn: &Connection, kind: EnumKind, value: &str) -> AppResult<()> {
    conn.execute(
        "DELETE FROM tend_enums WHERE kind = ?1 AND value = ?2",
        params![kind.as_str(), value],
    )?;
    Ok(())
}
