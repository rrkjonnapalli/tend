use super::*;
use crate::db::open_memory;
use crate::enums::rows::{self, NewRow};
use crate::enums::EnumStatus;
use crate::tasks::{create, NewTask};

fn task(conn: &mut Connection, parent: Option<String>) -> String {
    let new = NewTask {
        title: "T".into(),
        parent_id: parent,
        ..Default::default()
    };
    create(conn, &new).unwrap().id
}

fn tag(conn: &Connection, value: &str) {
    let new = NewRow {
        kind: EnumKind::Tag,
        value,
        label: &value.to_lowercase(),
        meta: Default::default(),
    };
    rows::insert(conn, &new).unwrap();
}

/// Puts the tag OPS, and only it, on the source.
fn tag_ops(conn: &Connection, source: Source) -> AppResult<()> {
    let wanted = Wanted {
        kind: EnumKind::Tag,
        values: &["OPS".to_string()],
    };
    set_enum_values(conn, source, wanted)
}

fn count(conn: &Connection) -> i64 {
    conn.query_row("SELECT count(*) FROM tend_links", [], |r| r.get(0))
        .unwrap()
}

fn of(id: &str) -> Source<'_> {
    Source {
        kind: SourceType::Task,
        id,
    }
}

#[test]
fn deleting_a_parent_task_drops_its_subtasks_links() {
    let mut conn = open_memory();
    tag(&conn, "OPS");
    let parent = task(&mut conn, None);
    let sub = task(&mut conn, Some(parent.clone()));
    tag_ops(&conn, of(&parent)).unwrap();
    tag_ops(&conn, of(&sub)).unwrap();
    assert_eq!(count(&conn), 2);
    crate::tasks::delete(&conn, &parent).unwrap();
    assert_eq!(
        count(&conn),
        0,
        "the cascade to the subtask fires its trigger too"
    );
}

#[test]
fn deleting_the_enum_or_the_attachment_drops_the_link() {
    let mut conn = open_memory();
    tag(&conn, "OPS");
    crate::notes::seed(&conn, "n", "n.md");
    let t = task(&mut conn, None);
    tag_ops(&conn, of(&t)).unwrap();
    insert(&conn, of(&t), Target::attachment("a_n")).unwrap();
    assert_eq!(enum_usage(&conn, EnumKind::Tag, "OPS").unwrap(), 1);
    rows::delete(&conn, EnumKind::Tag, "OPS").unwrap();
    assert_eq!(count(&conn), 1);
    conn.execute("DELETE FROM attachments WHERE id = 'a_n'", [])
        .unwrap();
    assert_eq!(count(&conn), 0);
}

#[test]
fn an_attachment_link_is_unique_with_an_empty_kind() {
    let mut conn = open_memory();
    crate::notes::seed(&conn, "n", "n.md");
    let t = task(&mut conn, None);
    insert(&conn, of(&t), Target::attachment("a_n")).unwrap();
    insert(&conn, of(&t), Target::attachment("a_n")).unwrap();
    assert_eq!(count(&conn), 1);
}

#[test]
fn a_blocked_value_cant_be_linked_and_blocking_removes_its_links() {
    let mut conn = open_memory();
    tag(&conn, "OPS");
    let t = task(&mut conn, None);
    tag_ops(&conn, of(&t)).unwrap();
    let ops = rows::Key {
        kind: EnumKind::Tag,
        value: "OPS",
    };
    crate::db::in_transaction(&mut conn, |tx| {
        rows::set_status(tx, ops, EnumStatus::Blocked)
    })
    .unwrap();
    assert_eq!(count(&conn), 0);
    assert!(tag_ops(&conn, of(&t)).unwrap_err().is(errors::TAG_BLOCKED));
    assert_eq!(count(&conn), 0);
}

#[test]
fn removing_a_link_that_isnt_there_is_not_found() {
    let conn = open_memory();
    assert!(delete_by_id(&conn, "nope")
        .unwrap_err()
        .is(errors::NOT_FOUND));
}
