use super::*;
use serde_json::json;

fn row(v: Value) -> Row {
    v.as_object().unwrap().clone()
}

/// A snapshot from `(table, rows)` pairs; each row needs an `id`.
fn snap(tables: &[(&'static str, Vec<Value>)]) -> Snapshot {
    let mut out = Snapshot::new();
    for (table, rows) in tables {
        let rows = rows
            .iter()
            .map(|v| (v["id"].as_str().unwrap().to_string(), row(v.clone())))
            .collect();
        out.insert(table, rows);
    }
    out
}

fn person(id: &str, name: &str, at: i64) -> Value {
    json!({ "id": id, "first_name": name, "updated_at": at })
}

fn names_of(s: &Snapshot) -> Vec<String> {
    s["people"]
        .values()
        .map(|r| text(r, "first_name").unwrap().to_string())
        .collect()
}

#[test]
fn a_change_on_one_side_wins_without_looking_at_the_clock() {
    let base = snap(&[("people", vec![person("p", "Ann", 10)])]);
    let local = snap(&[("people", vec![person("p", "Ann", 10)])]);
    let remote = snap(&[("people", vec![person("p", "Anna", 1)])]);
    assert_eq!(
        names_of(&reconcile(&Sides {
            base: &base,
            local: &local,
            remote: &remote
        })),
        vec!["Anna"]
    );
    assert_eq!(
        names_of(&reconcile(&Sides {
            base: &base,
            local: &remote,
            remote: &local
        })),
        vec!["Anna"]
    );
}

#[test]
fn both_changed_newer_wins_and_ties_are_the_same_everywhere() {
    let base = snap(&[("people", vec![person("p", "Ann", 1)])]);
    let a = snap(&[("people", vec![person("p", "Anna", 5)])]);
    let b = snap(&[("people", vec![person("p", "Annie", 9)])]);
    assert_eq!(
        names_of(&reconcile(&Sides {
            base: &base,
            local: &a,
            remote: &b
        })),
        vec!["Annie"]
    );
    let tie_a = snap(&[("people", vec![person("p", "Anna", 5)])]);
    let tie_b = snap(&[("people", vec![person("p", "Annie", 5)])]);
    assert_eq!(
        reconcile(&Sides {
            base: &base,
            local: &tie_a,
            remote: &tie_b
        }),
        reconcile(&Sides {
            base: &base,
            local: &tie_b,
            remote: &tie_a
        }),
        "a tie resolves the same way on both devices"
    );
}

#[test]
fn a_delete_beats_an_edit_and_adds_are_kept() {
    let base = snap(&[("people", vec![person("p", "Ann", 1)])]);
    let local = snap(&[("people", vec![person("n", "New", 3)])]);
    let remote = snap(&[(
        "people",
        vec![person("p", "Edited", 9), person("r", "Remote", 2)],
    )]);
    let merged = reconcile(&Sides {
        base: &base,
        local: &local,
        remote: &remote,
    });
    let ids: Vec<&String> = merged["people"].keys().collect();
    assert_eq!(ids, vec!["n", "r"]);
}

#[test]
fn colliding_links_and_folders_collapse_to_the_smallest_id() {
    let tables = [
        (
            "tasks",
            vec![json!({ "id": "t", "path": "", "status": "NOT_STARTED", "updated_at": 0 })],
        ),
        ("tags", vec![json!({ "id": "ops", "updated_at": 0 })]),
        (
            "statuses",
            vec![
                json!({ "id": "NOT_STARTED", "value": "NOT_STARTED", "label": "Not started", "position": 0, "updated_at": 0 }),
            ],
        ),
        (
            "task_folders",
            vec![json!({ "id": "root", "path": "", "updated_at": 0 })],
        ),
    ];
    let base = snap(&tables);
    let mut l = snap(&[
        (
            "task_tags",
            vec![json!({ "id": "b", "task_id": "t", "tag_id": "ops", "updated_at": 1 })],
        ),
        (
            "task_folders",
            vec![json!({ "id": "f2", "path": "work", "updated_at": 1 })],
        ),
    ]);
    let mut r = snap(&[
        (
            "task_tags",
            vec![json!({ "id": "a", "task_id": "t", "tag_id": "ops", "updated_at": 1 })],
        ),
        (
            "task_folders",
            vec![json!({ "id": "f1", "path": "work", "updated_at": 1 })],
        ),
    ]);
    for (table, rows) in &base {
        l.entry(table).or_default().extend(rows.clone());
        r.entry(table).or_default().extend(rows.clone());
    }
    let merged = reconcile(&Sides {
        base: &base,
        local: &l,
        remote: &r,
    });
    assert_eq!(merged["task_tags"].keys().collect::<Vec<_>>(), vec!["a"]);
    let folders: Vec<&String> = merged["task_folders"].keys().collect();
    assert_eq!(folders, vec!["f1", "root"]);
}

#[test]
fn equal_label_names_get_numbered_and_clashing_note_paths_a_suffix() {
    let empty = Snapshot::new();
    let l = snap(&[
        (
            "task_labels",
            vec![json!({ "id": "x", "name": "Review", "updated_at": 1 })],
        ),
        (
            "attachments",
            vec![json!({ "id": "a1", "path": "plan.md", "name": "plan.md", "updated_at": 1 })],
        ),
    ]);
    let r = snap(&[
        (
            "task_labels",
            vec![json!({ "id": "y", "name": "review", "updated_at": 1 })],
        ),
        (
            "attachments",
            vec![json!({ "id": "a2", "path": "plan.md", "name": "plan.md", "updated_at": 1 })],
        ),
    ]);
    let merged = reconcile(&Sides {
        base: &empty,
        local: &l,
        remote: &r,
    });
    let labels: Vec<&str> = merged["task_labels"]
        .values()
        .map(|r| text(r, "name").unwrap())
        .collect();
    assert_eq!(labels, vec!["Review", "review 2"]);
    let paths: Vec<&str> = merged["attachments"]
        .values()
        .map(|r| text(r, "path").unwrap())
        .collect();
    assert_eq!(paths, vec!["plan.md", "plan_1.md"]);
    assert_eq!(
        text(&merged["attachments"]["a2"], "name"),
        Some("plan_1.md")
    );
}

#[test]
fn references_to_gone_rows_are_fixed_or_dropped_in_cascade() {
    let base = snap(&[
        ("people", vec![person("p", "Ann", 0)]),
        (
            "statuses",
            vec![
                json!({ "id": "TODO", "value": "TODO", "label": "To do", "position": 0, "updated_at": 0 }),
                json!({ "id": "HOLD", "value": "HOLD", "label": "Hold", "position": 5, "updated_at": 0 }),
            ],
        ),
        (
            "task_folders",
            vec![
                json!({ "id": "root", "path": "", "updated_at": 0 }),
                json!({ "id": "w", "path": "work", "updated_at": 0 }),
            ],
        ),
        (
            "tasks",
            vec![
                json!({ "id": "parent", "path": "", "parent_id": null, "status": "TODO", "assignee_id": null, "updated_at": 0 }),
            ],
        ),
    ]);
    // Local deletes the person, the HOLD status and the folder; remote adds work that uses them.
    let mut local = base.clone();
    local.get_mut("people").unwrap().clear();
    local.get_mut("statuses").unwrap().remove("HOLD");
    local.get_mut("task_folders").unwrap().remove("w");
    local.get_mut("tasks").unwrap().remove("parent");
    let mut remote = base.clone();
    remote.get_mut("tasks").unwrap().insert(
        "t".into(),
        row(json!({ "id": "t", "path": "work", "parent_id": null, "status": "HOLD", "assignee_id": "p", "updated_at": 1 })),
    );
    remote.get_mut("tasks").unwrap().insert(
        "sub".into(),
        row(json!({ "id": "sub", "path": "", "parent_id": "parent", "status": "TODO", "assignee_id": null, "updated_at": 1 })),
    );
    remote.insert(
        "task_comments",
        BTreeMap::from([(
            "c".into(),
            row(json!({ "id": "c", "task_id": "sub", "updated_at": 1 })),
        )]),
    );
    let merged = reconcile(&Sides {
        base: &base,
        local: &local,
        remote: &remote,
    });
    let tasks: Vec<&String> = merged["tasks"].keys().collect();
    assert_eq!(tasks, vec!["t"], "the subtask went with its parent");
    let t = &merged["tasks"]["t"];
    assert_eq!(
        (text(t, "path"), text(t, "status"), t.get("assignee_id")),
        (Some(""), Some("TODO"), Some(&Value::Null))
    );
    assert!(
        merged["task_comments"].is_empty(),
        "the comment went with the subtask"
    );
}

#[test]
fn a_note_whose_folder_is_gone_moves_to_the_root() {
    let base = snap(&[
        (
            "note_folders",
            vec![
                json!({ "id": "root", "path": "", "updated_at": 0 }),
                json!({ "id": "f", "path": "ai", "updated_at": 0 }),
            ],
        ),
        (
            "attachments",
            vec![json!({ "id": "a0", "path": "x.md", "name": "x.md", "updated_at": 0 })],
        ),
        (
            "notes",
            vec![json!({ "id": "n0", "attachment_id": "a0", "updated_at": 0 })],
        ),
    ]);
    let mut local = base.clone();
    local.get_mut("note_folders").unwrap().remove("f");
    let mut remote = base.clone();
    remote.get_mut("attachments").unwrap().insert(
        "a1".into(),
        row(json!({ "id": "a1", "path": "ai/x.md", "name": "x.md", "updated_at": 1 })),
    );
    remote.get_mut("notes").unwrap().insert(
        "n1".into(),
        row(json!({ "id": "n1", "attachment_id": "a1", "updated_at": 1 })),
    );
    let merged = reconcile(&Sides {
        base: &base,
        local: &local,
        remote: &remote,
    });
    assert_eq!(text(&merged["attachments"]["a1"], "path"), Some("x_1.md"));
}

#[test]
fn merging_a_merge_changes_nothing() {
    let s = snap(&[("people", vec![person("p", "Ann", 1)])]);
    assert_eq!(
        reconcile(&Sides {
            base: &s,
            local: &s,
            remote: &s
        }),
        s
    );
}

#[test]
fn a_rename_on_one_side_and_an_edit_on_the_other_both_survive() {
    let body = |path: &str, sha: &str, at: i64| json!({ "id": "a", "path": path, "sha256": sha, "updated_at": at });
    let base = snap(&[("attachments", vec![body("plan.md", "old", 1)])]);
    let edited = snap(&[("attachments", vec![body("plan.md", "new", 5)])]);
    let renamed = snap(&[("attachments", vec![body("plan_v2.md", "old", 9)])]);
    let merged = reconcile(&Sides {
        base: &base,
        local: &edited,
        remote: &renamed,
    });
    let a = &merged["attachments"]["a"];
    assert_eq!(
        (text(a, "path"), text(a, "sha256")),
        (Some("plan_v2.md"), Some("new"))
    );
    assert_eq!(
        merged,
        reconcile(&Sides {
            base: &base,
            local: &renamed,
            remote: &edited
        }),
        "the same on both devices"
    );
}

#[test]
fn a_path_with_every_suffix_taken_gets_the_row_id() {
    let taken: BTreeSet<String> = std::iter::once("work/plan.md".to_string())
        .chain((1..=10_000).map(|n| format!("work/plan_{n}.md")))
        .collect();
    assert_eq!(free_path("work/plan.md", &taken), None);
    assert_eq!(with_id("work/plan.md", "01ABC"), "work/plan_01abc.md");
}
