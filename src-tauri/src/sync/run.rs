//! Connecting, restoring and one sync run (spec §13, §14). The caller pauses the notes watcher and
//! scans the vault first; afterwards it reloads the stores.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use git2::Oid;
use rusqlite::{Connection, Transaction};

use super::apply::{apply, Change};
use super::git::{Kept, Remote, Repo};
use super::reconcile::reconcile;
use super::shards::{self, Snapshot};
use super::state;
use crate::error::{errors, AppResult};
use crate::vault::SYNC_DIR;

/// Files above this are not committed (host limits); their rows still sync.
const MAX_FILE_BYTES: u64 = 100 * 1024 * 1024;

/// What a run did.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing changed on either side.
    Idle,
    /// Took the remote's changes; nothing of ours to push.
    Pulled,
    /// Pushed a new commit.
    Pushed,
}

/// The vault files a commit holds: shard files and every file a row points at that is on disk
/// (and under the size limit).
fn synced_paths(conn: &Connection, root: &Path) -> AppResult<Vec<String>> {
    let mut paths = Vec::new();
    collect_files(root, Path::new(SYNC_DIR), &mut paths)?;
    let mut stmt =
        conn.prepare("SELECT path FROM attachments WHERE missing_at IS NULL ORDER BY path")?;
    let files: Vec<String> = stmt
        .query_map([], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    for path in files {
        let size = fs::metadata(root.join(&path)).map(|m| m.len());
        if size.is_ok_and(|s| s <= MAX_FILE_BYTES) {
            paths.push(path);
        }
    }
    Ok(paths)
}

/// Every file under `dir` (relative to `root`), as vault-relative paths.
fn collect_files(root: &Path, dir: &Path, out: &mut Vec<String>) -> AppResult<()> {
    let Ok(entries) = fs::read_dir(root.join(dir)) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        let rel = dir.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            collect_files(root, &rel, out)?;
        } else {
            out.push(rel.to_string_lossy().into_owned());
        }
    }
    Ok(())
}

/// The tree to commit on top of `parent`: the vault's synced files, plus the parent's files Tend
/// doesn't manage (a README made with the repo, say).
fn tree_now(conn: &Connection, root: &Path, repo: &Repo, parent: Option<Oid>) -> AppResult<Oid> {
    let paths = synced_paths(conn, root)?;
    let Some(commit) = parent else {
        return repo.tree_of_files(&paths, None);
    };
    let managed = managed_paths(repo, commit)?;
    let kept = Kept {
        commit,
        managed: &managed,
    };
    repo.tree_of_files(&paths, Some(kept))
}

/// The files Tend manages in a commit: those its `attachments` rows point at. None in a commit
/// that holds no vault.
fn managed_paths(repo: &Repo, commit: Oid) -> AppResult<BTreeSet<String>> {
    if !holds_vault(repo, commit)? {
        return Ok(BTreeSet::new());
    }
    let rows = rows_at(repo, commit)?;
    let paths = rows["attachments"]
        .values()
        .filter_map(|row| row.get("path")?.as_str())
        .map(String::from)
        .collect();
    Ok(paths)
}

fn holds_vault(repo: &Repo, commit: Oid) -> AppResult<bool> {
    Ok(!repo.files_under(commit, SYNC_DIR)?.is_empty())
}

/// The rows a commit holds.
fn rows_at(repo: &Repo, commit: Oid) -> AppResult<Snapshot> {
    let files = repo.files_under(commit, SYNC_DIR)?;
    shards::check_manifest(&files)?;
    shards::parse(&files)
}

fn oid(text: &str) -> AppResult<Oid> {
    Oid::from_str(text).map_err(|_| errors::SYNC_BASE_LOST.err())
}

/// Starts syncing a vault to a remote that holds no vault yet (it may be empty, or hold a README
/// and the like, which stay). The first commit is the base.
pub fn connect(conn: &Connection, root: &Path, remote: &Remote) -> AppResult<()> {
    let repo = Repo::open(root)?;
    let head = repo.fetch(remote)?;
    if let Some(head) = head {
        if holds_vault(&repo, head)? {
            return Err(errors::SYNC_REMOTE_NOT_EMPTY.err());
        }
    }
    shards::export(conn, root)?;
    let tree = tree_now(conn, root, &repo, head)?;
    let commit = repo.commit(remote, tree, head, "Start syncing")?;
    state::set_pending(conn, Some(&commit.to_string()))?;
    repo.push(remote)?;
    state::set_base(conn, &commit.to_string())
}

/// Fills a new, empty vault from the remote; the restored commit is the base.
pub fn restore(conn: &mut Connection, root: &Path, remote: &Remote) -> AppResult<()> {
    let repo = Repo::open(root)?;
    let head = repo
        .fetch(remote)?
        .ok_or_else(|| errors::SYNC_NOT_A_VAULT.err())?;
    let rows = rows_at(&repo, head)?;
    let local = shards::snapshot(conn)?;
    let change = Change {
        local: &local,
        merged: &rows,
        remote: &rows,
    };
    let base = head.to_string();
    let set_base = |tx: &Transaction| state::set_base(tx, &base);
    apply(conn, root, &change, &repo.blobs(head)?, set_base)?;
    shards::export(conn, root)?;
    repo.point_branch(remote, head)
}

/// One sync, retried once when another device pushed in between. Records how it ended.
pub fn sync(conn: &mut Connection, root: &Path, remote: &Remote) -> AppResult<Outcome> {
    let mut result = run_once(conn, root, remote);
    let rejected = matches!(&result, Err(e) if e.is(errors::SYNC_REJECTED));
    if rejected {
        result = run_once(conn, root, remote);
    }
    let key = match &result {
        Ok(_) => state::OK.to_string(),
        Err(e) => e.key().to_string(),
    };
    state::record(conn, &key)?;
    result
}

/// The base for this run: a pending commit the remote now has becomes it (a crash after a push).
fn resolve_base(conn: &Connection, repo: &Repo, head: Oid) -> AppResult<Oid> {
    let saved = state::load(conn)?;
    let base = saved.base.ok_or_else(|| errors::SYNC_NOT_CONNECTED.err())?;
    if let Some(pending) = saved.pending {
        let pushed = oid(&pending)?;
        let landed = repo.has(pushed) && repo.contains(head, pushed)?;
        if landed {
            state::set_base(conn, &pending)?;
            return Ok(pushed);
        }
        state::set_pending(conn, None)?;
    }
    let base = oid(&base)?;
    let known = repo.has(base) && repo.contains(head, base)?;
    if !known {
        return Err(errors::SYNC_BASE_LOST.err());
    }
    Ok(base)
}

fn run_once(conn: &mut Connection, root: &Path, remote: &Remote) -> AppResult<Outcome> {
    let repo = Repo::open(root)?;
    let head = repo
        .fetch(remote)?
        .ok_or_else(|| errors::SYNC_BASE_LOST.err())?;
    let base = resolve_base(conn, &repo, head)?;
    shards::export(conn, root)?;
    let tree = tree_now(conn, root, &repo, Some(head))?;
    let idle = head == base && tree == repo.tree_of(base)?;
    if idle {
        return Ok(Outcome::Idle);
    }
    merge_in(conn, root, &repo, (base, head))?;
    shards::export(conn, root)?;
    let tree = tree_now(conn, root, &repo, Some(head))?;
    if tree == repo.tree_of(head)? {
        repo.point_branch(remote, head)?;
        return Ok(Outcome::Pulled);
    }
    let commit = repo.commit(remote, tree, Some(head), "Sync")?;
    state::set_pending(conn, Some(&commit.to_string()))?;
    repo.push(remote)?;
    state::set_base(conn, &commit.to_string())?;
    Ok(Outcome::Pushed)
}

/// Merges the remote's changes since the base into the vault. The fetched head becomes the base
/// in the same transaction: from here on this device holds everything the remote has.
fn merge_in(
    conn: &mut Connection,
    root: &Path,
    repo: &Repo,
    (base, head): (Oid, Oid),
) -> AppResult<()> {
    let base_rows = rows_at(repo, base)?;
    let remote_rows = rows_at(repo, head)?;
    let local_rows = shards::snapshot(conn)?;
    let merged = reconcile(&base_rows, &local_rows, &remote_rows);
    let change = Change {
        local: &local_rows,
        merged: &merged,
        remote: &remote_rows,
    };
    let head_text = head.to_string();
    let set_base = |tx: &Transaction| state::set_base(tx, &head_text);
    apply(conn, root, &change, &repo.blobs(head)?, set_base)?;
    Ok(())
}
