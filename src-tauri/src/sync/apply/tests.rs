use std::collections::HashMap;
use std::fs;

use serde_json::json;

use super::*;
use crate::db::open_memory;
use crate::notes;
use crate::sync::shards::{snapshot, Row};

/// Remote file contents, by path.
struct Remote(HashMap<String, Vec<u8>>);

impl Blobs for Remote {
    fn read(&self, path: &str) -> AppResult<Option<Vec<u8>>> {
        Ok(self.0.get(path).cloned())
    }
}

struct Vault {
    tmp: tempfile::TempDir,
    conn: Connection,
}

impl Vault {
    fn root(&self) -> &Path {
        self.tmp.path()
    }

    fn note(&mut self, folder: &str, title: &str) -> String {
        let root = self.tmp.path().to_path_buf();
        notes::create(&mut self.conn, &root, folder, title)
            .unwrap()
            .id
    }

    fn attachment_of(&self, note: &str) -> String {
        self.conn
            .query_row(
                "SELECT attachment_id FROM notes WHERE id = ?1",
                [note],
                |r| r.get(0),
            )
            .unwrap()
    }

    fn read(&self, path: &str) -> String {
        fs::read_to_string(self.root().join(path)).unwrap()
    }
}

fn vault() -> Vault {
    Vault {
        tmp: tempfile::tempdir().unwrap(),
        conn: open_memory(),
    }
}

fn set(row: &mut Row, column: &str, value: serde_json::Value) {
    row.insert(column.into(), value);
}

fn files_of(s: &mut Snapshot) -> &mut std::collections::BTreeMap<String, Row> {
    s.get_mut("attachments").unwrap()
}

fn no_tx(_: &Transaction) -> AppResult<()> {
    Ok(())
}

#[test]
fn rows_become_the_merged_rows_whatever_the_order_rules() {
    let mut v = vault();
    v.conn
        .execute_batch(
            "INSERT INTO statuses (id, value, label, color, position, is_default, updated_at)
               VALUES ('HOLD', 'HOLD', 'Hold', '#123456', 5, 0, 0);
             INSERT INTO task_folders (id, path, created_at, updated_at) VALUES ('w', 'work', 0, 0);
             INSERT INTO task_labels (id, name, color, position, created_at, updated_at) VALUES
               ('a', 'Alpha', '#123456', 0, 0, 0), ('b', 'Beta', '#123456', 1, 0, 0);
             INSERT INTO tasks (id, path, title, status, order_key, created_at, updated_at)
               VALUES ('t', 'work', 'T', 'HOLD', 'V', 0, 0);",
        )
        .unwrap();
    let local = snapshot(&v.conn).unwrap();
    let mut merged = local.clone();
    // The status and the folder go away while the task that used them moves off them...
    merged.get_mut("statuses").unwrap().remove("HOLD");
    merged.get_mut("task_folders").unwrap().remove("w");
    let task = merged.get_mut("tasks").unwrap().get_mut("t").unwrap();
    set(task, "status", json!("NOT_STARTED"));
    set(task, "path", json!(""));
    // ...and two labels swap names.
    let labels = merged.get_mut("task_labels").unwrap();
    set(labels.get_mut("a").unwrap(), "name", json!("Beta"));
    set(labels.get_mut("b").unwrap(), "name", json!("Alpha"));
    let change = Change {
        local: &local,
        merged: &merged,
        remote: &merged,
    };
    let root = v.root().to_path_buf();
    apply(&mut v.conn, &root, &change, &Remote(HashMap::new()), no_tx).unwrap();
    assert_eq!(snapshot(&v.conn).unwrap(), merged);
}

#[test]
fn files_follow_the_rows_moved_swapped_fetched_and_removed() {
    let mut v = vault();
    let plan = v.note("", "Plan");
    let idea = v.note("", "Idea");
    let gone = v.note("", "Gone");
    let local = snapshot(&v.conn).unwrap();
    let mut merged = local.clone();
    let (plan_a, idea_a, gone_a) = (
        v.attachment_of(&plan),
        v.attachment_of(&idea),
        v.attachment_of(&gone),
    );
    // Plan and Idea swap file names; Idea's text changed on the other device; Gone was deleted.
    set(
        files_of(&mut merged).get_mut(&plan_a).unwrap(),
        "path",
        json!("idea.md"),
    );
    let idea_row = files_of(&mut merged).get_mut(&idea_a).unwrap();
    set(idea_row, "path", json!("plan.md"));
    set(idea_row, "sha256", json!("remote-sha"));
    files_of(&mut merged).remove(&gone_a);
    merged.get_mut("notes").unwrap().remove(&gone);
    let remote = merged.clone();
    let blobs = Remote(HashMap::from([(
        "plan.md".to_string(),
        b"# Idea, edited".to_vec(),
    )]));
    let change = Change {
        local: &local,
        merged: &merged,
        remote: &remote,
    };
    let root = v.root().to_path_buf();
    apply(&mut v.conn, &root, &change, &blobs, no_tx).unwrap();
    assert_eq!(
        v.read("idea.md"),
        "# Plan\n",
        "Plan moved to idea.md with its own text"
    );
    assert_eq!(v.read("plan.md"), "# Idea, edited");
    assert!(!v.root().join("gone.md").exists());
    assert!(!v.root().join(".tend/staging").exists());
    let logged: i64 = v
        .conn
        .query_row(
            "SELECT count(*) FROM path_log WHERE reason = 'sync'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(logged, 2);
}

#[test]
fn a_file_the_remote_lacks_lets_its_row_in_and_leaves_the_old_file() {
    let mut v = vault();
    let plan = v.note("", "Plan");
    let local = snapshot(&v.conn).unwrap();
    let mut merged = local.clone();
    let a = v.attachment_of(&plan);
    set(
        merged.get_mut("attachments").unwrap().get_mut(&a).unwrap(),
        "sha256",
        json!("new"),
    );
    let change = Change {
        local: &local,
        merged: &merged,
        remote: &merged,
    };
    let root = v.root().to_path_buf();
    apply(&mut v.conn, &root, &change, &Remote(HashMap::new()), no_tx).unwrap();
    assert_eq!(snapshot(&v.conn).unwrap(), merged);
    assert_eq!(v.read("plan.md"), "# Plan\n");
    assert!(!v.root().join(".tend/staging").exists());
}

#[test]
fn a_folder_that_cannot_be_made_rolls_the_apply_back() {
    let mut v = vault();
    let local = snapshot(&v.conn).unwrap();
    let mut merged = local.clone();
    // A plain file sits where the incoming folder must go.
    fs::write(v.root().join("ideas"), "not a folder").unwrap();
    merged.get_mut("note_folders").unwrap().insert(
        "f1".into(),
        json!({ "id": "f1", "path": "ideas", "created_at": 0, "updated_at": 0 })
            .as_object()
            .unwrap()
            .clone(),
    );
    let change = Change {
        local: &local,
        merged: &merged,
        remote: &merged,
    };
    let root = v.root().to_path_buf();
    assert!(apply(&mut v.conn, &root, &change, &Remote(HashMap::new()), no_tx).is_err());
    assert_eq!(snapshot(&v.conn).unwrap(), local);
}

#[test]
fn a_path_outside_the_vault_is_refused() {
    for bad in ["../x.md", "/tmp/x.md", "a/../../x.md", ".tend/tend.db"] {
        assert!(files::check_path(bad).is_err(), "{bad}");
    }
    for ok in ["", "work/plan.md", ".tend/attachments/01H.pdf"] {
        assert!(files::check_path(ok).is_ok(), "{ok}");
    }
}

#[test]
fn work_done_inside_the_transaction_commits_with_the_rows() {
    let mut v = vault();
    let local = snapshot(&v.conn).unwrap();
    let change = Change {
        local: &local,
        merged: &local,
        remote: &local,
    };
    let root = v.root().to_path_buf();
    let mark = |tx: &Transaction| -> AppResult<()> {
        tx.execute("UPDATE sync_state SET last_synced_commit = 'abc'", [])?;
        Ok(())
    };
    apply(&mut v.conn, &root, &change, &Remote(HashMap::new()), mark).unwrap();
    let base: String = v
        .conn
        .query_row("SELECT last_synced_commit FROM sync_state", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(base, "abc");
}

#[test]
fn new_tasks_are_written_once() {
    let mut v = vault();
    let local = snapshot(&v.conn).unwrap();
    let mut merged = local.clone();
    let task = json!({
        "id": "01HZZZZZZZZZZZZZZZZZZZZZZZ", "path": "", "parent_id": null, "title": "New",
        "description": "", "status": "NOT_STARTED", "due_date": null, "due_time": null,
        "assignee_id": null, "order_key": "V", "created_at": 1, "updated_at": 1, "completed_at": null,
    });
    merged.get_mut("tasks").unwrap().insert(
        task["id"].as_str().unwrap().into(),
        task.as_object().unwrap().clone(),
    );
    let change = Change {
        local: &local,
        merged: &merged,
        remote: &merged,
    };
    let root = v.root().to_path_buf();
    apply(&mut v.conn, &root, &change, &Remote(HashMap::new()), no_tx).unwrap();
    assert_eq!(snapshot(&v.conn).unwrap(), merged);
}
