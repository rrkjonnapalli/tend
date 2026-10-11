use super::*;
use crate::db::open_memory;
use crate::enums::rows::{self, NewRow};
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
    set_enum_values(&conn, of(&parent), EnumKind::Tag, &["OPS".into()]).unwrap();
    set_enum_values(&conn, of(&sub), EnumKind::Tag, &["OPS".into()]).unwrap();
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
    set_enum_values(&conn, of(&t), EnumKind::Tag, &["OPS".into()]).unwrap();
    insert(&conn, of(&t), Target::attachment("a_n"), 0).unwrap();
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
    insert(&conn, of(&t), Target::attachment("a_n"), 0).unwrap();
    insert(&conn, of(&t), Target::attachment("a_n"), 0).unwrap();
    assert_eq!(count(&conn), 1);
}
