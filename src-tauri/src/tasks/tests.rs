use super::*;
use crate::db::open_memory;

fn ids(conn: &Connection) -> Vec<String> {
    data(conn, "2026-10-04")
        .unwrap()
        .tasks
        .into_iter()
        .map(|t| t.title)
        .collect()
}

#[test]
fn new_tasks_go_to_the_top_and_start_not_started() {
    let mut conn = open_memory();
    create(
        &mut conn,
        &NewTask {
            path: "work".into(),
            title: "A".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let b = create(
        &mut conn,
        &NewTask {
            path: "work".into(),
            title: "B".into(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(ids(&conn), vec!["B", "A"]);
    assert_eq!(b.status, FIRST);
    assert_eq!(data(&conn, "x").unwrap().folders, vec!["work"]);
}

#[test]
fn reorder_writes_only_the_moved_key() {
    let mut conn = open_memory();
    let a = create(
        &mut conn,
        &NewTask {
            title: "A".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let b = create(
        &mut conn,
        &NewTask {
            title: "B".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let after_a = crate::order::between(Some(&a.order_key), None);
    update(
        &mut conn,
        &b.id,
        &Patch {
            order_key: Some(after_a),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(ids(&conn), vec!["A", "B"]);
    assert_eq!(get(&conn, &a.id).unwrap().order_key, a.order_key);
}

#[test]
fn done_sets_and_clears_completed_at() {
    let mut conn = open_memory();
    let t = create(
        &mut conn,
        &NewTask {
            title: "A".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let done = update(
        &mut conn,
        &t.id,
        &Patch {
            status: Some(DONE.into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(done.completed_at.is_some());
    let back = update(
        &mut conn,
        &t.id,
        &Patch {
            status: Some("IN_PROGRESS".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(back.completed_at.is_none());
}

#[test]
fn patch_clears_optional_fields_with_null() {
    let mut conn = open_memory();
    let t = create(
        &mut conn,
        &NewTask {
            title: "A".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let p: Patch = serde_json::from_str(r#"{"dueDate":"2026-10-10","dueTime":"17:30"}"#).unwrap();
    let t2 = update(&mut conn, &t.id, &p).unwrap();
    assert_eq!(
        (t2.due_date.as_deref(), t2.due_time.as_deref()),
        (Some("2026-10-10"), Some("17:30"))
    );
    let p: Patch = serde_json::from_str(r#"{"dueTime":null}"#).unwrap();
    let t3 = update(&mut conn, &t.id, &p).unwrap();
    assert_eq!(
        (t3.due_date.as_deref(), t3.due_time),
        (Some("2026-10-10"), None)
    );
}

#[test]
fn folder_rename_rewrites_paths_below() {
    let mut conn = open_memory();
    create(
        &mut conn,
        &NewTask {
            path: "work/tend_v0_1".into(),
            title: "A".into(),
            ..Default::default()
        },
    )
    .unwrap();
    create(
        &mut conn,
        &NewTask {
            path: "work_x".into(),
            title: "B".into(),
            ..Default::default()
        },
    )
    .unwrap();
    folder_rename(&mut conn, "work", "job").unwrap();
    let d = data(&conn, "x").unwrap();
    assert_eq!(d.folders, vec!["job", "job/tend_v0_1", "work_x"]);
    let paths: Vec<_> = d.tasks.iter().map(|t| t.path.as_str()).collect();
    assert!(paths.contains(&"job/tend_v0_1") && paths.contains(&"work_x"));
    assert!(
        folder_rename(&mut conn, "job", "work_x").is_err(),
        "name clash"
    );
    assert!(
        folder_move(&mut conn, "job", "job/tend_v0_1").is_err(),
        "into itself"
    );
}

#[test]
fn moving_a_task_moves_its_subtasks() {
    let mut conn = open_memory();
    let p = create(
        &mut conn,
        &NewTask {
            path: "work".into(),
            title: "Parent".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let s = create(
        &mut conn,
        &NewTask {
            title: "Sub".into(),
            parent_id: Some(p.id.clone()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(s.path, "work", "subtasks live in the parent's folder");
    move_to(&mut conn, &p.id, "home").unwrap();
    assert_eq!(get(&conn, &s.id).unwrap().path, "home");
    assert!(move_to(&mut conn, &s.id, "work").is_err());
}

#[test]
fn folder_delete_removes_tasks_inside() {
    let mut conn = open_memory();
    create(
        &mut conn,
        &NewTask {
            path: "work/a".into(),
            title: "A".into(),
            ..Default::default()
        },
    )
    .unwrap();
    create(
        &mut conn,
        &NewTask {
            path: "home".into(),
            title: "B".into(),
            ..Default::default()
        },
    )
    .unwrap();
    folder_delete(&mut conn, "work").unwrap();
    assert_eq!(ids(&conn), vec!["B"]);
    assert_eq!(data(&conn, "x").unwrap().folders, vec!["home"]);
    assert!(folder_delete(&mut conn, "").is_err());
}

#[test]
fn my_day_overrides() {
    let mut conn = open_memory();
    let t = create(
        &mut conn,
        &NewTask {
            title: "A".into(),
            ..Default::default()
        },
    )
    .unwrap();
    set_my_day(&conn, &t.id, "2026-10-04", Some(true)).unwrap();
    assert_eq!(
        data(&conn, "2026-10-04").unwrap().my_day,
        vec![(t.id.clone(), true)]
    );
    set_my_day(&conn, &t.id, "2026-10-04", Some(false)).unwrap();
    assert_eq!(
        data(&conn, "2026-10-04").unwrap().my_day,
        vec![(t.id.clone(), false)]
    );
    set_my_day(&conn, &t.id, "2026-10-04", None).unwrap();
    assert!(data(&conn, "2026-10-04").unwrap().my_day.is_empty());
    assert!(data(&conn, "2026-10-05").unwrap().my_day.is_empty());
}

#[test]
fn subtasks_are_one_level_deep() {
    let mut conn = open_memory();
    let parent = create(
        &mut conn,
        &NewTask {
            title: "Parent".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let sub = create(
        &mut conn,
        &NewTask {
            title: "Sub".into(),
            parent_id: Some(parent.id.clone()),
            ..Default::default()
        },
    )
    .unwrap();
    let deeper = create(
        &mut conn,
        &NewTask {
            title: "Deeper".into(),
            parent_id: Some(sub.id.clone()),
            ..Default::default()
        },
    );
    assert!(deeper.unwrap_err().is(crate::error::errors::SUBTASK_DEPTH));
}

#[test]
fn folder_names_are_free_text_and_unique_ignoring_case() {
    let mut conn = open_memory();
    folder_create(&mut conn, "Client Work").unwrap();
    assert!(folder_create(&mut conn, "client work")
        .unwrap_err()
        .is(errors::NAME_TAKEN));
    assert_eq!(
        folder_rename(&mut conn, "Client Work", "client work").unwrap(),
        "client work"
    );
    assert!(folder_create(&mut conn, "a:b").is_err());
}
