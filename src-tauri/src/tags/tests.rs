use super::*;
use crate::db::open_memory;

#[test]
fn tags_are_slugs_and_their_own_ids() {
    let mut conn = open_memory();
    crate::notes::seed(&conn, "n", "n.md");
    let typed = ["UI Design", "tauri", " ", "ui-design", "C++", "!!"].map(String::from);
    let tags = set(&mut conn, Owner::Note, "n", &typed).unwrap();
    assert_eq!(tags, vec!["c", "tauri", "ui_design"]);
    assert_eq!(all(&conn).unwrap(), vec!["c", "tauri", "ui_design"]);
    let tags = set(&mut conn, Owner::Note, "n", &["Tauri".into()]).unwrap();
    assert_eq!(tags, vec!["tauri"]);
    assert_eq!(all(&conn).unwrap().len(), 3, "unused tags stay in the pool");
}

#[test]
fn the_schema_refuses_a_tag_outside_the_rule() {
    let conn = open_memory();
    for bad in ["UI", "a b", "_a", "a__b"] {
        let made = conn.execute(
            "INSERT INTO tags (id, created_at, updated_at) VALUES (?1, 0, 0)",
            [bad],
        );
        assert!(made.is_err(), "{bad}");
    }
}
