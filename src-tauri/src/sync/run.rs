//! Connecting, restoring and one sync run (spec §13, §14). The caller scans the vault first;
//! afterwards it reloads the stores. The watcher keeps running: the apply never overwrites a file
//! edited after the merge (`apply::files::install`).

#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use git2::Oid;
use rusqlite::{Connection, Transaction};

use super::apply::{apply, Blobs, Change};
use super::git::{Kept, Remote, Repo};
use super::reconcile::{reconcile, Sides};
use super::shards::{self, Snapshot};
use super::state;
use crate::db::{columns, tables};
use crate::error::{errors, AppResult};
use crate::files;
use crate::names;
use crate::notes;
use crate::vault::SYNC_DIR;

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
    let limit = files::Limit::of(conn)?;
    let mut paths = Vec::new();
    collect_files(root, Path::new(SYNC_DIR), &mut paths);
    let mut stmt =
        conn.prepare("SELECT path FROM attachments WHERE missing_at IS NULL ORDER BY path")?;
    let files: Vec<String> = stmt
        .query_map([], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    for path in files {
        let size = fs::metadata(root.join(&path)).map(|m| m.len());
        if size.is_ok_and(|s| limit.allows(s)) {
            paths.push(path);
        }
    }
    Ok(paths)
}

/// Every file under `dir` (relative to `root`), as vault-relative paths.
fn collect_files(root: &Path, dir: &Path, out: &mut Vec<String>) {
    for (name, kind) in files::shown_entries(&root.join(dir)) {
        let rel = dir.join(name);
        if kind.is_dir() {
            collect_files(root, &rel, out);
        } else {
            out.push(rel.to_string_lossy().into_owned());
        }
    }
}

/// The tree to commit on top of `parent`: the vault's synced files, plus the parent's files Tend
/// doesn't manage (a licence, say). `adopted` files became notes, so they leave the tree.
fn tree_now(
    conn: &Connection,
    root: &Path,
    repo: &Repo,
    (parent, adopted): (Option<Oid>, &[String]),
) -> AppResult<Oid> {
    let paths = synced_paths(conn, root)?;
    let Some(commit) = parent else {
        return repo.tree_of_files(&paths, None);
    };
    let in_parent = managed_files(repo, commit)?;
    let mut managed: BTreeSet<String> = in_parent.keys().cloned().collect();
    managed.extend(adopted.iter().cloned());
    let unchanged = unchanged_files(conn, &in_parent)?;
    let kept = Kept {
        commit,
        managed: &managed,
        unchanged: &unchanged,
        limit: files::Limit::of(conn)?,
    };
    repo.tree_of_files(&paths, Some(kept))
}

/// The files Tend manages in a commit, with their hashes: those its `attachments` rows point at.
/// None in a commit that holds no vault.
fn managed_files(repo: &Repo, commit: Oid) -> AppResult<BTreeMap<String, String>> {
    if !holds_vault(repo, commit)? {
        return Ok(BTreeMap::new());
    }
    let rows = rows_at(repo, commit)?;
    let text = |row: &shards::Row, c: &str| row.get(c).and_then(|v| v.as_str()).map(String::from);
    let files = rows[tables::ATTACHMENTS]
        .values()
        .filter_map(|row| Some((text(row, columns::PATH)?, text(row, columns::SHA256)?)))
        .collect();
    Ok(files)
}

/// Files whose path and hash are the same here as in the parent commit: the commit reuses the
/// parent's copy instead of reading and hashing them again.
fn unchanged_files(
    conn: &Connection,
    in_parent: &BTreeMap<String, String>,
) -> AppResult<BTreeSet<String>> {
    let mut stmt = conn.prepare("SELECT path, sha256 FROM attachments WHERE missing_at IS NULL")?;
    let here = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    let mut same = BTreeSet::new();
    for row in here {
        let (path, sha) = row?;
        if in_parent.get(&path) == Some(&sha) {
            same.insert(path);
        }
    }
    Ok(same)
}

/// Markdown files on the remote that Tend doesn't manage (the README made with the repo, a file
/// added on the host's website) become notes: each folder and the file name get their slug, and a
/// taken name gets `_1`, `_2`. Returns the remote paths taken in, which leave the next commit.
fn adopt(conn: &mut Connection, root: &Path, repo: &Repo, head: Oid) -> AppResult<Vec<String>> {
    let managed: BTreeSet<String> = managed_files(repo, head)?.into_keys().collect();
    let blobs = repo.blobs(head)?;
    let mut adopted = Vec::new();
    for path in repo.foreign_files(head, &managed)? {
        let Some(place) = note_place(&path) else {
            continue;
        };
        let Some(bytes) = blobs.read(&path)? else {
            continue;
        };
        if !files::Limit::of(conn)?.allows(bytes.len() as u64) {
            continue;
        }
        // Taken in on a run whose push failed: the note is already here.
        if !has_note_like(conn, &place.folder, &bytes)? {
            let text = String::from_utf8_lossy(&bytes);
            let note = notes::create(conn, root, &place.folder, &place.name)?;
            notes::write(conn, root, &note.id, &text)?;
        }
        adopted.push(path);
    }
    Ok(adopted)
}

/// A note directly in `folder` already holds exactly these bytes.
fn has_note_like(conn: &Connection, folder: &str, bytes: &[u8]) -> AppResult<bool> {
    let sha = files::sha256_of(bytes);
    let mut stmt = conn.prepare(
        "SELECT a.path FROM attachments a JOIN notes n ON n.attachment_id = a.id
         WHERE a.sha256 = ?1 AND a.missing_at IS NULL",
    )?;
    let paths: Vec<String> = stmt
        .query_map([sha], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    Ok(paths.iter().any(|p| names::parent(p) == folder))
}

/// Where a remote Markdown file goes as a note: its folders slugged, and its name (without `.md`)
/// as the note's title, which `notes::create` slugs. `None` for other files, or a folder name
/// with no letter or digit.
struct NotePlace {
    folder: String,
    name: String,
}

fn note_place(path: &str) -> Option<NotePlace> {
    // Hidden folders (`.github/`) hold the host's own files, never notes.
    let hidden = path.split('/').any(names::is_hidden);
    if hidden {
        return None;
    }
    let path = Path::new(path);
    let is_markdown = path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(&notes::EXT[1..]));
    if !is_markdown {
        return None;
    }
    let mut folder = String::new();
    for part in path.parent()?.iter() {
        let part = names::slug(&part.to_string_lossy())?;
        folder = names::join(&folder, &part);
    }
    let name = path.file_stem()?.to_string_lossy().into_owned();
    Some(NotePlace { folder, name })
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
pub fn connect(conn: &mut Connection, root: &Path, remote: &Remote) -> AppResult<()> {
    let repo = Repo::open(root)?;
    let head = repo.fetch(remote)?;
    if let Some(head) = head {
        if holds_vault(&repo, head)? {
            return Err(errors::SYNC_REMOTE_NOT_EMPTY.err());
        }
    }
    let adopted = match head {
        Some(head) => adopt(conn, root, &repo, head)?,
        None => Vec::new(),
    };
    shards::export(conn, root)?;
    let tree = tree_now(conn, root, &repo, (head, &adopted))?;
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
    let adopted = adopt(conn, root, &repo, head)?;
    shards::export(conn, root)?;
    let tree = tree_now(conn, root, &repo, (Some(head), &adopted))?;
    let idle = head == base && tree == repo.tree_of(base)?;
    if idle {
        return Ok(Outcome::Idle);
    }
    merge_in(conn, root, &repo, (base, head))?;
    shards::export(conn, root)?;
    let tree = tree_now(conn, root, &repo, (Some(head), &adopted))?;
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
    let merged = reconcile(&Sides {
        base: &base_rows,
        local: &local_rows,
        remote: &remote_rows,
    });
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
