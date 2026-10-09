//! The Git boundary (spec §6): everything sync asks of Git, on top of libgit2 (`git2`). Tend never
//! uses Git's merge: it fetches, reads trees, writes its own merged tree, commits it on top of the
//! fetched head and pushes. The commit holds only what Tend syncs: the snapshot shards, the note
//! files, the attached files and the `.gitignore`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Once;

use git2::{
    Cred, ErrorClass, ErrorCode, FetchOptions, IndexAddOption, ObjectType, Oid, PushOptions,
    RemoteCallbacks, Repository, Signature, TreeWalkMode, TreeWalkResult,
};

use super::apply::Blobs;
use crate::error::{errors, AppError, AppResult};
use crate::vault::TEND_DIR;

/// How long a connection may take, and how long a network read may stall.
const CONNECT_TIMEOUT_MS: i32 = 15_000;
const IO_TIMEOUT_MS: i32 = 60_000;
/// Credential prompts per operation before giving up (a wrong token would otherwise loop).
const MAX_AUTH_TRIES: u32 = 3;
const REMOTE_NAME: &str = "origin";
const GITIGNORE: &str = ".gitignore";
/// What the vault's Git never tracks: the live database and the apply's staging area.
const IGNORED: &str = ".tend/tend.db*\n.tend/staging/\n.DS_Store\n";

/// Where and how to sync.
pub struct Remote {
    pub url: String,
    pub branch: String,
    pub username: Option<String>,
    pub token: Option<String>,
    pub author_name: String,
    pub author_email: String,
}

impl From<git2::Error> for AppError {
    fn from(e: git2::Error) -> Self {
        let key = match (e.class(), e.code()) {
            _ if is_auth(&e) => errors::SYNC_AUTH,
            (ErrorClass::Net | ErrorClass::Ssl, _) => errors::SYNC_OFFLINE,
            (_, ErrorCode::NotFastForward) => errors::SYNC_REJECTED,
            _ => errors::SYNC_GIT,
        };
        key.with(e.message().to_string())
    }
}

fn is_auth(e: &git2::Error) -> bool {
    let message = e.message().to_lowercase();
    e.code() == ErrorCode::Auth || message.contains("401") || message.contains("403")
}

/// Network deadlines for libgit2, set once per process.
fn set_timeouts() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        // Safety: both only write libgit2's global options, before any network use.
        unsafe {
            let _ = git2::opts::set_server_connect_timeout_in_milliseconds(CONNECT_TIMEOUT_MS);
            let _ = git2::opts::set_server_timeout_in_milliseconds(IO_TIMEOUT_MS);
        }
    });
}

fn callbacks(remote: &Remote) -> RemoteCallbacks<'_> {
    let mut tries = 0;
    let mut cb = RemoteCallbacks::new();
    cb.credentials(move |_url, _user, _allowed| {
        tries += 1;
        let (Some(token), true) = (&remote.token, tries <= MAX_AUTH_TRIES) else {
            return Err(git2::Error::from_str("authentication failed"));
        };
        let username = remote.username.as_deref().unwrap_or("x-access-token");
        Cred::userpass_plaintext(username, token)
    });
    cb
}

/// A commit whose foreign files carry over, and the files Tend manages in it (the paths its
/// `attachments` rows point at).
pub struct Kept<'a> {
    pub commit: Oid,
    pub managed: &'a BTreeSet<String>,
    /// Managed files the vault still holds unchanged: the parent's entry is reused, unread.
    pub unchanged: &'a BTreeSet<String>,
}

/// Tend's own files: `.gitignore`, everything under `.tend/`, and files rows point at.
fn is_managed(path: &Path, rows: &BTreeSet<String>) -> bool {
    let own = path == Path::new(GITIGNORE) || path.starts_with(TEND_DIR);
    own || rows.contains(&*path.to_string_lossy())
}

pub struct Repo {
    repo: Repository,
}

impl Repo {
    /// The vault's repository, made on first use with Tend's `.gitignore`.
    pub fn open(root: &Path) -> AppResult<Repo> {
        set_timeouts();
        let repo = match Repository::open(root) {
            Ok(repo) => repo,
            Err(_) => Repository::init(root)?,
        };
        let ignore = root.join(GITIGNORE);
        if !ignore.exists() {
            std::fs::write(ignore, IGNORED)?;
        }
        Ok(Repo { repo })
    }

    fn remote_ref(branch: &str) -> String {
        format!("refs/remotes/{REMOTE_NAME}/{branch}")
    }

    /// Fetches the branch. Returns its head, or `None` when the remote has no such branch.
    pub fn fetch(&self, remote: &Remote) -> AppResult<Option<Oid>> {
        let mut origin = self.repo.remote_anonymous(&remote.url)?;
        let mut options = FetchOptions::new();
        options.remote_callbacks(callbacks(remote));
        let spec = format!(
            "+refs/heads/{}:{}",
            remote.branch,
            Self::remote_ref(&remote.branch)
        );
        origin.fetch(&[spec], Some(&mut options), None)?;
        let head = self.repo.refname_to_id(&Self::remote_ref(&remote.branch));
        match head {
            Ok(oid) => Ok(Some(oid)),
            Err(e) if e.code() == ErrorCode::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn has(&self, commit: Oid) -> bool {
        self.repo.find_commit(commit).is_ok()
    }

    /// `ancestor` is `commit` or in its history.
    pub fn contains(&self, commit: Oid, ancestor: Oid) -> AppResult<bool> {
        if commit == ancestor {
            return Ok(true);
        }
        Ok(self.repo.graph_descendant_of(commit, ancestor)?)
    }

    pub fn tree_of(&self, commit: Oid) -> AppResult<Oid> {
        Ok(self.repo.find_commit(commit)?.tree_id())
    }

    /// The text files under `dir` in a commit, keyed by their path below `dir`.
    pub fn files_under(&self, commit: Oid, dir: &str) -> AppResult<BTreeMap<PathBuf, String>> {
        let tree = self.repo.find_commit(commit)?.tree()?;
        let mut out = BTreeMap::new();
        let Ok(entry) = tree.get_path(Path::new(dir)) else {
            return Ok(out);
        };
        let sub = self.repo.find_tree(entry.id())?;
        let mut failed = None;
        sub.walk(TreeWalkMode::PreOrder, |parent, item| {
            if item.kind() != Some(ObjectType::Blob) {
                return TreeWalkResult::Ok;
            }
            let path = PathBuf::from(parent).join(item.name().unwrap_or_default());
            match self.repo.find_blob(item.id()) {
                Ok(blob) => {
                    out.insert(path, String::from_utf8_lossy(blob.content()).into_owned());
                    TreeWalkResult::Ok
                }
                Err(e) => {
                    failed = Some(e);
                    TreeWalkResult::Abort
                }
            }
        })?;
        match failed {
            Some(e) => Err(e.into()),
            None => Ok(out),
        }
    }

    /// The tree of the vault's synced files as they are on disk now (written to Git's object store,
    /// not yet committed). Files of `keep` that Tend doesn't manage (a README, say) stay in it.
    pub fn tree_of_files(&self, paths: &[String], keep: Option<Kept>) -> AppResult<Oid> {
        let mut index = self.repo.index()?;
        index.clear()?;
        if let Some(kept) = &keep {
            self.read_foreign(&mut index, kept)?;
        }
        let reused = keep.as_ref().map(|k| k.unchanged);
        index.add_all([GITIGNORE], IndexAddOption::FORCE, None)?;
        for path in paths {
            if reused.is_some_and(|r| r.contains(path)) {
                continue;
            }
            index.add_path(Path::new(path))?;
        }
        Ok(index.write_tree()?)
    }

    /// The commit's files Tend doesn't manage (see `is_managed`).
    pub fn foreign_files(&self, commit: Oid, managed: &BTreeSet<String>) -> AppResult<Vec<String>> {
        let tree = self.repo.find_commit(commit)?.tree()?;
        let mut out = Vec::new();
        tree.walk(TreeWalkMode::PreOrder, |parent, item| {
            if item.kind() == Some(ObjectType::Blob) {
                let path = PathBuf::from(parent).join(item.name().unwrap_or_default());
                if !is_managed(&path, managed) {
                    out.push(path.to_string_lossy().into_owned());
                }
            }
            TreeWalkResult::Ok
        })?;
        Ok(out)
    }

    /// Loads the commit's tree into `index`, minus every file Tend manages there (unless it is
    /// unchanged) and every file too big to keep (`files::MAX_FILE_BYTES`).
    fn read_foreign(&self, index: &mut git2::Index, kept: &Kept) -> AppResult<()> {
        let tree = self.repo.find_commit(kept.commit)?.tree()?;
        index.read_tree(&tree)?;
        let odb = self.repo.odb()?;
        let dropped: Vec<PathBuf> = index
            .iter()
            .filter_map(|entry| {
                let path = String::from_utf8_lossy(&entry.path).into_owned();
                // The object's header holds its size; the content isn't read.
                let size = odb.read_header(entry.id).map_or(0, |(size, _)| size as u64);
                let reused = kept.unchanged.contains(&path) && crate::files::within_limit(size);
                let stale = is_managed(Path::new(&path), kept.managed) && !reused;
                let too_big = !crate::files::within_limit(size);
                (stale || too_big).then(|| PathBuf::from(path))
            })
            .collect();
        for path in dropped {
            index.remove_path(&path)?;
        }
        Ok(())
    }

    /// Commits `tree` with `parent` (none for the first commit) and points the local branch at it.
    pub fn commit(
        &self,
        remote: &Remote,
        tree: Oid,
        parent: Option<Oid>,
        message: &str,
    ) -> AppResult<Oid> {
        let tree = self.repo.find_tree(tree)?;
        let who = Signature::now(&remote.author_name, &remote.author_email)?;
        let parents: Vec<git2::Commit> = match parent {
            Some(p) => vec![self.repo.find_commit(p)?],
            None => Vec::new(),
        };
        let parents: Vec<&git2::Commit> = parents.iter().collect();
        let commit = self
            .repo
            .commit(None, &who, &who, message, &tree, &parents)?;
        self.point_branch(remote, commit)?;
        Ok(commit)
    }

    /// Pushes the local branch. A rejection (someone pushed first) is `SYNC_REJECTED`.
    pub fn push(&self, remote: &Remote) -> AppResult<()> {
        let mut origin = self.repo.remote_anonymous(&remote.url)?;
        let mut rejected = None;
        {
            let mut cb = callbacks(remote);
            cb.push_update_reference(|_name, status| {
                rejected = status.map(String::from);
                Ok(())
            });
            let mut options = PushOptions::new();
            options.remote_callbacks(cb);
            let spec = format!("refs/heads/{0}:refs/heads/{0}", remote.branch);
            origin.push(&[spec], Some(&mut options))?;
        }
        match rejected {
            Some(reason) => Err(errors::SYNC_REJECTED.with(reason)),
            None => Ok(()),
        }
    }

    /// Points the local branch at `commit` (after a restore, or a run with nothing to push).
    pub fn point_branch(&self, remote: &Remote, commit: Oid) -> AppResult<()> {
        let branch = format!("refs/heads/{}", remote.branch);
        self.repo.reference(&branch, commit, true, "tend sync")?;
        self.repo.set_head(&branch)?;
        Ok(())
    }

    /// File contents at a commit, for applying (see `apply::Blobs`).
    pub fn blobs(&self, commit: Oid) -> AppResult<CommitBlobs<'_>> {
        let tree = self.repo.find_commit(commit)?.tree()?;
        Ok(CommitBlobs {
            repo: &self.repo,
            tree,
        })
    }
}

pub struct CommitBlobs<'r> {
    repo: &'r Repository,
    tree: git2::Tree<'r>,
}

impl Blobs for CommitBlobs<'_> {
    fn read(&self, path: &str) -> AppResult<Option<Vec<u8>>> {
        let Ok(entry) = self.tree.get_path(Path::new(path)) else {
            return Ok(None);
        };
        let blob = self.repo.find_blob(entry.id())?;
        Ok(Some(blob.content().to_vec()))
    }
}
