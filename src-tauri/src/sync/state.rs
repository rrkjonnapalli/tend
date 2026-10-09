//! The `sync_state` row: the base of the next merge, the commit being pushed, and the last run.

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::AppResult;
use crate::util::now_ms;

pub struct State {
    /// The commit this device and the remote last agreed on; `None` until connected.
    pub base: Option<String>,
    /// A commit made and being pushed; the base once the remote has it.
    pub pending: Option<String>,
}

pub fn load(conn: &Connection) -> AppResult<State> {
    let row = conn
        .query_row(
            "SELECT last_synced_commit, pending_commit FROM sync_state WHERE id = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let (base, pending) = row.unwrap_or((None, None));
    Ok(State { base, pending })
}

pub fn set_base(conn: &Connection, commit: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE sync_state SET last_synced_commit = ?1, pending_commit = NULL WHERE id = 1",
        [commit],
    )?;
    Ok(())
}

pub fn set_pending(conn: &Connection, commit: Option<&str>) -> AppResult<()> {
    conn.execute(
        "UPDATE sync_state SET pending_commit = ?1 WHERE id = 1",
        [commit],
    )?;
    Ok(())
}

/// What `record` stores for a run that worked.
pub const OK: &str = "ok";

/// How the last run ended: `OK`, or the error key. A run that worked also moves `last_ok_at`.
pub fn record(conn: &Connection, result: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE sync_state SET last_run_at = ?1, last_result = ?2,
           last_ok_at = CASE WHEN ?2 = ?3 THEN ?1 ELSE last_ok_at END
         WHERE id = 1",
        params![now_ms(), result, OK],
    )?;
    Ok(())
}

/// Forgets the base and the pending commit (disconnecting).
pub fn forget(conn: &Connection) -> AppResult<()> {
    conn.execute(
        "UPDATE sync_state SET last_synced_commit = NULL, pending_commit = NULL WHERE id = 1",
        [],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests;
