//! `tend_links`: a record carrying an enum value (a tag, a label) or an attachment. Rows go away
//! with their source, attachment or enum row (triggers in `docs/schema.sql`).

#[cfg(test)]
mod tests;

use rusqlite::{params, Connection};

use crate::enums::{EnumKind, SourceType, TargetType};
use crate::error::AppResult;
use crate::util::{new_id, now_ms};

/// The record a link starts from.
#[derive(Clone, Copy)]
pub struct Source<'a> {
    pub kind: SourceType,
    pub id: &'a str,
}

/// What a link points at: an enum value of a kind, or an attachment id (kind '').
#[derive(Clone, Copy)]
pub struct Target<'a> {
    pub kind: TargetType,
    pub enum_kind: Option<EnumKind>,
    pub value: &'a str,
}

impl<'a> Target<'a> {
    pub fn of_enum(kind: EnumKind, value: &'a str) -> Self {
        Target {
            kind: TargetType::Enum,
            enum_kind: Some(kind),
            value,
        }
    }

    pub fn attachment(id: &'a str) -> Self {
        Target {
            kind: TargetType::Attachment,
            enum_kind: None,
            value: id,
        }
    }

    fn enum_kind_str(&self) -> &'static str {
        self.enum_kind.map_or("", EnumKind::as_str)
    }
}

/// Adds the link; the same link already there is left as it is.
pub fn insert(conn: &Connection, source: Source, target: Target, now: i64) -> AppResult<()> {
    conn.execute(
        "INSERT OR IGNORE INTO tend_links
           (id, source_type, source_id, target_type, target_kind, target_value, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        params![
            new_id(),
            source.kind.as_str(),
            source.id,
            target.kind.as_str(),
            target.enum_kind_str(),
            target.value,
            now
        ],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, source: Source, target: Target) -> AppResult<()> {
    conn.execute(
        "DELETE FROM tend_links WHERE source_type = ?1 AND source_id = ?2
           AND target_type = ?3 AND target_kind = ?4 AND target_value = ?5",
        params![
            source.kind.as_str(),
            source.id,
            target.kind.as_str(),
            target.enum_kind_str(),
            target.value
        ],
    )?;
    Ok(())
}

pub fn delete_by_id(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM tend_links WHERE id = ?1", [id])?;
    Ok(())
}

/// The values of `kind` the source carries, sorted.
pub fn enum_values(conn: &Connection, source: Source, kind: EnumKind) -> AppResult<Vec<String>> {
    let values = conn
        .prepare(
            "SELECT target_value FROM tend_links
             WHERE source_type = ?1 AND source_id = ?2 AND target_type = ?3 AND target_kind = ?4
             ORDER BY target_value",
        )?
        .query_map(
            params![
                source.kind.as_str(),
                source.id,
                TargetType::Enum.as_str(),
                kind.as_str()
            ],
            |r| r.get(0),
        )?
        .collect::<Result<_, _>>()?;
    Ok(values)
}

/// Replaces the values of `kind` the source carries. Only links that change are written, so
/// unchanged ones keep their ids (and aren't recorded as deleted by sync).
pub fn set_enum_values(
    conn: &Connection,
    source: Source,
    kind: EnumKind,
    wanted: &[String],
) -> AppResult<()> {
    let current = enum_values(conn, source, kind)?;
    for value in current.iter().filter(|v| !wanted.contains(v)) {
        delete(conn, source, Target::of_enum(kind, value))?;
    }
    let now = now_ms();
    for value in wanted.iter().filter(|v| !current.contains(v)) {
        insert(conn, source, Target::of_enum(kind, value), now)?;
    }
    Ok(())
}

/// Every (source id, value) pair of `kind` on sources of one type, for lists and boards.
pub fn enum_pairs(
    conn: &Connection,
    source: SourceType,
    kind: EnumKind,
) -> AppResult<Vec<(String, String)>> {
    let pairs = conn
        .prepare(
            "SELECT source_id, target_value FROM tend_links
             WHERE source_type = ?1 AND target_type = ?2 AND target_kind = ?3",
        )?
        .query_map(
            params![source.as_str(), TargetType::Enum.as_str(), kind.as_str()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?
        .collect::<Result<_, _>>()?;
    Ok(pairs)
}

/// How many records carry the value.
pub fn enum_usage(conn: &Connection, kind: EnumKind, value: &str) -> AppResult<i64> {
    let count = conn.query_row(
        "SELECT count(*) FROM tend_links
         WHERE target_type = ?1 AND target_kind = ?2 AND target_value = ?3",
        params![TargetType::Enum.as_str(), kind.as_str(), value],
        |r| r.get(0),
    )?;
    Ok(count)
}
