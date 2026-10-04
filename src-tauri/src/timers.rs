//! Timers. The database keeps only title, end time and state; the window counts down from
//! `ends_at` and rings when it passes. Nothing fires while Tend is closed.

use rusqlite::{params, Connection};
use serde::Serialize;

use crate::error::{AppError, AppResult};
use crate::util::{new_id, now_ms};

const MINUTE_MS: i64 = 60_000;

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Timer {
    pub id: String,
    pub title: String,
    pub ends_at: i64,
}

fn running(conn: &Connection, sql_tail: &str, now: Option<i64>) -> AppResult<Vec<Timer>> {
    let sql = format!(
        "SELECT id, title, ends_at FROM timers WHERE state = 'running' {sql_tail} ORDER BY ends_at"
    );
    let map = |r: &rusqlite::Row| {
        Ok(Timer {
            id: r.get(0)?,
            title: r.get(1)?,
            ends_at: r.get(2)?,
        })
    };
    let mut stmt = conn.prepare(&sql)?;
    let rows = match now {
        Some(n) => stmt.query_map([n], map)?.collect::<Result<_, _>>()?,
        None => stmt.query_map([], map)?.collect::<Result<_, _>>()?,
    };
    Ok(rows)
}

/// Running timers, soonest first.
pub fn list(conn: &Connection) -> AppResult<Vec<Timer>> {
    running(conn, "", None)
}

pub fn start(conn: &Connection, title: &str, duration_ms: i64) -> AppResult<Timer> {
    if duration_ms <= 0 {
        return Err(AppError::rule("bad_duration", "must be longer than 0"));
    }
    let now = now_ms();
    let t = Timer {
        id: new_id(),
        title: title.trim().to_string(),
        ends_at: now + duration_ms,
    };
    conn.execute(
        "INSERT INTO timers (id, title, ends_at, state, created_at, updated_at) VALUES (?1, ?2, ?3, 'running', ?4, ?4)",
        params![t.id, t.title, t.ends_at, now],
    )?;
    Ok(t)
}

/// +1m: one minute from the later of the end time and now.
pub fn add_minute(conn: &Connection, id: &str) -> AppResult<()> {
    let now = now_ms();
    conn.execute(
        "UPDATE timers SET ends_at = max(ends_at, ?1) + ?2, state = 'running', updated_at = ?1 WHERE id = ?3",
        params![now, MINUTE_MS, id],
    )?;
    Ok(())
}

pub fn cancel(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE timers SET state = 'cancelled', updated_at = ?1 WHERE id = ?2",
        params![now_ms(), id],
    )?;
    Ok(())
}

pub fn ring(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE timers SET state = 'rung', updated_at = ?1 WHERE id = ?2 AND state = 'running'",
        params![now_ms(), id],
    )?;
    Ok(())
}

/// On launch: running timers that ended while Tend was closed. They are marked rung, so they
/// show only once.
pub fn finished_while_closed(conn: &mut Connection) -> AppResult<Vec<Timer>> {
    let now = now_ms();
    let done = running(conn, "AND ends_at <= ?1", Some(now))?;
    let tx = conn.transaction()?;
    for t in &done {
        tx.execute(
            "UPDATE timers SET state = 'rung', updated_at = ?1 WHERE id = ?2",
            params![now, t.id],
        )?;
    }
    tx.commit()?;
    Ok(done)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_memory;

    #[test]
    fn start_list_extend_cancel_ring() {
        let conn = open_memory();
        assert!(start(&conn, "x", 0).is_err());
        let a = start(&conn, " Tea ", 300_000).unwrap();
        let b = start(&conn, "Motor", 60_000).unwrap();
        assert_eq!(a.title, "Tea");
        assert_eq!(
            list(&conn)
                .unwrap()
                .iter()
                .map(|t| t.id.clone())
                .collect::<Vec<_>>(),
            vec![b.id.clone(), a.id.clone()]
        );
        add_minute(&conn, &b.id).unwrap();
        let ends = list(&conn)
            .unwrap()
            .into_iter()
            .find(|t| t.id == b.id)
            .unwrap()
            .ends_at;
        assert_eq!(ends, b.ends_at + MINUTE_MS);
        cancel(&conn, &a.id).unwrap();
        ring(&conn, &b.id).unwrap();
        assert!(list(&conn).unwrap().is_empty());
    }

    #[test]
    fn plus_a_minute_after_the_end_counts_from_now() {
        let conn = open_memory();
        let t = start(&conn, "", 1).unwrap();
        conn.execute("UPDATE timers SET ends_at = ends_at - 600000", [])
            .unwrap();
        add_minute(&conn, &t.id).unwrap();
        let ends = list(&conn).unwrap()[0].ends_at;
        assert!(ends >= now_ms() + MINUTE_MS - 1000);
    }

    #[test]
    fn overdue_timers_show_once_after_launch() {
        let mut conn = open_memory();
        let gone = start(&conn, "Tea", 1000).unwrap();
        start(&conn, "Later", 600_000).unwrap();
        conn.execute("UPDATE timers SET ends_at = 1 WHERE id = ?1", [&gone.id])
            .unwrap();
        let done = finished_while_closed(&mut conn).unwrap();
        assert_eq!(
            done.iter().map(|t| t.title.as_str()).collect::<Vec<_>>(),
            vec!["Tea"]
        );
        assert!(finished_while_closed(&mut conn).unwrap().is_empty());
        assert_eq!(list(&conn).unwrap().len(), 1);
    }
}
