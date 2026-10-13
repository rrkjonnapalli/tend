use super::*;
use crate::db::open_memory;

#[test]
fn add_list_edit() {
    let mut conn = open_memory();
    let t = crate::tasks::create(
        &mut conn,
        &crate::tasks::NewTask {
            title: "T".into(),
            ..Default::default()
        },
    )
    .unwrap();
    add(
        &conn,
        &NewComment {
            task: &t.id,
            body: "first",
        },
    )
    .unwrap();
    add(
        &conn,
        &NewComment {
            task: &t.id,
            body: "second",
        },
    )
    .unwrap();
    assert!(add(
        &conn,
        &NewComment {
            task: &t.id,
            body: "  "
        }
    )
    .is_err());
    let all = list(&conn, &t.id).unwrap();
    assert_eq!(
        all.iter().map(|c| c.body.as_str()).collect::<Vec<_>>(),
        vec!["second", "first"]
    );
    edit(
        &conn,
        &Edit {
            id: &all[1].id,
            body: "first, edited",
        },
    )
    .unwrap();
    let again = list(&conn, &t.id).unwrap();
    assert_eq!(again[1].body, "first, edited");
    assert!(again[1].edited_at.is_some());
}

#[test]
fn editing_a_comment_that_isnt_there_is_not_found() {
    let conn = open_memory();
    let change = Edit {
        id: "nope",
        body: "x",
    };
    assert!(edit(&conn, &change).unwrap_err().is(errors::NOT_FOUND));
}
