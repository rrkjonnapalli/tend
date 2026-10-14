//! Brings rows an older app wrote up to this app's schema. They go into an in-memory database at
//! their own version, the same migrations the vault's database ran take them to the latest, and
//! the rows are read back. The steps are deterministic (see `db`), so every device turns the same
//! old rows into the same new ones and the merge sees no false changes.

#[cfg(test)]
mod tests;

use rusqlite::{Connection, OptionalExtension};
use rusqlite_migration::Migrations;

use super::apply::rows;
use super::shards::{self, table_names, Snapshot};
use crate::error::{errors, AppResult};

/// `old` written at schema `from`, migrated to the latest of `migrations`.
pub fn upgrade(old: &Snapshot, from: i64, migrations: &Migrations) -> AppResult<Snapshot> {
    let from = usize::try_from(from).map_err(|_| errors::SYNC_NOT_A_VAULT.err())?;
    let mut conn = Connection::open_in_memory()?;
    migrations.to_version(&mut conn, from)?;
    load(&mut conn, old)?;
    migrations.to_latest(&mut conn)?;
    shards::snapshot(&conn)
}

/// Makes the synced rows equal `old`. The schema's default rows are there already and some can't
/// be deleted, so they are updated in place like any other row.
fn load(conn: &mut Connection, old: &Snapshot) -> AppResult<()> {
    let tx = conn.transaction()?;
    let mut present = Vec::new();
    for table in table_names() {
        // A table a later migration adds isn't there yet.
        if exists(&tx, table)? {
            present.push(table);
        }
    }
    let defaults = shards::snapshot_of(&tx, &present)?;
    rows::write(&tx, &defaults, old)?;
    tx.commit()?;
    Ok(())
}

fn exists(conn: &Connection, table: &str) -> AppResult<bool> {
    let found = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [table],
            |_| Ok(()),
        )
        .optional()?;
    Ok(found.is_some())
}
