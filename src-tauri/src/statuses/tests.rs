use super::*;
use crate::db::open_memory;

fn values(conn: &Connection) -> Vec<String> {
    list(conn).unwrap().into_iter().map(|s| s.value).collect()
}

fn code(r: AppResult<impl std::fmt::Debug>) -> &'static str {
    match r {
        Err(e) => e.code(),
        Ok(v) => panic!("expected an error, got {v:?}"),
    }
}

#[test]
fn defaults_are_first_middle_last() {
    let conn = open_memory();
    assert_eq!(values(&conn), vec!["NOT_STARTED", "IN_PROGRESS", "DONE"]);
}

#[test]
fn custom_statuses_go_before_done() {
    let conn = open_memory();
    add(&conn, "Waiting", "WAITING", "#d57149").unwrap();
    add(&conn, "In review", "IN_REVIEW", "#6f5ee4").unwrap();
    assert_eq!(
        values(&conn),
        vec!["NOT_STARTED", "IN_PROGRESS", "WAITING", "IN_REVIEW", "DONE"]
    );
}

#[test]
fn add_rules() {
    let conn = open_memory();
    assert_eq!(code(add(&conn, "Ready", "done", "#000000")), "bad_value");
    assert_eq!(code(add(&conn, "Ready", "DONE", "#000000")), "reserved");
    assert_eq!(
        code(add(&conn, "Ready", "In_Progress", "#000000")),
        "bad_value"
    );
    assert_eq!(
        code(add(&conn, "not started", "READY", "#000000")),
        "label_taken"
    );
    assert_eq!(code(add(&conn, "Ready", "READY", "blue")), "bad_color");
    add(&conn, "Ready", "READY", "#000000").unwrap();
    assert_eq!(code(add(&conn, "Other", "READY", "#000000")), "value_taken");
    assert_eq!(code(add(&conn, "  ", "X", "#000000")), "empty_label");
}

#[test]
fn rename_and_recolor() {
    let conn = open_memory();
    assert_eq!(
        rename(&conn, "NOT_STARTED", "Ready").unwrap().label,
        "Ready"
    );
    assert_eq!(code(rename(&conn, "IN_PROGRESS", "ready")), "label_taken");
    assert_eq!(
        rename(&conn, "IN_PROGRESS", "In Progress").unwrap().label,
        "In Progress"
    );
    assert_eq!(recolor(&conn, "DONE", "#62d2a2").unwrap().color, "#62d2a2");
}

#[test]
fn reorder_keeps_first_and_last() {
    let mut conn = open_memory();
    add(&conn, "Waiting", "WAITING", "#d57149").unwrap();
    let order = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    reorder(
        &mut conn,
        &order(&["NOT_STARTED", "WAITING", "IN_PROGRESS", "DONE"]),
    )
    .unwrap();
    assert_eq!(
        values(&conn),
        vec!["NOT_STARTED", "WAITING", "IN_PROGRESS", "DONE"]
    );
    assert_eq!(
        code(reorder(
            &mut conn,
            &order(&["WAITING", "NOT_STARTED", "IN_PROGRESS", "DONE"])
        )),
        "bad_order"
    );
    assert_eq!(
        code(reorder(
            &mut conn,
            &order(&["NOT_STARTED", "IN_PROGRESS", "DONE"])
        )),
        "bad_order"
    );
}

#[test]
fn delete_rules() {
    let mut conn = open_memory();
    add(&conn, "Waiting", "WAITING", "#d57149").unwrap();
    assert_eq!(code(delete(&conn, "DONE")), "default");
    let t = crate::tasks::create(
        &mut conn,
        &crate::tasks::NewTask {
            title: "T".into(),
            status: Some("WAITING".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(code(delete(&conn, "WAITING")), "in_use");
    crate::tasks::delete(&conn, &t.id).unwrap();
    delete(&conn, "WAITING").unwrap();
    assert_eq!(values(&conn), vec!["NOT_STARTED", "IN_PROGRESS", "DONE"]);
}
