//! Applying a merge (spec §10): files are staged, the rows written in one transaction, then the
//! staged files moved into place. If anything fails before the commit, the vault is as it was.

mod files;
mod rows;

#[cfg(test)]
mod tests;

use std::path::Path;

use rusqlite::{Connection, Transaction};
use serde_json::Value;

use super::shards::Snapshot;
use crate::error::AppResult;
use crate::notes::{log_path, Moved, PathKind, Reason};
use crate::util::now_ms;

pub use files::Blobs;

/// The three states an apply works from.
pub struct Change<'a> {
    /// The database's synced rows now.
    pub local: &'a Snapshot,
    /// What they must become.
    pub merged: &'a Snapshot,
    /// The fetched remote commit's rows (whose files `Blobs` holds).
    pub remote: &'a Snapshot,
}

/// Makes the vault match `change.merged`. `also` runs inside the same transaction (the sync run
/// moves its base there).
pub fn apply(
    conn: &mut Connection,
    root: &Path,
    change: &Change,
    blobs: &dyn Blobs,
    also: impl FnOnce(&Transaction) -> AppResult<()>,
) -> AppResult<()> {
    let plan = files::plan(change.local, change.merged, change.remote)?;
    let staged = match files::stage(root, &plan, blobs) {
        Ok(staged) => staged,
        Err(e) => {
            files::discard(root)?;
            return Err(e);
        }
    };
    let written = write_rows(conn, change, also);
    if let Err(e) = written {
        files::discard(root)?;
        return Err(e);
    }
    files::install(root, &plan, &staged)
}

fn write_rows(
    conn: &mut Connection,
    change: &Change,
    also: impl FnOnce(&Transaction) -> AppResult<()>,
) -> AppResult<()> {
    // Immediate: the app's own connection can't write between this check and the commit.
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    // The merge was worked out from `local`; an edit made since then must not be overwritten,
    // so the run starts over (`sync` retries a rejected run once).
    if super::shards::snapshot(&tx)? != *change.local {
        return Err(crate::error::errors::SYNC_REJECTED.with("The vault changed during the sync"));
    }
    rows::write(&tx, change.local, change.merged)?;
    log_moves(&tx, change)?;
    also(&tx)?;
    tx.commit()?;
    Ok(())
}

/// Records in `path_log` every note file and note folder this sync moved.
fn log_moves(tx: &Transaction, change: &Change) -> AppResult<()> {
    let at = now_ms();
    let bodies = note_bodies(change.merged);
    for (table, kind) in [
        ("attachments", PathKind::Note),
        ("note_folders", PathKind::Folder),
    ] {
        let (Some(was), Some(now)) = (change.local.get(table), change.merged.get(table)) else {
            continue;
        };
        for (id, row) in now {
            let is_note_file = table != "attachments" || bodies.contains(id);
            let old = was
                .get(id)
                .and_then(|r| r.get("path"))
                .and_then(Value::as_str);
            let new = row.get("path").and_then(Value::as_str);
            let (Some(old), Some(new)) = (old, new) else {
                continue;
            };
            if old == new || !is_note_file {
                continue;
            }
            let moved = Moved {
                kind,
                reason: Reason::Sync,
                old: old.to_string(),
                new: new.to_string(),
            };
            log_path(tx, &moved, at)?;
        }
    }
    Ok(())
}

/// Attachment ids that are note bodies.
fn note_bodies(s: &Snapshot) -> std::collections::BTreeSet<String> {
    s.get("notes")
        .map(|rows| {
            rows.values()
                .filter_map(|r| r.get("attachment_id").and_then(Value::as_str))
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}
