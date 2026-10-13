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
fn new_tasks_sort_by_their_key_and_start_not_started() {
    let mut conn = open_memory();
    create(
        &mut conn,
        &NewTask {
            path: "work".into(),
            title: "A".into(),
            order_key: "a1".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let b = create(
        &mut conn,
        &NewTask {
            path: "work".into(),
            title: "B".into(),
            order_key: "a0".into(),
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
            order_key: "a1".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let b = create(
        &mut conn,
        &NewTask {
            title: "B".into(),
            order_key: "a0".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let after_a = "a2".to_string();
    update(
        &conn,
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
        &conn,
        &t.id,
        &Patch {
            status: Some(DONE.into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(done.completed_at.is_some());
    let back = update(
        &conn,
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
    let t2 = update(&conn, &t.id, &p).unwrap();
    assert_eq!(
        (t2.due_date.as_deref(), t2.due_time.as_deref()),
        (Some("2026-10-10"), Some("17:30"))
    );
    let p: Patch = serde_json::from_str(r#"{"dueTime":null}"#).unwrap();
    let t3 = update(&conn, &t.id, &p).unwrap();
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
    assert_eq!(
        folder_rename(&mut conn, "job", "work_x").unwrap(),
        "work_x_1",
        "a clash gets a suffix"
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
    move_to(
        &mut conn,
        &Move {
            id: &p.id,
            target: "home",
        },
    )
    .unwrap();
    assert_eq!(get(&conn, &s.id).unwrap().path, "home");
    assert!(move_to(
        &mut conn,
        &Move {
            id: &s.id,
            target: "work"
        }
    )
    .is_err());
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
    set_my_day(
        &conn,
        &DayChoice {
            id: &t.id,
            day: "2026-10-04",
            included: Some(true),
        },
    )
    .unwrap();
    assert_eq!(
        data(&conn, "2026-10-04").unwrap().my_day,
        vec![(t.id.clone(), true)]
    );
    set_my_day(
        &conn,
        &DayChoice {
            id: &t.id,
            day: "2026-10-04",
            included: Some(false),
        },
    )
    .unwrap();
    assert_eq!(
        data(&conn, "2026-10-04").unwrap().my_day,
        vec![(t.id.clone(), false)]
    );
    set_my_day(
        &conn,
        &DayChoice {
            id: &t.id,
            day: "2026-10-04",
            included: None,
        },
    )
    .unwrap();
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
fn folder_names_are_slugged_and_clashes_get_a_suffix() {
    let mut conn = open_memory();
    assert_eq!(
        folder_create(&mut conn, " Client Work.").unwrap(),
        "client_work"
    );
    assert_eq!(
        folder_create(&mut conn, "client work").unwrap(),
        "client_work_1"
    );
    assert!(folder_create(&mut conn, "!!").is_err());
    let t = create(
        &mut conn,
        &NewTask {
            path: "Client Work/X".into(),
            title: "T".into(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(t.path, "client_work/x");
}

#[test]
fn a_new_task_can_be_due_on_creation() {
    let mut conn = open_memory();
    let t = create(
        &mut conn,
        &NewTask {
            title: "Today".into(),
            due_date: Some("2026-10-09".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(t.due_date.as_deref(), Some("2026-10-09"));
}

#[test]
fn a_due_date_or_time_that_does_not_exist_is_refused() {
    let mut conn = open_memory();
    let t = create(
        &mut conn,
        &NewTask {
            title: "T".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let due = |date: &str, time: &str| Patch {
        due_date: Some(Some(date.into())),
        due_time: Some(Some(time.into())),
        ..Default::default()
    };
    assert!(update(&conn, &t.id, &due("2026-02-31", "09:00"))
        .unwrap_err()
        .is(errors::DUE_DATE_INVALID));
    assert!(update(&conn, &t.id, &due("2026-02-28", "29:59"))
        .unwrap_err()
        .is(errors::DUE_TIME_INVALID));
    assert!(update(&conn, &t.id, &due("2026-02-28", "17:30")).is_ok());
}

fn titled(conn: &mut Connection, status: Option<&str>) -> AppResult<Task> {
    let new = NewTask {
        title: "T".into(),
        status: status.map(String::from),
        order_key: "a0".into(),
        ..Default::default()
    };
    create(conn, &new)
}

#[test]
fn a_blocked_status_cant_be_given_to_a_task() {
    let mut conn = open_memory();
    conn.execute(
        "UPDATE tend_enums SET status = 'BLOCKED' WHERE value = 'IN_PROGRESS'",
        [],
    )
    .unwrap();
    let refused = titled(&mut conn, Some("IN_PROGRESS")).unwrap_err();
    assert!(refused.is(errors::ENUM_BLOCKED));
    let task = titled(&mut conn, None).unwrap();
    let patch = Patch {
        status: Some("IN_PROGRESS".into()),
        ..Default::default()
    };
    assert!(update(&conn, &task.id, &patch)
        .unwrap_err()
        .is(errors::ENUM_BLOCKED));
    assert_eq!(get(&conn, &task.id).unwrap().status, FIRST);
}

#[test]
fn an_update_answers_with_the_task_as_written() {
    let mut conn = open_memory();
    let task = titled(&mut conn, None).unwrap();
    let patch = Patch {
        title: Some("  Renamed ".into()),
        status: Some(DONE.into()),
        ..Default::default()
    };
    let done = update(&conn, &task.id, &patch).unwrap();
    assert_eq!(done.title, "Renamed");
    assert!(done.completed_at.is_some());
    assert_eq!(done, get(&conn, &task.id).unwrap());
    // Only the title: the status and its stamp stay.
    let patch = Patch {
        title: Some("Again".into()),
        ..Default::default()
    };
    let again = update(&conn, &task.id, &patch).unwrap();
    assert_eq!(again.completed_at, done.completed_at);
    assert_eq!(again.status, DONE);
}

#[test]
fn a_task_or_folder_that_isnt_there_is_not_found() {
    let mut conn = open_memory();
    let not_found = |e: crate::error::AppError| assert!(e.is(errors::NOT_FOUND), "{e}");
    not_found(update(&conn, "nope", &Patch::default()).unwrap_err());
    not_found(delete(&conn, "nope").unwrap_err());
    let change = Move {
        id: "nope",
        target: "work",
    };
    not_found(move_to(&mut conn, &change).unwrap_err());
    not_found(folder_delete(&mut conn, "nope").unwrap_err());
    not_found(folder_rename(&mut conn, "nope", "other").unwrap_err());
}
