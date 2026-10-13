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

/// A `tend_enums` row; `position` goes into its meta JSON.
fn enum_row(id: &str, kind: EnumKind, value: &str, label: &str, position: i64) -> Value {
    let meta = json!({ "position": position }).to_string();
    json!({ "id": id, "kind": kind.as_str(), "value": value, "label": label, "meta": meta, "updated_at": 0 })
}

/// A task carrying a tag.
fn tag_link(id: &str, task: &str, tag: &str, at: i64) -> Value {
    json!({
        "id": id,
        "source_type": SourceType::Task.as_str(),
        "source_id": task,
        "target_type": TargetType::Enum.as_str(),
        "target_kind": EnumKind::Tag.as_str(),
        "target_value": tag,
        "updated_at": at
    })
}

fn names_of(s: &Snapshot) -> Vec<String> {
    s[tables::PEOPLE]
        .values()
        .map(|r| text(r, "first_name").unwrap().to_string())
        .collect()
}

#[test]
fn a_change_on_one_side_wins_without_looking_at_the_clock() {
    let base = snap(&[(tables::PEOPLE, vec![person("p", "Ann", 10)])]);
    let local = snap(&[(tables::PEOPLE, vec![person("p", "Ann", 10)])]);
    let remote = snap(&[(tables::PEOPLE, vec![person("p", "Anna", 1)])]);
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
    let base = snap(&[(tables::PEOPLE, vec![person("p", "Ann", 1)])]);
    let a = snap(&[(tables::PEOPLE, vec![person("p", "Anna", 5)])]);
    let b = snap(&[(tables::PEOPLE, vec![person("p", "Annie", 9)])]);
    assert_eq!(
        names_of(&reconcile(&Sides {
            base: &base,
            local: &a,
            remote: &b
        })),
        vec!["Annie"]
    );
    let tie_a = snap(&[(tables::PEOPLE, vec![person("p", "Anna", 5)])]);
    let tie_b = snap(&[(tables::PEOPLE, vec![person("p", "Annie", 5)])]);
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
    let base = snap(&[(tables::PEOPLE, vec![person("p", "Ann", 1)])]);
    let local = snap(&[(tables::PEOPLE, vec![person("n", "New", 3)])]);
    let remote = snap(&[(
        tables::PEOPLE,
        vec![person("p", "Edited", 9), person("r", "Remote", 2)],
    )]);
    let merged = reconcile(&Sides {
        base: &base,
        local: &local,
        remote: &remote,
    });
    let ids: Vec<&String> = merged[tables::PEOPLE].keys().collect();
    assert_eq!(ids, vec!["n", "r"]);
}

#[test]
fn colliding_links_and_folders_collapse_to_the_smallest_id() {
    let tables = [
        (
            tables::TASKS,
            vec![json!({ "id": "t", "path": "", "status": "NOT_STARTED", "updated_at": 0 })],
        ),
        (
            tables::ENUMS,
            vec![
                enum_row("T1", EnumKind::Tag, "OPS", "ops", 0),
                enum_row("S1", EnumKind::TaskStatus, "NOT_STARTED", "Not started", 0),
            ],
        ),
        (
            tables::TASK_FOLDERS,
            vec![json!({ "id": "root", "path": "", "updated_at": 0 })],
        ),
    ];
    let base = snap(&tables);
    let mut l = snap(&[
        (tables::LINKS, vec![tag_link("b", "t", "OPS", 1)]),
        (
            tables::ENUMS,
            vec![enum_row("T3", EnumKind::Tag, "Q4", "q4", 0)],
        ),
        (
            tables::TASK_FOLDERS,
            vec![json!({ "id": "f2", "path": "work", "updated_at": 1 })],
        ),
    ]);
    let mut r = snap(&[
        (tables::LINKS, vec![tag_link("a", "t", "OPS", 1)]),
        (
            tables::ENUMS,
            vec![enum_row("T2", EnumKind::Tag, "Q4", "q4", 0)],
        ),
        (
            tables::TASK_FOLDERS,
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
    assert_eq!(merged[tables::LINKS].keys().collect::<Vec<_>>(), vec!["a"]);
    let enums: Vec<&String> = merged[tables::ENUMS].keys().collect();
    assert_eq!(
        enums,
        vec!["S1", "T1", "T2"],
        "the same tag made twice keeps the smaller id"
    );
    let folders: Vec<&String> = merged[tables::TASK_FOLDERS].keys().collect();
    assert_eq!(folders, vec!["f1", "root"]);
}

#[test]
fn equal_label_names_get_numbered_and_clashing_note_paths_a_suffix() {
    let empty = Snapshot::new();
    let l = snap(&[
        (
            tables::ENUMS,
            vec![
                enum_row("x", EnumKind::TaskLabel, "REVIEW", "Review", 0),
                enum_row("z", EnumKind::TaskStatus, "REVIEW", "review", 0),
            ],
        ),
        (
            tables::ATTACHMENTS,
            vec![json!({ "id": "a1", "path": "plan.md", "name": "plan.md", "updated_at": 1 })],
        ),
    ]);
    let r = snap(&[
        (
            tables::ENUMS,
            vec![enum_row(
                "y",
                EnumKind::TaskLabel,
                "CODE_REVIEW",
                "review",
                0,
            )],
        ),
        (
            tables::ATTACHMENTS,
            vec![json!({ "id": "a2", "path": "plan.md", "name": "plan.md", "updated_at": 1 })],
        ),
    ]);
    let merged = reconcile(&Sides {
        base: &empty,
        local: &l,
        remote: &r,
    });
    let labels: Vec<&str> = merged[tables::ENUMS]
        .values()
        .map(|r| text(r, "label").unwrap())
        .collect();
    assert_eq!(
        labels,
        vec!["Review", "review 2", "review"],
        "numbered within a kind; a status may share a label's name"
    );
    let paths: Vec<&str> = merged[tables::ATTACHMENTS]
        .values()
        .map(|r| text(r, "path").unwrap())
        .collect();
    assert_eq!(paths, vec!["plan.md", "plan_1.md"]);
    assert_eq!(
        text(&merged[tables::ATTACHMENTS]["a2"], "name"),
        Some("plan_1.md")
    );
}

#[test]
fn references_to_gone_rows_are_fixed_or_dropped_in_cascade() {
    let base = snap(&[
        (tables::PEOPLE, vec![person("p", "Ann", 0)]),
        (
            tables::ENUMS,
            vec![
                enum_row("TODO", EnumKind::TaskStatus, "TODO", "To do", 0),
                enum_row("HOLD", EnumKind::TaskStatus, "HOLD", "Hold", 5),
            ],
        ),
        (
            tables::TASK_FOLDERS,
            vec![
                json!({ "id": "root", "path": "", "updated_at": 0 }),
                json!({ "id": "w", "path": "work", "updated_at": 0 }),
            ],
        ),
        (
            tables::TASKS,
            vec![
                json!({ "id": "parent", "path": "", "parent_id": null, "status": "TODO", "assignee_id": null, "updated_at": 0 }),
            ],
        ),
    ]);
    // Local deletes the person, the HOLD status and the folder; remote adds work that uses them.
    let mut local = base.clone();
    local.get_mut(tables::PEOPLE).unwrap().clear();
    local.get_mut(tables::ENUMS).unwrap().remove("HOLD");
    local.get_mut(tables::TASK_FOLDERS).unwrap().remove("w");
    local.get_mut(tables::TASKS).unwrap().remove("parent");
    let mut remote = base.clone();
    remote.get_mut(tables::TASKS).unwrap().insert(
        "t".into(),
        row(json!({ "id": "t", "path": "work", "parent_id": null, "status": "HOLD", "assignee_id": "p", "updated_at": 1 })),
    );
    remote.get_mut(tables::TASKS).unwrap().insert(
        "sub".into(),
        row(json!({ "id": "sub", "path": "", "parent_id": "parent", "status": "TODO", "assignee_id": null, "updated_at": 1 })),
    );
    remote.insert(
        tables::TASK_COMMENTS,
        BTreeMap::from([(
            "c".into(),
            row(json!({ "id": "c", "task_id": "sub", "updated_at": 1 })),
        )]),
    );
    remote.insert(
        tables::LINKS,
        BTreeMap::from([
            ("l1".into(), row(tag_link("l1", "sub", "OPS", 1))),
            ("l2".into(), row(tag_link("l2", "t", "GONE", 1))),
        ]),
    );
    let merged = reconcile(&Sides {
        base: &base,
        local: &local,
        remote: &remote,
    });
    let tasks: Vec<&String> = merged[tables::TASKS].keys().collect();
    assert_eq!(tasks, vec!["t"], "the subtask went with its parent");
    let t = &merged[tables::TASKS]["t"];
    assert_eq!(
        (text(t, "path"), text(t, "status"), t.get("assignee_id")),
        (Some(""), Some("TODO"), Some(&Value::Null))
    );
    assert!(
        merged[tables::TASK_COMMENTS].is_empty(),
        "the comment went with the subtask"
    );
    assert!(
        merged[tables::LINKS].is_empty(),
        "links to a gone task or a missing tag are dropped"
    );
}

#[test]
fn a_note_whose_folder_is_gone_moves_to_the_root() {
    let base = snap(&[
        (
            tables::NOTE_FOLDERS,
            vec![
                json!({ "id": "root", "path": "", "updated_at": 0 }),
                json!({ "id": "f", "path": "ai", "updated_at": 0 }),
            ],
        ),
        (
            tables::ATTACHMENTS,
            vec![json!({ "id": "a0", "path": "x.md", "name": "x.md", "updated_at": 0 })],
        ),
        (
            tables::NOTES,
            vec![json!({ "id": "n0", "attachment_id": "a0", "updated_at": 0 })],
        ),
    ]);
    let mut local = base.clone();
    local.get_mut(tables::NOTE_FOLDERS).unwrap().remove("f");
    let mut remote = base.clone();
    remote.get_mut(tables::ATTACHMENTS).unwrap().insert(
        "a1".into(),
        row(json!({ "id": "a1", "path": "ai/x.md", "name": "x.md", "updated_at": 1 })),
    );
    remote.get_mut(tables::NOTES).unwrap().insert(
        "n1".into(),
        row(json!({ "id": "n1", "attachment_id": "a1", "updated_at": 1 })),
    );
    let merged = reconcile(&Sides {
        base: &base,
        local: &local,
        remote: &remote,
    });
    assert_eq!(
        text(&merged[tables::ATTACHMENTS]["a1"], "path"),
        Some("x_1.md")
    );
}

#[test]
fn merging_a_merge_changes_nothing() {
    let s = snap(&[(tables::PEOPLE, vec![person("p", "Ann", 1)])]);
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
    let base = snap(&[(tables::ATTACHMENTS, vec![body("plan.md", "old", 1)])]);
    let edited = snap(&[(tables::ATTACHMENTS, vec![body("plan.md", "new", 5)])]);
    let renamed = snap(&[(tables::ATTACHMENTS, vec![body("plan_v2.md", "old", 9)])]);
    let merged = reconcile(&Sides {
        base: &base,
        local: &edited,
        remote: &renamed,
    });
    let a = &merged[tables::ATTACHMENTS]["a"];
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
        .chain((1..=100_000).map(|n| format!("work/plan_{n}.md")))
        .collect();
    assert_eq!(free_path("work/plan.md", &taken), None);
    assert_eq!(with_id("work/plan.md", "01ABC"), "work/plan_01abc.md");
}

#[test]
fn a_fallback_the_rules_refuse_too_is_not_a_change_every_pass() {
    // No root folder row: a task moved to the root still fails the folder rule.
    let task = json!({ "id": "t", "path": "gone", "title": "T", "updated_at": 1 });
    let mut merged = snap(&[(tables::TASKS, vec![task]), (tables::TASK_FOLDERS, vec![])]);
    assert!(fix_pass(&mut merged));
    assert!(
        !fix_pass(&mut merged),
        "the second pass has nothing left to do"
    );
}

fn word(id: &str, title: &str, meaning: &str, notes: &str, at: i64) -> Value {
    json!({ "id": id, "title": title, "kind": "WORD", "meaning": meaning, "notes": notes, "dictionary": null, "updated_at": at })
}

fn word_kinds() -> Value {
    enum_row("WORD_KIND:WORD", EnumKind::WordKind, "WORD", "word", 0)
}

fn word_tag(id: &str, word: &str, tag: &str) -> Value {
    let mut link = tag_link(id, word, tag, 1);
    link["source_type"] = json!(SourceType::Word.as_str());
    link
}

#[test]
fn the_same_word_added_on_two_devices_becomes_one() {
    let base = snap(&[(
        tables::ENUMS,
        vec![
            word_kinds(),
            enum_row("T1", EnumKind::Tag, "READING", "reading", 0),
            enum_row("T2", EnumKind::Tag, "SOUND", "sound", 0),
        ],
    )]);
    let mut local = base.clone();
    local.insert(
        tables::WORDS,
        BTreeMap::from([("w1".into(), row(word("w1", "petrichor", "", "rain", 5)))]),
    );
    local.insert(
        tables::LINKS,
        BTreeMap::from([("l1".into(), row(word_tag("l1", "w1", "READING")))]),
    );
    let mut remote = base.clone();
    remote.insert(
        tables::WORDS,
        BTreeMap::from([(
            "w2".into(),
            row(word("w2", "petrichor", "smell of rain", "", 9)),
        )]),
    );
    remote.insert(
        tables::LINKS,
        BTreeMap::from([
            ("l2".into(), row(word_tag("l2", "w2", "READING"))),
            ("l3".into(), row(word_tag("l3", "w2", "SOUND"))),
        ]),
    );
    let merged = reconcile(&Sides {
        base: &base,
        local: &local,
        remote: &remote,
    });
    let words: Vec<&String> = merged[tables::WORDS].keys().collect();
    assert_eq!(words, vec!["w1"], "the smaller id is kept");
    let w = &merged[tables::WORDS]["w1"];
    assert_eq!(
        (text(w, "meaning"), text(w, tables::NOTES)),
        (Some("smell of rain"), Some("rain")),
        "the newer row's fields, an empty one filled from the other"
    );
    let tags: Vec<(&str, &str)> = merged[tables::LINKS]
        .values()
        .map(|l| {
            (
                text(l, "source_id").unwrap(),
                text(l, "target_value").unwrap(),
            )
        })
        .collect();
    assert_eq!(
        tags,
        vec![("w1", "READING"), ("w1", "SOUND")],
        "tags moved, doubles dropped"
    );
    let flipped = reconcile(&Sides {
        base: &base,
        local: &remote,
        remote: &local,
    });
    assert_eq!(merged, flipped, "the same on both devices");
}

#[test]
fn the_same_link_saved_on_two_devices_becomes_one() {
    let link = |id: &str, why: &str, at: i64| json!({ "id": id, "kind": "LINK", "value": "https://svelte.dev", "title": null, "why": why, "url_key": "svelte.dev", "updated_at": at });
    let text_ref = json!({ "id": "r9", "kind": "TEXT", "value": "Perfect Days", "title": null, "why": "", "url_key": null, "updated_at": 1 });
    let empty = Snapshot::new();
    let l = snap(&[(tables::REFS, vec![link("r1", "runes", 1), text_ref.clone()])]);
    let mut text_twin = text_ref.clone();
    text_twin["id"] = json!("r8");
    let r = snap(&[(tables::REFS, vec![link("r2", "", 2), text_twin])]);
    let merged = reconcile(&Sides {
        base: &empty,
        local: &l,
        remote: &r,
    });
    let ids: Vec<&String> = merged[tables::REFS].keys().collect();
    assert_eq!(
        ids,
        vec!["r1", "r8", "r9"],
        "text references have no key and both stay"
    );
    assert_eq!(text(&merged[tables::REFS]["r1"], "why"), Some("runes"));
}

#[test]
fn snaps_and_stacks_whose_stack_is_gone_go_to_the_board() {
    let stack = |id: &str, parent: Option<&str>| json!({ "id": id, "parent_id": parent, "name": id, "updated_at": 0 });
    let base = snap(&[(tables::SNAP_STACKS, vec![stack("s1", None)])]);
    let mut local = base.clone();
    local.get_mut(tables::SNAP_STACKS).unwrap().remove("s1");
    let mut remote = base.clone();
    remote
        .get_mut(tables::SNAP_STACKS)
        .unwrap()
        .insert("s2".into(), row(stack("s2", Some("s1"))));
    remote.insert(
        tables::SNAPS,
        BTreeMap::from([(
            "n".into(),
            row(json!({ "id": "n", "stack_id": "s1", "text": "x", "updated_at": 1 })),
        )]),
    );
    let merged = reconcile(&Sides {
        base: &base,
        local: &local,
        remote: &remote,
    });
    assert_eq!(merged[tables::SNAP_STACKS]["s2"]["parent_id"], Value::Null);
    assert_eq!(merged[tables::SNAPS]["n"]["stack_id"], Value::Null);
}

#[test]
fn stacks_nested_in_each_other_are_untangled() {
    let stack = |id: &str, parent: Option<&str>, at: i64| json!({ "id": id, "parent_id": parent, "name": id, "updated_at": at });
    let base = snap(&[(
        tables::SNAP_STACKS,
        vec![stack("a", None, 0), stack("b", None, 0)],
    )]);
    // One device moves a into b, the other b into a.
    let local = snap(&[(
        tables::SNAP_STACKS,
        vec![stack("a", Some("b"), 1), stack("b", None, 0)],
    )]);
    let remote = snap(&[(
        tables::SNAP_STACKS,
        vec![stack("a", None, 0), stack("b", Some("a"), 1)],
    )]);
    let merged = reconcile(&Sides {
        base: &base,
        local: &local,
        remote: &remote,
    });
    let parent = |id: &str| merged[tables::SNAP_STACKS][id]["parent_id"].clone();
    assert_eq!((parent("a"), parent("b")), (json!("b"), Value::Null));
}

#[test]
fn a_word_whose_kind_is_gone_becomes_other() {
    let slang = enum_row("K", EnumKind::WordKind, "SLANG", "slang", 6);
    let base = snap(&[
        (tables::ENUMS, vec![word_kinds(), slang]),
        (tables::WORDS, vec![]),
    ]);
    let mut local = base.clone();
    local.get_mut(tables::ENUMS).unwrap().remove("K");
    let mut remote = base.clone();
    let mut w = word("w", "Gnarly", "", "", 1);
    w["kind"] = json!("SLANG");
    remote
        .get_mut(tables::WORDS)
        .unwrap()
        .insert("w".into(), row(w));
    let merged = reconcile(&Sides {
        base: &base,
        local: &local,
        remote: &remote,
    });
    let w = &merged[tables::WORDS]["w"];
    assert_eq!(
        (text(w, "kind"), text(w, "title")),
        (Some("OTHER"), Some("Gnarly"))
    );
}

#[test]
fn a_default_enum_row_wins_over_one_a_user_made() {
    let empty = Snapshot::new();
    // One device made CONCEPT by hand; the other has it as a default (a newer app's seed).
    let made = enum_row("01ABC", EnumKind::WordKind, "CONCEPT", "concept", 3);
    let mut seeded = enum_row(
        "WORD_KIND:CONCEPT",
        EnumKind::WordKind,
        "CONCEPT",
        "concept",
        3,
    );
    seeded["is_default"] = json!(1);
    let l = snap(&[(tables::ENUMS, vec![made])]);
    let r = snap(&[(tables::ENUMS, vec![seeded])]);
    let merged = reconcile(&Sides {
        base: &empty,
        local: &l,
        remote: &r,
    });
    let ids: Vec<&String> = merged[tables::ENUMS].keys().collect();
    assert_eq!(ids, vec!["WORD_KIND:CONCEPT"]);
}

#[test]
fn a_recolor_and_a_reorder_of_one_label_both_survive() {
    let label = |meta: Value, at: i64| {
        let mut row = enum_row("TASK_LABEL:ALPHA", EnumKind::TaskLabel, "ALPHA", "Alpha", 0);
        row["meta"] = json!(meta.to_string());
        row["updated_at"] = json!(at);
        row
    };
    let base = snap(&[(
        tables::ENUMS,
        vec![label(json!({ "color": "#111111", "position": 0 }), 0)],
    )]);
    let l = snap(&[(
        tables::ENUMS,
        vec![label(json!({ "color": "#222222", "position": 0 }), 5)],
    )]);
    let r = snap(&[(
        tables::ENUMS,
        vec![label(json!({ "color": "#111111", "position": 3 }), 9)],
    )]);
    let merged = reconcile(&Sides {
        base: &base,
        local: &l,
        remote: &r,
    });
    let meta = text(&merged[tables::ENUMS]["TASK_LABEL:ALPHA"], "meta").unwrap();
    assert_eq!(
        meta,
        json!({ "color": "#222222", "position": 3 }).to_string()
    );
}

#[test]
fn a_tag_blocked_on_one_device_comes_off_what_another_put_it_on() {
    let tag = |value: &str, status: &str, at: i64| {
        let mut row = enum_row(value, EnumKind::Tag, value, value, 0);
        row[columns::STATUS] = json!(status);
        row[columns::UPDATED_AT] = json!(at);
        row
    };
    let tasks = (tables::TASKS, vec![json!({ "id": "t", "updated_at": 0 })]);
    let active = vec![tag("OPS", "ACTIVE", 0), tag("Q4", "ACTIVE", 0)];
    let base = snap(&[tasks.clone(), (tables::ENUMS, active.clone())]);
    // This device tags the task; the other blocks one of the two tags meanwhile.
    let links = vec![tag_link("l1", "t", "OPS", 5), tag_link("l2", "t", "Q4", 5)];
    let local = snap(&[
        tasks.clone(),
        (tables::ENUMS, active),
        (tables::LINKS, links),
    ]);
    let blocked = vec![tag("OPS", "BLOCKED", 9), tag("Q4", "ACTIVE", 0)];
    let remote = snap(&[tasks, (tables::ENUMS, blocked)]);
    let merged = reconcile(&Sides {
        base: &base,
        local: &local,
        remote: &remote,
    });
    assert_eq!(
        text(&merged[tables::ENUMS]["OPS"], columns::STATUS),
        Some("BLOCKED")
    );
    let kept: Vec<&String> = merged[tables::LINKS].keys().collect();
    assert_eq!(kept, vec!["l2"], "only the link to the blocked tag goes");
}

#[test]
fn a_status_without_a_position_is_never_the_first_status() {
    let mut unplaced = enum_row("AAA", EnumKind::TaskStatus, "AAA", "Aaa", 0);
    unplaced[columns::META] = json!("{}");
    let placed = enum_row("TODO", EnumKind::TaskStatus, "TODO", "To do", 7);
    let task = json!({ "id": "t", "status": "GONE", "updated_at": 0 });
    let local = snap(&[
        (tables::ENUMS, vec![unplaced, placed]),
        (tables::TASKS, vec![task]),
    ]);
    let empty = Snapshot::new();
    let merged = reconcile(&Sides {
        base: &empty,
        local: &local,
        remote: &empty,
    });
    assert_eq!(
        text(&merged[tables::TASKS]["t"], columns::STATUS),
        Some("TODO")
    );
}
