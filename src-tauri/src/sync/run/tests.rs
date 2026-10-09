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
    connect(&a.vault.db(), a.root(), &remote).unwrap();

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
    tasks::update(&mut b.vault.db(), &task, &patch).unwrap();
    a.sync(&remote).unwrap();
    b.sync(&remote).unwrap();
    a.sync(&remote).unwrap();
    assert!(!b.rows()["tasks"].contains_key(&task));
    assert_eq!(a.rows(), b.rows());
}

#[test]
fn a_remote_with_a_vault_is_refused_on_connect() {
    let server = tempfile::tempdir().unwrap();
    let remote = remote(server.path());
    let a = Device::new();
    connect(&a.vault.db(), a.root(), &remote).unwrap();
    let c = Device::new();
    let refused = connect(&c.vault.db(), c.root(), &remote).unwrap_err();
    assert!(refused.is(errors::SYNC_REMOTE_NOT_EMPTY));
}

#[test]
fn a_rewritten_remote_stops_sync_and_asks_for_a_restore() {
    let server = tempfile::tempdir().unwrap();
    let remote = remote(server.path());
    let a = Device::new();
    connect(&a.vault.db(), a.root(), &remote).unwrap();
    // Someone replaces the branch with unrelated history.
    let other = Device::new();
    let bare = git2::Repository::open_bare(server.path()).unwrap();
    bare.find_reference("refs/heads/main")
        .unwrap()
        .delete()
        .unwrap();
    connect(&other.vault.db(), other.root(), &remote).unwrap();
    let stopped = a.sync(&remote).unwrap_err();
    assert!(stopped.is(errors::SYNC_BASE_LOST));
}

#[test]
fn a_push_that_landed_before_a_crash_becomes_the_base() {
    let server = tempfile::tempdir().unwrap();
    let remote = remote(server.path());
    let a = Device::new();
    connect(&a.vault.db(), a.root(), &remote).unwrap();
    let id = "01HZZZZZZZZZZZZZZZZZZZZZZZ";
    person(&a, id);
    assert_eq!(a.sync(&remote).unwrap(), Outcome::Pushed);
    // Pretend the run crashed after the push: the base is still the old commit, the new one pending.
    let pushed = state::load(&a.vault.db()).unwrap().base.unwrap();
    let repo = git2::Repository::open(a.root()).unwrap();
    let old = repo
        .find_commit(oid(&pushed).unwrap())
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
    assert!(!a.rows()["people"].contains_key(id));
}
