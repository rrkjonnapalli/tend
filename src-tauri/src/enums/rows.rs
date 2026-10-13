//! Reads and writes `tend_enums`, one kind at a time. The rules of each list (which values are
//! allowed, which labels clash, what can be deleted) live in that list's module.

use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde_json::{Map, Value};

use super::{EnumKind, EnumStatus, TargetType};
use crate::error::AppResult;
use crate::util::now_ms;

/// `meta` keys.
pub const COLOR: &str = "color";
pub const POSITION: &str = "position";

/// The position of a row whose `meta` has none: it sorts after every row that has one.
const LAST: i64 = i64::MAX;

/// A row's place in its list, from its `meta`.
pub fn position(meta: &Map<String, Value>) -> i64 {
    meta.get(POSITION).and_then(Value::as_i64).unwrap_or(LAST)
}

/// One row of a list.
#[derive(Debug, Clone, PartialEq)]
pub struct EnumRow {
    pub value: String,
    pub label: String,
    pub is_default: bool,
    pub status: EnumStatus,
    pub meta: Map<String, Value>,
}

impl EnumRow {
    pub fn position(&self) -> i64 {
        position(&self.meta)
    }

    pub fn text(&self, key: &str) -> String {
        let value = self.meta.get(key).and_then(Value::as_str);
        value.unwrap_or_default().to_string()
    }
}

/// A row as stored: `status` and `meta` still text.
type Stored = (String, String, bool, String, String);

const COLS: &str = "value, label, is_default, status, meta";

fn to_stored(r: &rusqlite::Row) -> rusqlite::Result<Stored> {
    Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
}

fn to_row((value, label, is_default, status, meta): Stored) -> AppResult<EnumRow> {
    let meta = serde_json::from_str(&meta)?;
    Ok(EnumRow {
        value,
        label,
        is_default,
        status: EnumStatus::parse(&status),
        meta,
    })
}

/// The kind's rows in their order (`position`, then value).
pub fn list(conn: &Connection, kind: EnumKind) -> AppResult<Vec<EnumRow>> {
    let stored: Vec<Stored> = conn
        .prepare(&format!("SELECT {COLS} FROM tend_enums WHERE kind = ?1"))?
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
            &format!("SELECT {COLS} FROM tend_enums WHERE kind = ?1 AND value = ?2"),
            params![kind.as_str(), value],
            to_stored,
        )
        .optional()?;
    stored.map(to_row).transpose()
}

/// Refuses a value that is blocked. A value the kind doesn't have passes: whether it must exist is
/// the caller's rule.
pub fn check_active(conn: &Connection, kind: EnumKind, value: &str) -> AppResult<()> {
    let blocked = conn
        .query_row(
            "SELECT 1 FROM tend_enums WHERE kind = ?1 AND value = ?2 AND status = ?3",
            params![kind.as_str(), value, EnumStatus::Blocked.as_str()],
            |_| Ok(()),
        )
        .optional()?;
    if blocked.is_some() {
        return Err(kind.blocked());
    }
    Ok(())
}

/// A label a row of a kind is to get. `except` is that row's value when it already exists (a
/// rename), so its own label doesn't count as taken.
pub struct LabelUse<'a> {
    pub kind: EnumKind,
    pub label: &'a str,
    pub except: Option<&'a str>,
}

/// Whether another row of the kind has this label, ignoring case.
pub fn label_taken(conn: &Connection, wanted: &LabelUse) -> AppResult<bool> {
    let taken = conn
        .query_row(
            "SELECT 1 FROM tend_enums
             WHERE kind = ?1 AND label = ?2 COLLATE NOCASE AND value IS NOT ?3",
            params![wanted.kind.as_str(), wanted.label, wanted.except],
            |_| Ok(()),
        )
        .optional()?;
    Ok(taken.is_some())
}

/// The position after the kind's last row (0 in an empty list). `before` names a row that stays
/// behind the new one, so it isn't counted (Done stays the last status).
pub fn next_position(conn: &Connection, kind: EnumKind, before: Option<&str>) -> AppResult<i64> {
    let next = conn.query_row(
        "SELECT coalesce(max(json_extract(meta, '$.' || ?1)) + 1, 0) FROM tend_enums
         WHERE kind = ?2 AND value IS NOT ?3",
        params![POSITION, kind.as_str(), before],
        |r| r.get(0),
    )?;
    Ok(next)
}

/// A row to add.
pub struct NewRow<'a> {
    pub kind: EnumKind,
    pub value: &'a str,
    pub label: &'a str,
    pub meta: Map<String, Value>,
}

/// One row of a kind, named by its value.
#[derive(Clone, Copy)]
pub struct Key<'a> {
    pub kind: EnumKind,
    pub value: &'a str,
}

/// One `meta` key and the value it gets.
pub struct MetaEntry<'a> {
    pub key: &'a str,
    pub to: Value,
}

/// A row's id: the same on every device (and as the seeded defaults), so a tag made on two
/// devices syncs as one row.
fn row_id(kind: EnumKind, value: &str) -> String {
    format!("{}:{value}", kind.as_str())
}

/// Adds the row and returns it; `None` when a row with the same kind and value is already there
/// (it is left as it is).
pub fn insert(conn: &Connection, new: &NewRow) -> AppResult<Option<EnumRow>> {
    let meta = serde_json::to_string(&new.meta)?;
    let stored = conn
        .query_row(
            &format!(
                "INSERT OR IGNORE INTO tend_enums
                   (id, kind, value, label, meta, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6) RETURNING {COLS}"
            ),
            params![
                row_id(new.kind, new.value),
                new.kind.as_str(),
                new.value,
                new.label,
                meta,
                now_ms()
            ],
            to_stored,
        )
        .optional()?;
    stored.map(to_row).transpose()
}

pub fn set_label(conn: &Connection, row: Key, label: &str) -> AppResult<EnumRow> {
    let stored = conn
        .query_row(
            &format!(
                "UPDATE tend_enums SET label = ?1, updated_at = ?2
                 WHERE kind = ?3 AND value = ?4 RETURNING {COLS}"
            ),
            params![label, now_ms(), row.kind.as_str(), row.value],
            to_stored,
        )
        .optional()?;
    to_row(stored.ok_or_else(|| row.kind.not_found())?)
}

/// Sets one `meta` key. Writes only when the value changes, so an unchanged row keeps its time.
pub fn set_meta(conn: &Connection, row: Key, entry: &MetaEntry) -> AppResult<EnumRow> {
    let to = serde_json::to_string(&entry.to)?;
    let changed = conn
        .query_row(
            &format!(
                "UPDATE tend_enums SET meta = json_set(meta, '$.' || ?1, json(?2)), updated_at = ?3
                 WHERE kind = ?4 AND value = ?5
                   AND json_extract(meta, '$.' || ?1) IS NOT json_extract(?2, '$')
                 RETURNING {COLS}"
            ),
            params![entry.key, to, now_ms(), row.kind.as_str(), row.value],
            to_stored,
        )
        .optional()?;
    match changed {
        Some(stored) => to_row(stored),
        // Nothing written: the row already has this value, or it isn't there.
        None => get(conn, row.kind, row.value)?.ok_or_else(|| row.kind.not_found()),
    }
}

/// Blocks or unblocks the row. Blocking takes the row off everything that carries it, in the
/// caller's transaction; unblocking brings none of that back.
pub fn set_status(tx: &Transaction, row: Key, status: EnumStatus) -> AppResult<()> {
    let changed = tx.execute(
        "UPDATE tend_enums SET status = ?1, updated_at = ?2 WHERE kind = ?3 AND value = ?4",
        params![status.as_str(), now_ms(), row.kind.as_str(), row.value],
    )?;
    if changed == 0 {
        return Err(row.kind.not_found());
    }
    if status == EnumStatus::Blocked {
        tx.execute(
            "DELETE FROM tend_links
             WHERE target_type = ?1 AND target_kind = ?2 AND target_value = ?3",
            params![TargetType::Enum.as_str(), row.kind.as_str(), row.value],
        )?;
    }
    Ok(())
}

/// Deletes the row; its links go with it (trigger `tend_enums_drop_links`).
pub fn delete(conn: &Connection, kind: EnumKind, value: &str) -> AppResult<()> {
    let deleted = conn.execute(
        "DELETE FROM tend_enums WHERE kind = ?1 AND value = ?2",
        params![kind.as_str(), value],
    )?;
    if deleted == 0 {
        return Err(kind.not_found());
    }
    Ok(())
}
