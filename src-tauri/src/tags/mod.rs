//! The shared tag pool: `TAG` rows of `tend_enums`. A tag is its slug under the name rule
//! (`ui_design`), kept as the label; its value is the slug in capitals (`UI_DESIGN`). Notes, tasks
//! and Library items carry tags through `tend_links`. Tags are never renamed.

#[cfg(test)]
mod tests;

use std::collections::HashMap;

use rusqlite::Connection;
use serde::Serialize;
use serde_json::Map;

use crate::db::in_transaction;
use crate::enums::rows::{self, Key, NewRow};
use crate::enums::{EnumKind, EnumStatus, SourceType};
use crate::error::AppResult;
use crate::links::{self, Source, Wanted};
use crate::names;

const KIND: EnumKind = EnumKind::Tag;

/// A stored value back to the tag as shown.
fn shown(value: &str) -> String {
    value.to_lowercase()
}

/// A tag in the pool with how it stands: for the list where tags are blocked and unblocked.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TagRow {
    /// The tag as shown, exactly as `all` gives it.
    pub name: String,
    pub status: EnumStatus,
    /// How many records carry it.
    pub uses: i64,
}

/// Every tag that can be used, as shown, sorted. Blocked tags are left out.
pub fn all(conn: &Connection) -> AppResult<Vec<String>> {
    let mut tags: Vec<String> = rows::list(conn, KIND)?
        .into_iter()
        .filter(|r| r.status == EnumStatus::Active)
        .map(|r| r.label)
        .collect();
    tags.sort();
    Ok(tags)
}

/// Every tag in the pool, blocked ones too, sorted by name.
pub fn overview(conn: &Connection) -> AppResult<Vec<TagRow>> {
    let uses = links::enum_usages(conn, KIND)?;
    let mut tags: Vec<TagRow> = rows::list(conn, KIND)?
        .into_iter()
        .map(|r| TagRow {
            uses: uses.get(&r.value).copied().unwrap_or(0),
            name: r.label,
            status: r.status,
        })
        .collect();
    tags.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(tags)
}

/// Blocks or unblocks the tag `name` (as shown). Blocking takes it off every record; unblocking
/// puts it back on none.
pub fn set_status(conn: &mut Connection, name: &str, status: EnumStatus) -> AppResult<()> {
    let value = name.to_uppercase();
    let row = Key {
        kind: KIND,
        value: &value,
    };
    in_transaction(conn, |tx| rows::set_status(tx, row, status))
}

pub fn of(conn: &Connection, owner: SourceType, id: &str) -> AppResult<Vec<String>> {
    let source = Source { kind: owner, id };
    let values = links::enum_values(conn, source, KIND)?;
    Ok(values.iter().map(|v| shown(v)).collect())
}

/// The tags of every record of one type that has any, by record id, each list sorted.
pub fn by_record(conn: &Connection, owner: SourceType) -> AppResult<HashMap<String, Vec<String>>> {
    let mut out: HashMap<String, Vec<String>> = HashMap::new();
    for (id, value) in links::enum_pairs(conn, owner, KIND)? {
        out.entry(id).or_default().push(shown(&value));
    }
    for tags in out.values_mut() {
        tags.sort();
    }
    Ok(out)
}

/// The tags typed text names, as shown: each slugged, blanks and duplicates dropped, in order.
fn tag_names(typed: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for name in typed.iter().filter_map(|t| names::slug(t)) {
        if !out.contains(&name) {
            out.push(name);
        }
    }
    out
}

fn ensure(conn: &Connection, tag: &str) -> AppResult<String> {
    let value = tag.to_uppercase();
    rows::insert(
        conn,
        &NewRow {
            kind: KIND,
            value: &value,
            label: tag,
            meta: Map::new(),
        },
    )?;
    Ok(value)
}

/// Replaces the source's tags with `names`, each saved as its slug. Text with no letter or digit
/// is ignored; tags no longer used anywhere stay in the pool. Adding a blocked tag is refused
/// (`TAG_BLOCKED`) and nothing changes.
pub fn set(conn: &mut Connection, source: Source, names: &[String]) -> AppResult<Vec<String>> {
    in_transaction(conn, |tx| {
        let mut values = Vec::new();
        for tag in tag_names(names) {
            values.push(ensure(tx, &tag)?);
        }
        let wanted = Wanted {
            kind: KIND,
            values: &values,
        };
        links::set_enum_values(tx, source, wanted)
    })?;
    of(conn, source.kind, source.id)
}
