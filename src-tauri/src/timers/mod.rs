//! Timers. The database keeps only title, end time and state; the window counts down from
//! `ends_at` and rings when it passes. Nothing fires while Tend is closed.

#[cfg(test)]
mod tests;

use rusqlite::{params, Connection};
use serde::Serialize;

use crate::error::{errors, AppError, AppResult};
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
        return Err(errors::TIMER_DURATION.err());
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

fn not_found() -> AppError {
    errors::NOT_FOUND.with("Timer was not found")
}

/// +1m: one minute from the later of the end time and now.
pub fn add_minute(conn: &Connection, id: &str) -> AppResult<()> {
    let now = now_ms();
    let changed = conn.execute(
        "UPDATE timers SET ends_at = max(ends_at, ?1) + ?2, state = 'running', updated_at = ?1 WHERE id = ?3",
        params![now, MINUTE_MS, id],
    )?;
    if changed == 0 {
        return Err(not_found());
    }
    Ok(())
}

pub fn cancel(conn: &Connection, id: &str) -> AppResult<()> {
    let changed = conn.execute(
        "UPDATE timers SET state = 'cancelled', updated_at = ?1 WHERE id = ?2",
        params![now_ms(), id],
    )?;
    if changed == 0 {
        return Err(not_found());
    }
    Ok(())
}

/// Marks a running timer rung. One that isn't running any more (cancelled, or rung already) is
/// left as it is: the window rings every timer whose time passes, whatever happened to it since.
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
