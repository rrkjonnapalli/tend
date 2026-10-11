//! Word kinds: `WORD_KIND` rows of `tend_enums`, in their set order. `lookup` says whether the
//! dictionary is asked for words of that kind.

use rusqlite::Connection;
use serde::Serialize;
use serde_json::Value;

use crate::enums::rows::{self, EnumRow};
use crate::enums::EnumKind;
use crate::error::AppResult;

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
