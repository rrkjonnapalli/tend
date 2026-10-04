//! Feedback the user writes from the bottom bar, kept in the vault.

use rusqlite::{params, Connection};

use crate::error::{AppError, AppResult};
use crate::util::{new_id, now_ms};

pub fn add(conn: &Connection, kind: &str, body: &str, app_version: &str) -> AppResult<()> {
    if !["bug", "idea", "other"].contains(&kind) {
        return Err(AppError::rule("bad_kind", "Feedback is a bug, an idea or other."));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_memory;

    #[test]
    fn saves_feedback_rows() {
        let conn = open_memory();
        add(&conn, "idea", "  dark mode  ", "0.1.0").unwrap();
        assert!(add(&conn, "rant", "x", "0.1.0").is_err());
        assert!(add(&conn, "bug", " ", "0.1.0").is_err());
        let (kind, body, v): (String, String, String) =
            conn.query_row("SELECT kind, body, app_version FROM feedback", [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).unwrap();
        assert_eq!((kind.as_str(), body.as_str(), v.as_str()), ("idea", "dark mode", "0.1.0"));
    }
}
