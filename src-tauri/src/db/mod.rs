//! SQLite connection and migrations. `docs/schema.sql` is migration 1, unchanged.

#[cfg(test)]
mod tests;

use std::path::Path;

use rusqlite::{Connection, OpenFlags, OptionalExtension, Transaction};

use crate::error::{errors, AppResult};

pub const APP_ID: &str = "tend";

/// Migration n is `MIGRATIONS[n - 1]`. The app's schema version is the length of this list.
const MIGRATIONS: &[&str] = &[include_str!("../../../docs/schema.sql")];

pub fn schema_version() -> i64 {
    MIGRATIONS.len() as i64
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
    let existed = path.exists();
    let from = if existed { Some(check(path)?) } else { None };

    let mut conn = Connection::open(path)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;

    let current = from.unwrap_or(0);
    migrate(&mut conn, current)?;

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

/// Reads `meta` without writing. Returns the stored schema version when the file belongs to
/// Tend and is not newer than this app.
pub fn check(path: &Path) -> AppResult<i64> {
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| errors::INCOMPATIBLE_DB.err())?;
    let meta = |key: &str| -> Option<String> {
        conn.query_row("SELECT value FROM meta WHERE key = ?1", [key], |r| r.get(0))
            .optional()
            .ok()
            .flatten()
    };
    let app_id = meta("app_id");
    let version = meta("schema_version").and_then(|v| v.parse::<i64>().ok());
    match (app_id.as_deref(), version) {
        (Some(APP_ID), Some(v)) if v >= 1 && v <= schema_version() => Ok(v),
        _ => Err(errors::INCOMPATIBLE_DB.err()),
    }
}

fn migrate(conn: &mut Connection, current: i64) -> AppResult<()> {
    for (i, sql) in MIGRATIONS.iter().enumerate().skip(current as usize) {
        let version = i as i64 + 1;
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.execute(
            "INSERT INTO meta (key, value) VALUES ('schema_version', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [version.to_string()],
        )?;
        tx.commit()?;
    }
    Ok(())
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
    migrate(&mut conn, 0).unwrap();
    conn
}
