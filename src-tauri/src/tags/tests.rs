use super::*;
use crate::db::open_memory;

#[test]
fn tags_are_slugs_stored_in_capitals() {
    let mut conn = open_memory();
    crate::notes::seed(&conn, "n", "n.md");
    let typed = ["UI Design", "tauri", " ", "ui-design", "C++", "!!"].map(String::from);
    let tags = set(&mut conn, Owner::Note, "n", &typed).unwrap();
    assert_eq!(tags, vec!["c", "tauri", "ui_design"]);
    assert_eq!(all(&conn).unwrap(), vec!["c", "tauri", "ui_design"]);
    let stored = rows::get(&conn, KIND, "UI_DESIGN").unwrap().unwrap();
    assert_eq!(stored.label, "ui_design");
    let tags = set(&mut conn, Owner::Note, "n", &["Tauri".into()]).unwrap();
    assert_eq!(tags, vec!["tauri"]);
    assert_eq!(all(&conn).unwrap().len(), 3, "unused tags stay in the pool");
}

#[test]
fn an_unchanged_tag_keeps_its_link() {
    let mut conn = open_memory();
    crate::notes::seed(&conn, "n", "n.md");
    set(&mut conn, Owner::Note, "n", &["ops".into()]).unwrap();
    let link = |c: &Connection| -> String {
        c.query_row("SELECT id FROM tend_links", [], |r| r.get(0))
            .unwrap()
    };
    let before = link(&conn);
    set(&mut conn, Owner::Note, "n", &["ops".into(), "q4".into()]).unwrap();
    let ids: Vec<String> = conn
        .prepare("SELECT id FROM tend_links ORDER BY target_value")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(ids[0], before);
}
