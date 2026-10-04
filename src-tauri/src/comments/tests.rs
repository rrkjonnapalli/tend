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
    add(&conn, &t.id, "first").unwrap();
    add(&conn, &t.id, "second").unwrap();
    assert!(add(&conn, &t.id, "  ").is_err());
    let all = list(&conn, &t.id).unwrap();
    assert_eq!(
        all.iter().map(|c| c.body.as_str()).collect::<Vec<_>>(),
        vec!["second", "first"]
    );
    edit(&conn, &all[1].id, "first, edited").unwrap();
    let again = list(&conn, &t.id).unwrap();
    assert_eq!(again[1].body, "first, edited");
    assert!(again[1].edited_at.is_some());
}
