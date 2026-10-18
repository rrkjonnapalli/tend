use std::fs;

use super::*;
use crate::notes;
use crate::state::Vault;
use crate::tasks::{self, NewTask};

/// A device: a vault folder with its database.
struct Device {
    _tmp: tempfile::TempDir,
    vault: Vault,
}

impl Device {
    fn new() -> Device {
        let tmp = tempfile::tempdir().unwrap();
        let (_, vault) = crate::vault::open(tmp.path(), false);
        Device {
            _tmp: tmp,
            vault: vault.unwrap(),
        }
    }

    fn root(&self) -> &Path {
        &self.vault.root
    }

    fn rows(&self) -> Snapshot {
        shards::snapshot(&self.vault.db()).unwrap()
    }

    fn sync(&self, remote: &Remote) -> AppResult<Outcome> {
        let root = self.root().to_path_buf();
        sync(&mut self.vault.db(), &root, remote)
    }

    fn read(&self, path: &str) -> String {
        fs::read_to_string(self.root().join(path)).unwrap()
    }

    fn task(&self, title: &str) -> String {
        let new = NewTask {
            title: title.into(),
            ..Default::default()
        };
        tasks::create(&mut self.vault.db(), &new).unwrap().id
    }
}

fn remote(dir: &Path) -> Remote {
    git2::Repository::init_bare(dir).unwrap();
    Remote {
        url: dir.to_string_lossy().into_owned(),
        branch: "main".into(),
        username: None,
        token: None,
        author_name: "Tend".into(),
        author_email: "tend@localhost".into(),
    }
}

fn person(device: &Device, id: &str) {
    device
        .vault
        .db()
        .execute(
            "INSERT INTO people (id, first_name, created_at, updated_at) VALUES (?1, 'Ann', 0, 0)",
            [id],
        )
        .unwrap();
}

#[test]
fn two_devices_connect_restore_and_converge() {
    let server = tempfile::tempdir().unwrap();
    let remote = remote(server.path());
    let a = Device::new();
    let plan = notes::create(&mut a.vault.db(), &a.vault.notes_dir(), "work", "Plan").unwrap();
    let task = a.task("Ship");
    connect(&mut a.vault.db(), a.root(), &remote).unwrap();

    let b = Device::new();
    let root = b.root().to_path_buf();
    restore(&mut b.vault.db(), &root, &remote).unwrap();
    assert_eq!(b.rows(), a.rows(), "a restore brings every row");
    assert_eq!(
        fs::read_to_string(b.root().join("work/plan.md")).unwrap(),
        "# Plan\n"
    );

    // Both change something; each sync brings the other's change.
    notes::write(
        &a.vault.db(),
        &a.vault.notes_dir(),
        &plan.id,
        "# Plan\n\nship it",
    )
    .unwrap();
    person(&b, "01HZZZZZZZZZZZZZZZZZZZZZZZ");
    assert_eq!(a.sync(&remote).unwrap(), Outcome::Pushed);
    assert_eq!(
        b.sync(&remote).unwrap(),
        Outcome::Pushed,
        "B pushes its person after taking A's edit"
    );
    assert_eq!(a.sync(&remote).unwrap(), Outcome::Pulled);
    assert_eq!(a.rows(), b.rows());
    assert_eq!(
        fs::read_to_string(b.root().join("work/plan.md")).unwrap(),
        "# Plan\n\nship it"
    );

    // Nothing new: both runs are idle.
    assert_eq!(a.sync(&remote).unwrap(), Outcome::Idle);
    assert_eq!(b.sync(&remote).unwrap(), Outcome::Idle);

    // A deletes the task while B edits it: the delete wins.
    tasks::delete(&a.vault.db(), &task).unwrap();
    let patch = tasks::Patch {
        title: Some("Ship v2".into()),
        ..Default::default()
    };
    tasks::update(&b.vault.db(), &task, &patch).unwrap();
    a.sync(&remote).unwrap();
    b.sync(&remote).unwrap();
    a.sync(&remote).unwrap();
    assert!(!b.rows()[tables::TASKS].contains_key(&task));
    assert_eq!(a.rows(), b.rows());
}

/// Every row's id in a table.
fn ids(device: &Device, table: &str) -> Vec<String> {
    device.rows()[table].keys().cloned().collect()
}

#[test]
fn a_remote_with_a_vault_is_merged_on_connect() {
    let server = tempfile::tempdir().unwrap();
    let remote = remote(server.path());
    let a = Device::new();
    let ship = a.task("Ship");
    connect(&mut a.vault.db(), a.root(), &remote).unwrap();

    let c = Device::new();
    let write = c.task("Write");
    connect(&mut c.vault.db(), c.root(), &remote).unwrap();
    assert_eq!(a.sync(&remote).unwrap(), Outcome::Pulled);
    for device in [&a, &c] {
        let tasks = ids(device, tables::TASKS);
        assert!(tasks.contains(&ship) && tasks.contains(&write), "{tasks:?}");
    }
    assert_eq!(a.rows(), c.rows());
}

#[test]
fn two_vaults_with_the_same_note_path_keep_both_notes_on_connect() {
    let server = tempfile::tempdir().unwrap();
    let remote = remote(server.path());
    let a = Device::new();
    let ours = notes::create(&mut a.vault.db(), &a.vault.notes_dir(), "", "Plan").unwrap();
    notes::write(
        &a.vault.db(),
        &a.vault.notes_dir(),
        &ours.id,
        "# Plan\n\nours",
    )
    .unwrap();
    connect(&mut a.vault.db(), a.root(), &remote).unwrap();

    let c = Device::new();
    let theirs = notes::create(&mut c.vault.db(), &c.vault.notes_dir(), "", "Plan").unwrap();
    notes::write(
        &c.vault.db(),
        &c.vault.notes_dir(),
        &theirs.id,
        "# Plan\n\ntheirs",
    )
    .unwrap();
    connect(&mut c.vault.db(), c.root(), &remote).unwrap();
    a.sync(&remote).unwrap();

    for device in [&a, &c] {
        let texts = [device.read("plan.md"), device.read("plan_1.md")];
        assert!(texts.contains(&"# Plan\n\nours".to_string()), "{texts:?}");
        assert!(texts.contains(&"# Plan\n\ntheirs".to_string()), "{texts:?}");
    }
    assert_eq!(a.rows(), c.rows());
}

/// Replaces the remote's branch with another vault's unrelated history.
fn rewrite_remote(server: &Path, remote: &Remote) -> Device {
    let bare = git2::Repository::open_bare(server).unwrap();
    bare.find_reference("refs/heads/main")
        .unwrap()
        .delete()
        .unwrap();
    let other = Device::new();
    other.task("Theirs");
    connect(&mut other.vault.db(), other.root(), remote).unwrap();
    other
}

#[test]
fn a_rewritten_remote_merges_on_an_empty_base_and_keeps_both_sides() {
    let server = tempfile::tempdir().unwrap();
    let remote = remote(server.path());
    let a = Device::new();
    connect(&mut a.vault.db(), a.root(), &remote).unwrap();
    let other = rewrite_remote(server.path(), &remote);
    let mine = a.task("Mine, made while the remote changed");

    assert_eq!(a.sync(&remote).unwrap(), Outcome::Pushed);
    other.sync(&remote).unwrap();
    assert!(ids(&other, tables::TASKS).contains(&mine));
    assert_eq!(a.rows(), other.rows());
    assert_eq!(
        a.sync(&remote).unwrap(),
        Outcome::Idle,
        "the merge became the base"
    );
}

#[test]
fn a_deleted_branch_gets_the_vault_pushed_again() {
    let server = tempfile::tempdir().unwrap();
    let remote = remote(server.path());
    let a = Device::new();
    connect(&mut a.vault.db(), a.root(), &remote).unwrap();
    let bare = git2::Repository::open_bare(server.path()).unwrap();
    bare.find_reference("refs/heads/main")
        .unwrap()
        .delete()
        .unwrap();
    a.task("Ship");

    assert_eq!(a.sync(&remote).unwrap(), Outcome::Pushed);
    let b = Device::new();
    let root = b.root().to_path_buf();
    restore(&mut b.vault.db(), &root, &remote).unwrap();
    assert_eq!(b.rows(), a.rows());
}

#[test]
fn a_push_that_landed_before_a_crash_becomes_the_base() {
    let server = tempfile::tempdir().unwrap();
    let remote = remote(server.path());
    let a = Device::new();
    connect(&mut a.vault.db(), a.root(), &remote).unwrap();
    let id = "01HZZZZZZZZZZZZZZZZZZZZZZZ";
    person(&a, id);
    assert_eq!(a.sync(&remote).unwrap(), Outcome::Pushed);
    // Pretend the run crashed after the push: the base is still the old commit, the new one pending.
    let pushed = state::load(&a.vault.db()).unwrap().base.unwrap();
    let repo = git2::Repository::open(a.root()).unwrap();
    let old = repo
        .find_commit(parse(&pushed).unwrap())
        .unwrap()
        .parent_id(0)
        .unwrap();
    state::set_base(&a.vault.db(), &old.to_string()).unwrap();
    state::set_pending(&a.vault.db(), Some(&pushed)).unwrap();
    // Deleting the person now must stay a delete, not look like the remote adding it.
    a.vault
        .db()
        .execute("DELETE FROM people WHERE id = ?1", [id])
        .unwrap();
    a.sync(&remote).unwrap();
    assert!(!a.rows()[tables::PEOPLE].contains_key(id));
}

/// Makes the remote's first commit hold `files`, the way a host's "new repo" page (or its web
/// editor) does.
fn host_commit(server: &Path, files: &[(&str, &str)]) {
    let bare = git2::Repository::open_bare(server).unwrap();
    let mut index = git2::Index::new().unwrap();
    for (path, text) in files {
        let blob = bare.blob(text.as_bytes()).unwrap();
        let entry = git2::IndexEntry {
            ctime: git2::IndexTime::new(0, 0),
            mtime: git2::IndexTime::new(0, 0),
            dev: 0,
            ino: 0,
            mode: 0o100644,
            uid: 0,
            gid: 0,
            file_size: text.len() as u32,
            id: blob,
            flags: 0,
            flags_extended: 0,
            path: path.as_bytes().to_vec(),
        };
        index.add(&entry).unwrap();
    }
    let tree = bare.find_tree(index.write_tree_to(&bare).unwrap()).unwrap();
    let who = git2::Signature::now("Host", "host@example.com").unwrap();
    bare.commit(
        Some("refs/heads/main"),
        &who,
        &who,
        "Initial commit",
        &tree,
        &[],
    )
    .unwrap();
}

fn remote_files(server: &Path) -> Vec<String> {
    let bare = git2::Repository::open_bare(server).unwrap();
    let head = bare.refname_to_id("refs/heads/main").unwrap();
    let tree = bare.find_commit(head).unwrap().tree().unwrap();
    let mut out = Vec::new();
    tree.walk(git2::TreeWalkMode::PreOrder, |parent, item| {
        let is_file = item.kind() == Some(git2::ObjectType::Blob);
        let path = format!("{parent}{}", item.name().unwrap());
        if is_file && !path.starts_with(".tend/") {
            out.push(path);
        }
        git2::TreeWalkResult::Ok
    })
    .unwrap();
    out.sort();
    out
}

#[test]
fn markdown_made_on_the_host_becomes_notes_and_other_files_stay() {
    let server = tempfile::tempdir().unwrap();
    let remote = remote(server.path());
    host_commit(
        server.path(),
        &[
            ("README.md", "# notes\n"),
            ("Docs/Guide.md", "# Guide\n"),
            ("LICENSE", "MIT"),
        ],
    );
    let a = Device::new();
    notes::create(&mut a.vault.db(), &a.vault.notes_dir(), "", "Readme").unwrap();
    connect(&mut a.vault.db(), a.root(), &remote).unwrap();

    // The local readme keeps its name; the remote one is taken in next to it.
    assert_eq!(a.read("readme.md"), "# Readme\n");
    assert_eq!(a.read("readme_1.md"), "# notes\n");
    assert_eq!(a.read("docs/guide.md"), "# Guide\n");
    assert_eq!(
        remote_files(server.path()),
        [
            ".gitignore",
            "LICENSE",
            "docs/guide.md",
            "readme.md",
            "readme_1.md"
        ]
    );
    assert!(
        !a.root().join("LICENSE").exists(),
        "Tend writes only its own files"
    );

    let b = Device::new();
    let root = b.root().to_path_buf();
    restore(&mut b.vault.db(), &root, &remote).unwrap();
    assert_eq!(b.rows(), a.rows());
    person(&b, "01HZZZZZZZZZZZZZZZZZZZZZZZ");
    assert_eq!(b.sync(&remote).unwrap(), Outcome::Pushed);
    assert!(remote_files(server.path()).contains(&"LICENSE".to_string()));
    assert_eq!(a.sync(&remote).unwrap(), Outcome::Pulled);
    assert_eq!(a.sync(&remote).unwrap(), Outcome::Idle);
}

#[test]
fn a_markdown_file_added_on_the_host_later_is_taken_in_on_the_next_sync() {
    let server = tempfile::tempdir().unwrap();
    let remote = remote(server.path());
    let a = Device::new();
    connect(&mut a.vault.db(), a.root(), &remote).unwrap();
    // Someone adds a file through the host's web editor, on top of Tend's commit.
    let bare = git2::Repository::open_bare(server.path()).unwrap();
    let head = bare
        .find_commit(bare.refname_to_id("refs/heads/main").unwrap())
        .unwrap();
    let mut tree = bare.treebuilder(Some(&head.tree().unwrap())).unwrap();
    tree.insert("Todo.md", bare.blob(b"- milk").unwrap(), 0o100644)
        .unwrap();
    let tree = bare.find_tree(tree.write().unwrap()).unwrap();
    let who = git2::Signature::now("Host", "host@example.com").unwrap();
    bare.commit(
        Some("refs/heads/main"),
        &who,
        &who,
        "Add Todo",
        &tree,
        &[&head],
    )
    .unwrap();

    assert_eq!(a.sync(&remote).unwrap(), Outcome::Pushed);
    assert_eq!(a.read("todo.md"), "- milk");
    assert!(remote_files(server.path()).contains(&"todo.md".to_string()));
    assert!(!remote_files(server.path()).contains(&"Todo.md".to_string()));
}

#[test]
fn a_file_taken_in_before_a_failed_push_is_not_taken_in_twice() {
    let server = tempfile::tempdir().unwrap();
    let remote = remote(server.path());
    host_commit(server.path(), &[("README.md", "# notes\n")]);
    let a = Device::new();
    // An earlier connect took the README in, then its push failed.
    let repo = Repo::open(a.root()).unwrap();
    let head = repo.fetch(&remote).unwrap().unwrap();
    adopt(&mut a.vault.db(), a.root(), &repo, head).unwrap();
    assert!(a.root().join("readme.md").is_file());
    connect(&mut a.vault.db(), a.root(), &remote).unwrap();
    assert!(!a.root().join("readme_1.md").exists());
    assert_eq!(remote_files(server.path()), [".gitignore", "readme.md"]);
}

#[test]
fn hidden_and_oversized_files_on_the_host_are_never_taken_in_and_big_ones_leave_the_repo() {
    let server = tempfile::tempdir().unwrap();
    let remote = remote(server.path());
    let big = "x".repeat(crate::files::MAX_FILE_BYTES as usize);
    host_commit(
        server.path(),
        &[
            (".github/ISSUE_TEMPLATE/bug.md", "# Bug\n"),
            ("huge.md", big.as_str()),
        ],
    );
    let a = Device::new();
    connect(&mut a.vault.db(), a.root(), &remote).unwrap();
    assert!(
        !a.root().join("github").exists(),
        "a hidden folder's Markdown is not a note"
    );
    assert!(!a.root().join("huge.md").exists());
    assert_eq!(
        remote_files(server.path()),
        [".github/ISSUE_TEMPLATE/bug.md", ".gitignore"]
    );
}
