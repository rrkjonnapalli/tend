//! The shared tag pool. Notes and tasks link to tags by id; names are case-insensitive.

#[cfg(test)]
mod tests;

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::AppResult;
use crate::util::{new_id, now_ms};

/// Which owner table a tag list belongs to.
#[derive(Clone, Copy)]
pub enum Owner {
    Note,
    Task,
}

impl Owner {
    fn table(self) -> (&'static str, &'static str) {
        match self {
            Owner::Note => ("note_tags", "note_id"),
            Owner::Task => ("task_tags", "task_id"),
        }
    }
}

pub fn all(conn: &Connection) -> AppResult<Vec<String>> {
    let names = conn
        .prepare("SELECT name FROM tags ORDER BY name COLLATE NOCASE")?
        .query_map([], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    Ok(names)
}

pub fn of(conn: &Connection, owner: Owner, id: &str) -> AppResult<Vec<String>> {
    let (table, col) = owner.table();
    let sql = format!(
        "SELECT t.name FROM {table} x JOIN tags t ON t.id = x.tag_id WHERE x.{col} = ?1 ORDER BY t.name COLLATE NOCASE"
    );
    let names = conn
        .prepare(&sql)?
        .query_map([id], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    Ok(names)
}

/// Finds a tag by name (any case) or creates it.
fn ensure(conn: &Connection, name: &str) -> AppResult<String> {
    let found: Option<String> = conn
        .query_row("SELECT id FROM tags WHERE name = ?1", [name], |r| r.get(0))
        .optional()?;
    if let Some(id) = found {
        return Ok(id);
    }
    let id = new_id();
    conn.execute(
        "INSERT INTO tags (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
        params![id, name, now_ms()],
    )?;
    Ok(id)
}

/// Replaces the owner's tags. Blank names are ignored; tags no longer used anywhere stay in the
/// pool. Only links that change are written, so unchanged links keep their ids (and aren't
/// recorded as deleted).
pub fn set(
    conn: &mut Connection,
    owner: Owner,
    id: &str,
    names: &[String],
) -> AppResult<Vec<String>> {
    let (table, col) = owner.table();
    let tx = conn.transaction()?;
    let mut wanted = Vec::new();
    for name in names.iter().map(|n| n.trim()).filter(|n| !n.is_empty()) {
        wanted.push(ensure(&tx, name)?);
    }
    let current: Vec<String> = tx
        .prepare(&format!("SELECT tag_id FROM {table} WHERE {col} = ?1"))?
        .query_map([id], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    for tag in current.iter().filter(|t| !wanted.contains(t)) {
        tx.execute(
            &format!("DELETE FROM {table} WHERE {col} = ?1 AND tag_id = ?2"),
            params![id, tag],
        )?;
    }
    let now = now_ms();
    for tag in wanted.iter().filter(|t| !current.contains(t)) {
        tx.execute(
            &format!("INSERT OR IGNORE INTO {table} (id, {col}, tag_id, updated_at) VALUES (?1, ?2, ?3, ?4)"),
            params![new_id(), id, tag, now],
        )?;
    }
    tx.commit()?;
    of(conn, owner, id)
}
