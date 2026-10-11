//! The shared tag pool: `TAG` rows of `tend_enums`. A tag is its slug under the name rule
//! (`ui_design`), kept as the label; its value is the slug in capitals (`UI_DESIGN`). Notes and
//! tasks carry tags through `tend_links`. Tags are never renamed.

#[cfg(test)]
mod tests;

use rusqlite::Connection;
use serde_json::Map;

use crate::db::in_transaction;
use crate::enums::rows::{self, NewRow};
use crate::enums::{EnumKind, SourceType};
use crate::error::AppResult;
use crate::links::{self, Source};
use crate::names;

const KIND: EnumKind = EnumKind::Tag;

/// Which record a tag list belongs to.
#[derive(Clone, Copy)]
pub enum Owner {
    Note,
    Task,
}

impl Owner {
    fn source(self, id: &str) -> Source<'_> {
        let kind = match self {
            Owner::Note => SourceType::Note,
            Owner::Task => SourceType::Task,
        };
        Source { kind, id }
    }
}

/// A stored value back to the tag as shown.
fn shown(value: &str) -> String {
    value.to_lowercase()
}

/// Every tag in the pool, as shown, sorted.
pub fn all(conn: &Connection) -> AppResult<Vec<String>> {
    let mut tags: Vec<String> = rows::list(conn, KIND)?
        .into_iter()
        .map(|r| r.label)
        .collect();
    tags.sort();
    Ok(tags)
}

pub fn of(conn: &Connection, owner: Owner, id: &str) -> AppResult<Vec<String>> {
    let values = links::enum_values(conn, owner.source(id), KIND)?;
    Ok(values.iter().map(|v| shown(v)).collect())
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

/// Replaces the owner's tags with `names`, each saved as its slug. Text with no letter or digit is
/// ignored; tags no longer used anywhere stay in the pool.
pub fn set(
    conn: &mut Connection,
    owner: Owner,
    id: &str,
    names: &[String],
) -> AppResult<Vec<String>> {
    let source = owner.source(id);
    in_transaction(conn, |tx| {
        let mut values = Vec::new();
        for tag in tag_names(names) {
            values.push(ensure(tx, &tag)?);
        }
        links::set_enum_values(tx, source, KIND, &values)
    })?;
    of(conn, owner, id)
}
