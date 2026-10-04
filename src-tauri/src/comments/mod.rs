//! Task comments (markdown).

#[cfg(test)]
mod tests;

use rusqlite::{params, Connection};
use serde::Serialize;

use crate::error::{AppError, AppResult};
use crate::util::{new_id, now_ms};

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub id: String,
    pub body: String,
    pub created_at: i64,
    pub edited_at: Option<i64>,
}

fn check(body: &str) -> AppResult<&str> {
    let body = body.trim();
    if body.is_empty() {
        return Err(AppError::rule("empty_comment", "A comment can’t be empty"));
    }
    Ok(body)
}

/// Newest first.
pub fn list(conn: &Connection, task: &str) -> AppResult<Vec<Comment>> {
    let rows = conn
        .prepare("SELECT id, body, created_at, edited_at FROM task_comments WHERE task_id = ?1 ORDER BY created_at DESC, rowid DESC")?
        .query_map([task], |r| Ok(Comment { id: r.get(0)?, body: r.get(1)?, created_at: r.get(2)?, edited_at: r.get(3)? }))?
        .collect::<Result<_, _>>()?;
    Ok(rows)
}

pub fn add(conn: &Connection, task: &str, body: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO task_comments (id, task_id, body, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![new_id(), task, check(body)?, now_ms()],
    )?;
    Ok(())
}

pub fn edit(conn: &Connection, id: &str, body: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE task_comments SET body = ?1, edited_at = ?2 WHERE id = ?3",
        params![check(body)?, now_ms(), id],
    )?;
    Ok(())
}
