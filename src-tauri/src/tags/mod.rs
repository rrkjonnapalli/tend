//! The shared tag pool. A tag is its name, a slug under the name rule (`ui_design`), which is also
//! its id; notes and tasks link to it. Tags are never renamed.

#[cfg(test)]
mod tests;

use rusqlite::{params, Connection};

use crate::error::AppResult;
use crate::names;
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
        .prepare("SELECT id FROM tags ORDER BY id")?
        .query_map([], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    Ok(names)
}

pub fn of(conn: &Connection, owner: Owner, id: &str) -> AppResult<Vec<String>> {
    let (table, col) = owner.table();
    let sql = format!("SELECT tag_id FROM {table} WHERE {col} = ?1 ORDER BY tag_id");
    let names = conn
        .prepare(&sql)?
        .query_map([id], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    Ok(names)
}

/// The tags typed text names, as saved: each slugged, blanks and duplicates dropped, in order.
fn tag_names(typed: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for name in typed.iter().filter_map(|t| names::slug(t)) {
        if !out.contains(&name) {
            out.push(name);
        }
    }
    out
}

fn ensure(conn: &Connection, tag: &str) -> AppResult<()> {
    conn.execute(
        "INSERT OR IGNORE INTO tags (id, created_at, updated_at) VALUES (?1, ?2, ?2)",
        params![tag, now_ms()],
    )?;
    Ok(())
}

/// Replaces the owner's tags with `names`, each saved as its slug. Text with no letter or digit
/// is ignored; tags no longer used anywhere stay in the
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
    let wanted = tag_names(names);
    for tag in &wanted {
        ensure(&tx, tag)?;
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
