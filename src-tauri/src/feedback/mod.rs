//! Feedback the user writes from the bottom bar, kept in the vault.

#[cfg(test)]
mod tests;

use rusqlite::{params, Connection};

use crate::error::{AppError, AppResult};
use crate::util::{new_id, now_ms};

pub fn add(conn: &Connection, kind: &str, body: &str, app_version: &str) -> AppResult<()> {
    if !["bug", "idea", "other"].contains(&kind) {
        return Err(AppError::rule(
            "bad_kind",
            "Feedback is a bug, an idea or other.",
        ));
    }
    let body = body.trim();
    if body.is_empty() {
        return Err(AppError::rule("empty_feedback", "Write something first."));
    }
    conn.execute(
        "INSERT INTO feedback (id, kind, body, app_version, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![new_id(), kind, body, app_version, now_ms()],
    )?;
    Ok(())
}
