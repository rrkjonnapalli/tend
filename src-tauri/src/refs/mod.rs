//! References: links and named things (a book, a film) worth keeping. Only the value is needed.
//! A link keeps a `url_key`, its URL normalised, so the same link can't be saved twice.

mod title;

#[cfg(test)]
mod tests;

use rusqlite::{params, Connection, OptionalExtension, ToSql, Transaction};
use serde::{Deserialize, Serialize};

use crate::enums::{RefKind, SourceType};
use crate::error::{errors, AppResult};
use crate::tags;
use crate::util::{new_id, now_ms};

pub use title::fetch as fetch_title;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Ref {
    pub id: String,
    pub kind: RefKind,
    /// The URL or the text as entered.
    pub value: String,
    /// Fetched or typed; `None` shows the value.
    pub title: Option<String>,
    pub why: String,
    pub url_key: Option<String>,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewRef {
    pub kind: RefKind,
    pub value: String,
    pub title: Option<String>,
    #[serde(default)]
    pub why: String,
}

/// Fields an edit can change; absent fields stay. `title: null` clears it.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Patch {
    pub kind: Option<RefKind>,
    pub value: Option<String>,
    #[serde(default, with = "crate::util::nullable")]
    pub title: Option<Option<String>>,
    pub why: Option<String>,
}

/// A link's duplicate key: scheme, one trailing slash and letter case ignored.
pub fn url_key(url: &str) -> String {
    let url = url.trim();
    let lower = url.to_lowercase();
    let mut key = lower.as_str();
    for scheme in ["https://", "http://"] {
        if let Some(rest) = key.strip_prefix(scheme) {
            key = rest;
        }
    }
    key.strip_suffix('/').unwrap_or(key).to_string()
}

const COLS: &str = "id, kind, value, title, why, url_key, created_at, updated_at";

fn to_ref(r: &rusqlite::Row) -> rusqlite::Result<Ref> {
    let kind: String = r.get(1)?;
    Ok(Ref {
        id: r.get(0)?,
        kind: RefKind::parse(&kind),
        value: r.get(2)?,
        title: r.get(3)?,
        why: r.get(4)?,
        url_key: r.get(5)?,
        tags: Vec::new(),
        created_at: r.get(6)?,
        updated_at: r.get(7)?,
    })
}

/// Every reference, newest first, with its tags.
pub fn list(conn: &Connection) -> AppResult<Vec<Ref>> {
    let mut tags = tags::by_record(conn, SourceType::Ref)?;
    let mut refs: Vec<Ref> = conn
        .prepare(&format!(
            "SELECT {COLS} FROM refs ORDER BY created_at DESC, id DESC"
        ))?
        .query_map([], to_ref)?
        .collect::<Result<_, _>>()?;
    for r in &mut refs {
        r.tags = tags.remove(&r.id).unwrap_or_default();
    }
    Ok(refs)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Ref> {
    let mut r = conn
        .query_row(
            &format!("SELECT {COLS} FROM refs WHERE id = ?1"),
            [id],
            to_ref,
        )
        .optional()?
        .ok_or_else(|| errors::NOT_FOUND.with("Reference was not found"))?;
    r.tags = tags::of(conn, SourceType::Ref, id)?;
    Ok(r)
}

fn check_value(value: &str) -> AppResult<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(errors::REF_VALUE_EMPTY.err());
    }
    Ok(value.to_string())
}

/// A typed title, trimmed; blank means none.
fn kept_title(title: Option<&str>) -> Option<String> {
    let title = title.map(str::trim).filter(|t| !t.is_empty());
    title.map(String::from)
}

/// The key a link is checked by; text has none. Refuses a link another reference (not
/// `except`) already has.
fn checked_key(
    conn: &Connection,
    kind: RefKind,
    value: &str,
    except: Option<&str>,
) -> AppResult<Option<String>> {
    if kind != RefKind::Link {
        return Ok(None);
    }
    let key = url_key(value);
    let taken = conn
        .query_row(
            "SELECT 1 FROM refs WHERE url_key = ?1 AND id IS NOT ?2",
            params![key, except],
            |_| Ok(()),
        )
        .optional()?;
    if taken.is_some() {
        return Err(errors::REF_LINK_TAKEN.err());
    }
    Ok(Some(key))
}

pub fn create(conn: &Connection, new: &NewRef) -> AppResult<Ref> {
    let value = check_value(&new.value)?;
    let key = checked_key(conn, new.kind, &value, None)?;
    let id = new_id();
    conn.execute(
        "INSERT INTO refs (id, kind, value, title, why, url_key, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        params![
            id,
            new.kind.as_str(),
            value,
            kept_title(new.title.as_deref()),
            new.why,
            key,
            now_ms()
        ],
    )?;
    get(conn, &id)
}

pub fn update(conn: &mut Connection, id: &str, patch: &Patch) -> AppResult<Ref> {
    let current = get(conn, id)?;
    let kind = patch.kind.unwrap_or(current.kind);
    let value = check_value(patch.value.as_deref().unwrap_or(&current.value))?;
    let key = checked_key(conn, kind, &value, Some(id))?;
    let tx = conn.transaction()?;
    set(&tx, id, "kind", &kind.as_str())?;
    set(&tx, id, "value", &value)?;
    set(&tx, id, "url_key", &key)?;
    if let Some(title) = &patch.title {
        set(&tx, id, "title", &kept_title(title.as_deref()))?;
    }
    if let Some(why) = &patch.why {
        set(&tx, id, "why", why)?;
    }
    set(&tx, id, "updated_at", &now_ms())?;
    tx.commit()?;
    get(conn, id)
}

/// Writes one column. `column` is always one of the literals in `update`, never user input.
fn set(tx: &Transaction, id: &str, column: &'static str, value: &dyn ToSql) -> AppResult<()> {
    tx.execute(
        &format!("UPDATE refs SET {column} = ?1 WHERE id = ?2"),
        params![value, id],
    )?;
    Ok(())
}

/// Deletes the reference; its tags come off with it (trigger `refs_drop_links`).
pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM refs WHERE id = ?1", [id])?;
    Ok(())
}
