use super::*;
use crate::db::open_memory;
use crate::error::{errors, ErrorDef};

fn values(conn: &Connection) -> Vec<String> {
    list(conn).unwrap().into_iter().map(|s| s.value).collect()
}

fn fails_with(r: AppResult<impl std::fmt::Debug>, def: ErrorDef) -> bool {
    match r {
        Err(e) => e.is(def),
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
    assert!(fails_with(
        add(&conn, "Ready", "done", "#000000"),
        errors::STATUS_VALUE
    ));
    assert!(fails_with(
        add(&conn, "Ready", "DONE", "#000000"),
        errors::STATUS_RESERVED
    ));
    assert!(fails_with(
        add(&conn, "Ready", "In_Progress", "#000000"),
        errors::STATUS_VALUE
    ));
    assert!(fails_with(
        add(&conn, "not started", "READY", "#000000"),
        errors::STATUS_LABEL_TAKEN
    ));
    assert!(fails_with(
        add(&conn, "Ready", "READY", "blue"),
        errors::COLOR_INVALID
    ));
    add(&conn, "Ready", "READY", "#000000").unwrap();
    assert!(fails_with(
        add(&conn, "Other", "READY", "#000000"),
        errors::STATUS_VALUE_TAKEN
    ));
    assert!(fails_with(
        add(&conn, "  ", "X", "#000000"),
        errors::STATUS_LABEL_EMPTY
    ));
}

#[test]
fn rename_and_recolor() {
    let conn = open_memory();
    assert_eq!(
        rename(&conn, "NOT_STARTED", "Ready").unwrap().label,
        "Ready"
    );
    assert!(fails_with(
        rename(&conn, "IN_PROGRESS", "ready"),
        errors::STATUS_LABEL_TAKEN
    ));
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
    assert!(fails_with(
        reorder(
            &mut conn,
            &order(&["WAITING", "NOT_STARTED", "IN_PROGRESS", "DONE"])
        ),
        errors::STATUS_ORDER
    ));
    assert!(fails_with(
        reorder(&mut conn, &order(&["NOT_STARTED", "IN_PROGRESS", "DONE"])),
        errors::STATUS_ORDER
    ));
}

#[test]
fn delete_rules() {
    let mut conn = open_memory();
    add(&conn, "Waiting", "WAITING", "#d57149").unwrap();
    assert!(fails_with(delete(&conn, "DONE"), errors::STATUS_DEFAULT));
    let t = crate::tasks::create(
        &mut conn,
        &crate::tasks::NewTask {
            title: "T".into(),
            status: Some("WAITING".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(fails_with(delete(&conn, "WAITING"), errors::STATUS_IN_USE));
    crate::tasks::delete(&conn, &t.id).unwrap();
    delete(&conn, "WAITING").unwrap();
    assert_eq!(values(&conn), vec!["NOT_STARTED", "IN_PROGRESS", "DONE"]);
}

#[test]
fn a_custom_status_id_is_its_value() {
    let conn = open_memory();
    add(&conn, "Blocked", "BLOCKED", "#d4b106").unwrap();
    let id: String = conn
        .query_row("SELECT id FROM statuses WHERE value = 'BLOCKED'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(id, "BLOCKED");
}
