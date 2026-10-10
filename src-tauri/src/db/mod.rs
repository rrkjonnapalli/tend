//! SQLite connection and migrations, run by `rusqlite_migration`. `docs/schema.sql` is migration 1;
//! later changes are new entries in `STEPS`. The schema version lives in `PRAGMA user_version`.

#[cfg(test)]
mod tests;

use std::path::Path;

use rusqlite::{Connection, OpenFlags, OptionalExtension, Transaction, TransactionBehavior};
use rusqlite_migration::{Migrations, M};

use crate::error::{errors, AppError, AppResult};

pub const APP_ID: &str = "tend";

/// SQL: `path` is the folder `?1` or anything under it.
pub const UNDER: &str = "(path = ?1 OR substr(path, 1, length(?1) + 1) = ?1 || '/')";
const BUSY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// Migration n is `STEPS[n - 1]`. Each step is checked for broken foreign keys before it commits.
const STEPS: &[M<'static>] = &[
    M::up(include_str!("../../../docs/schema.sql")).foreign_key_check(),
    M::up(include_str!("../../../docs/migrations/002_order_keys.sql")).foreign_key_check(),
];
const MIGRATIONS: Migrations<'static> = Migrations::from_slice(STEPS);

pub fn schema_version() -> i64 {
    STEPS.len() as i64
}

impl From<rusqlite_migration::Error> for AppError {
    fn from(e: rusqlite_migration::Error) -> Self {
        errors::DB.with(e.to_string())
    }
}

/// What happened to the database while opening it.
#[derive(Debug, PartialEq, Eq)]
pub enum Opened {
    Created,
    Current,
    Migrated { from: i64, to: i64 },
}

/// Opens or creates the database. An existing file is checked read-only first, so an
/// incompatible file is never modified.
pub fn open(path: &Path) -> AppResult<(Connection, Opened)> {
    let from = if path.exists() {
        Some(check(path)?)
    } else {
        None
    };
    let mut conn = Connection::open(path)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    // The app and the sync runner each hold a connection; a writer waits this long for the other.
    conn.busy_timeout(BUSY_TIMEOUT)?;
    // Every transaction takes the write lock when it starts. A deferred one that reads first and
    // writes later fails at once ("database is locked") if the other connection committed in
    // between; an immediate one waits its turn (BUSY_TIMEOUT) instead.
    conn.set_transaction_behavior(TransactionBehavior::Immediate);
    MIGRATIONS.to_latest(&mut conn)?;
    let opened = match from {
        None => Opened::Created,
        Some(v) if v == schema_version() => Opened::Current,
        Some(v) => Opened::Migrated {
            from: v,
            to: schema_version(),
        },
    };
    Ok((conn, opened))
}

/// Reads the file without writing. Returns its schema version when it belongs to Tend and is not
/// newer than this app.
pub fn check(path: &Path) -> AppResult<i64> {
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| errors::INCOMPATIBLE_DB.err())?;
    let app_id: Option<String> = conn
        .query_row("SELECT value FROM meta WHERE key = 'app_id'", [], |r| {
            r.get(0)
        })
        .optional()
        .ok()
        .flatten();
    let version: Option<i64> = conn.query_row("PRAGMA user_version", [], |r| r.get(0)).ok();
    match (app_id.as_deref(), version) {
        (Some(APP_ID), Some(v)) if v >= 1 && v <= schema_version() => Ok(v),
        _ => Err(errors::INCOMPATIBLE_DB.err()),
    }
}

/// For changes that touch disk first: runs `step` in one transaction and, if it fails, calls
/// `undo` to put the disk back, then returns the database error. Undo is best effort; if it
/// fails too, the next scan re-indexes what is on disk.
pub fn commit_or_undo<T>(
    conn: &mut Connection,
    undo: impl FnOnce(),
    step: impl FnOnce(&Transaction) -> AppResult<T>,
) -> AppResult<T> {
    let result = in_transaction(conn, step);
    if result.is_err() {
        undo();
    }
    result
}

/// Runs `step` in one transaction, committed only when it succeeds.
pub fn in_transaction<T>(
    conn: &mut Connection,
    step: impl FnOnce(&Transaction) -> AppResult<T>,
) -> AppResult<T> {
    let tx = conn.transaction()?;
    let out = step(&tx)?;
    tx.commit()?;
    Ok(out)
}

#[cfg(test)]
pub fn open_memory() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    MIGRATIONS.to_latest(&mut conn).unwrap();
    conn
}
