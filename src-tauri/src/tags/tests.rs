use super::*;
use crate::db::open_memory;
use crate::error::errors;

/// The note the tests tag.
fn note() -> Source<'static> {
    Source {
        kind: SourceType::Note,
        id: "n",
    }
}

#[test]
fn tags_are_slugs_stored_in_capitals() {
    let mut conn = open_memory();
    crate::notes::seed(&conn, "n", "n.md");
    let typed = ["UI Design", "tauri", " ", "ui-design", "C++", "!!"].map(String::from);
    let tags = set(&mut conn, note(), &typed).unwrap();
    assert_eq!(tags, vec!["c", "tauri", "ui_design"]);
    assert_eq!(all(&conn).unwrap(), vec!["c", "tauri", "ui_design"]);
    let stored = rows::get(&conn, KIND, "UI_DESIGN").unwrap().unwrap();
    assert_eq!(stored.label, "ui_design");
    let tags = set(&mut conn, note(), &["Tauri".into()]).unwrap();
    assert_eq!(tags, vec!["tauri"]);
    assert_eq!(all(&conn).unwrap().len(), 3, "unused tags stay in the pool");
}

#[test]
fn an_unchanged_tag_keeps_its_link() {
    let mut conn = open_memory();
    crate::notes::seed(&conn, "n", "n.md");
    set(&mut conn, note(), &["ops".into()]).unwrap();
    let link = |c: &Connection| -> String {
        c.query_row("SELECT id FROM tend_links", [], |r| r.get(0))
            .unwrap()
    };
    let before = link(&conn);
    set(&mut conn, note(), &["ops".into(), "q4".into()]).unwrap();
    let ids: Vec<String> = conn
        .prepare("SELECT id FROM tend_links ORDER BY target_value")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(ids[0], before);
}

fn links(conn: &Connection) -> i64 {
    conn.query_row("SELECT count(*) FROM tend_links", [], |r| r.get(0))
        .unwrap()
}

#[test]
fn a_blocked_tag_comes_off_everything_and_cant_be_added_until_unblocked() {
    let mut conn = open_memory();
    crate::notes::seed(&conn, "n", "n.md");
    set(&mut conn, note(), &["ops".into(), "q4".into()]).unwrap();
    set_status(&mut conn, "ops", EnumStatus::Blocked).unwrap();
    assert_eq!(
        links(&conn),
        1,
        "the blocked tag's link went with the block"
    );
    assert_eq!(of(&conn, SourceType::Note, "n").unwrap(), vec!["q4"]);
    assert_eq!(
        all(&conn).unwrap(),
        vec!["q4"],
        "blocked tags aren't offered"
    );

    let refused = set(&mut conn, note(), &["q4".into(), "Ops".into()]).unwrap_err();
    assert!(refused.is(errors::TAG_BLOCKED));
    assert_eq!(of(&conn, SourceType::Note, "n").unwrap(), vec!["q4"]);

    set_status(&mut conn, "ops", EnumStatus::Active).unwrap();
    assert_eq!(links(&conn), 1, "unblocking brings no link back");
    let tags = set(&mut conn, note(), &["q4".into(), "ops".into()]).unwrap();
    assert_eq!(tags, vec!["ops", "q4"]);
}

#[test]
fn the_overview_lists_every_tag_with_its_status_and_uses() {
    let mut conn = open_memory();
    crate::notes::seed(&conn, "n", "n.md");
    crate::notes::seed(&conn, "m", "m.md");
    set(&mut conn, note(), &["ops".into(), "q4".into()]).unwrap();
    let other = Source {
        kind: SourceType::Note,
        id: "m",
    };
    set(&mut conn, other, &["ops".into(), "alpha".into()]).unwrap();
    set_status(&mut conn, "q4", EnumStatus::Blocked).unwrap();
    let row = |name: &str, status, uses| TagRow {
        name: name.into(),
        status,
        uses,
    };
    assert_eq!(
        overview(&conn).unwrap(),
        vec![
            row("alpha", EnumStatus::Active, 1),
            row("ops", EnumStatus::Active, 2),
            row("q4", EnumStatus::Blocked, 0),
        ]
    );
    let json = serde_json::to_value(row("q4", EnumStatus::Blocked, 0)).unwrap();
    assert_eq!(
        json,
        serde_json::json!({ "name": "q4", "status": "BLOCKED", "uses": 0 })
    );
}

#[test]
fn blocking_a_tag_that_isnt_there_is_not_found() {
    let mut conn = open_memory();
    let missing = set_status(&mut conn, "nope", EnumStatus::Blocked).unwrap_err();
    assert!(missing.is(errors::NOT_FOUND));
    assert_eq!(missing.to_string(), "Tag was not found");
}
