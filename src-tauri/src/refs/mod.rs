//! References: links and named things (a book, a film) worth keeping. Only the value is needed.
//! A link keeps a `url_key`, its URL normalised, so the same link can't be saved twice.

mod title;

#[cfg(test)]
mod tests;

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::db::{columns, tables, Returning, Update};
use crate::enums::{RefKind, SourceType};
use crate::error::{errors, AppError, AppResult};
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

/// What a write to `refs` answers with.
const WRITTEN: Returning<Ref> = Returning {
    columns: COLS,
    map: to_ref,
};

fn not_found() -> AppError {
    errors::NOT_FOUND.with("Reference was not found")
}

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

/// A kind and value a reference is to have. `except` is that reference's id when it already
/// exists (an edit), so its own link doesn't count as taken.
struct Wanted<'a> {
    kind: RefKind,
    value: &'a str,
    except: Option<&'a str>,
}

/// The key a link is checked by; text has none. Refuses a link another reference already has.
fn checked_key(conn: &Connection, wanted: &Wanted) -> AppResult<Option<String>> {
    if wanted.kind != RefKind::Link {
        return Ok(None);
    }
    let key = url_key(wanted.value);
    let taken = conn
        .query_row(
            "SELECT 1 FROM refs WHERE url_key = ?1 AND id IS NOT ?2",
            params![key, wanted.except],
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
    let wanted = Wanted {
        kind: new.kind,
        value: &value,
        except: None,
    };
    let key = checked_key(conn, &wanted)?;
    let added = conn.query_row(
        &format!(
            "INSERT INTO refs (id, kind, value, title, why, url_key, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7) RETURNING {COLS}"
        ),
        params![
            new_id(),
            new.kind.as_str(),
            value,
            kept_title(new.title.as_deref()),
            new.why,
            key,
            now_ms()
        ],
        to_ref,
    )?;
    // A new reference carries no tags yet.
    Ok(added)
}

/// What a reference is after a patch that names its kind or value.
struct Identity {
    kind: RefKind,
    value: String,
    url_key: Option<String>,
}

/// The kind, value and key after `patch`. The key depends on both the kind and the value, so
/// whichever the patch leaves out comes from the reference as it is.
fn checked_identity(conn: &Connection, id: &str, patch: &Patch) -> AppResult<Identity> {
    let current: Option<(String, String)> = conn
        .query_row("SELECT kind, value FROM refs WHERE id = ?1", [id], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .optional()?;
    let (current_kind, current_value) = current.ok_or_else(not_found)?;
    let kind = patch.kind.unwrap_or(RefKind::parse(&current_kind));
    let value = check_value(patch.value.as_deref().unwrap_or(&current_value))?;
    let wanted = Wanted {
        kind,
        value: &value,
        except: Some(id),
    };
    let url_key = checked_key(conn, &wanted)?;
    Ok(Identity {
        kind,
        value,
        url_key,
    })
}

pub fn update(conn: &Connection, id: &str, patch: &Patch) -> AppResult<Ref> {
    // Only when asked: a kind this version doesn't know must survive an edit of the note.
    let names_identity = patch.kind.is_some() || patch.value.is_some();
    let identity = if names_identity {
        Some(checked_identity(conn, id, patch)?)
    } else {
        None
    };
    let mut update = Update::new(tables::REFS);
    if let Some(identity) = &identity {
        update.set(columns::KIND, identity.kind.as_str());
        update.set(columns::VALUE, &identity.value);
        update.set(columns::URL_KEY, &identity.url_key);
    }
    if let Some(title) = &patch.title {
        update.set(columns::TITLE, kept_title(title.as_deref()));
    }
    update.set_some("why", patch.why.as_ref());
    let mut updated = update.run(conn, id, WRITTEN)?.ok_or_else(not_found)?;
    updated.tags = tags::of(conn, SourceType::Ref, id)?;
    Ok(updated)
}

/// Deletes the reference; its tags come off with it (trigger `refs_drop_links`).
pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    let deleted = conn.execute("DELETE FROM refs WHERE id = ?1", [id])?;
    if deleted == 0 {
        return Err(not_found());
    }
    Ok(())
}
