//! Task comments (markdown).

#[cfg(test)]
mod tests;

use rusqlite::{params, Connection};
use serde::Serialize;

use crate::error::{errors, AppResult};
use crate::util::{new_id, now_ms};

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub id: String,
    pub body: String,
    pub created_at: i64,
    /// Set when the comment was changed after it was written.
    pub edited_at: Option<i64>,
}

fn to_comment(r: &rusqlite::Row) -> rusqlite::Result<Comment> {
    let created_at: i64 = r.get(2)?;
    let updated_at: i64 = r.get(3)?;
    let edited = updated_at > created_at;
    Ok(Comment {
        id: r.get(0)?,
        body: r.get(1)?,
        created_at,
        edited_at: edited.then_some(updated_at),
    })
}

fn check(body: &str) -> AppResult<&str> {
    let body = body.trim();
    if body.is_empty() {
        return Err(errors::COMMENT_EMPTY.err());
    }
    Ok(body)
}

/// Newest first.
pub fn list(conn: &Connection, task: &str) -> AppResult<Vec<Comment>> {
    let rows = conn
        .prepare(
            "SELECT id, body, created_at, updated_at FROM task_comments
             WHERE task_id = ?1 ORDER BY created_at DESC, rowid DESC",
        )?
        .query_map([task], to_comment)?
        .collect::<Result<_, _>>()?;
    Ok(rows)
}

pub fn add(conn: &Connection, task: &str, body: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO task_comments (id, task_id, body, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
        params![new_id(), task, check(body)?, now_ms()],
    )?;
    Ok(())
}

pub fn edit(conn: &Connection, id: &str, body: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE task_comments SET body = ?1, updated_at = max(?2, created_at + 1) WHERE id = ?3",
        params![check(body)?, now_ms(), id],
    )?;
    Ok(())
}
