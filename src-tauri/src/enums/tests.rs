use super::*;
use crate::db::open_memory;
use rusqlite::Connection;
use serde_json::json;

/// Each source table removes its links when a row goes (`<table>_drop_links`), naming its own type.
#[test]
fn every_source_type_has_its_clean_up_trigger() {
    let conn = open_memory();
    for source in SourceType::ALL {
        let sql: String = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type = 'trigger' AND tbl_name = ?1
                   AND sql LIKE '%tend_links%'",
                [source.table()],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| panic!("no clean-up trigger on {}", source.table()));
        let names_type = format!("source_type = '{}'", source.as_str());
        assert!(sql.contains(&names_type), "{sql}");
        assert_eq!(SourceType::parse(source.as_str()), Some(source));
    }
}

#[test]
fn rows_keep_their_order_and_meta() {
    let conn = open_memory();
    let statuses = rows::list(&conn, EnumKind::TaskStatus).unwrap();
    let values: Vec<&str> = statuses.iter().map(|r| r.value.as_str()).collect();
    assert_eq!(values, ["NOT_STARTED", "IN_PROGRESS", "DONE"]);
    assert_eq!(statuses[2].text(rows::COLOR), "#1e7a55");
    assert!(statuses.iter().all(|r| r.is_default));
}

#[test]
fn default_rows_cant_be_deleted() {
    let conn = open_memory();
    assert!(rows::delete(&conn, EnumKind::TaskStatus, "DONE").is_err());
}

#[test]
fn papers_and_reference_kinds_read_back_and_unknowns_fall_back() {
    for paper in Paper::ALL {
        assert_eq!(Paper::parse(paper.as_str()), paper);
        let json = serde_json::to_value(paper).unwrap();
        assert_eq!(json, paper.as_str());
    }
    assert_eq!(Paper::parse("FOLDED"), Paper::Scrap);
    assert_eq!(RefKind::parse("LINK"), RefKind::Link);
    assert_eq!(RefKind::parse("BOOK"), RefKind::Text);
    assert_eq!(serde_json::to_value(RefKind::Text).unwrap(), "TEXT");
}

fn label(conn: &Connection, value: &str, meta: serde_json::Value) {
    let new = rows::NewRow {
        kind: EnumKind::TaskLabel,
        value,
        label: value,
        meta: meta.as_object().unwrap().clone(),
    };
    assert!(rows::insert(conn, &new).unwrap().is_some());
}

fn key(value: &str) -> rows::Key<'_> {
    rows::Key {
        kind: EnumKind::TaskLabel,
        value,
    }
}

fn updated_at(conn: &Connection, value: &str) -> i64 {
    conn.query_row(
        "SELECT updated_at FROM tend_enums WHERE kind = 'TASK_LABEL' AND value = ?1",
        [value],
        |r| r.get(0),
    )
    .unwrap()
}

#[test]
fn a_row_without_a_position_sorts_last_and_isnt_counted_for_the_next_one() {
    let conn = open_memory();
    let kind = EnumKind::TaskLabel;
    assert_eq!(rows::next_position(&conn, kind, None).unwrap(), 0);
    label(&conn, "A", json!({}));
    assert_eq!(rows::next_position(&conn, kind, None).unwrap(), 0);
    label(&conn, "Z", json!({ "position": 0 }));
    label(&conn, "M", json!({ "position": 4 }));
    let values: Vec<String> = rows::list(&conn, kind)
        .unwrap()
        .into_iter()
        .map(|r| r.value)
        .collect();
    assert_eq!(values, ["Z", "M", "A"]);
    assert_eq!(rows::next_position(&conn, kind, None).unwrap(), 5);
    assert_eq!(rows::next_position(&conn, kind, Some("M")).unwrap(), 1);
}

#[test]
fn an_unchanged_meta_value_keeps_the_rows_time() {
    let conn = open_memory();
    label(&conn, "A", json!({ "color": "#111111", "position": 3 }));
    conn.execute("UPDATE tend_enums SET updated_at = 1 WHERE value = 'A'", [])
        .unwrap();
    let entry = |key, to| rows::MetaEntry { key, to };
    let same = rows::set_meta(&conn, key("A"), &entry(rows::COLOR, json!("#111111"))).unwrap();
    assert_eq!(same.text(rows::COLOR), "#111111");
    rows::set_meta(&conn, key("A"), &entry(rows::POSITION, json!(3))).unwrap();
    assert_eq!(
        updated_at(&conn, "A"),
        1,
        "nothing changed, nothing written"
    );

    let changed = rows::set_meta(&conn, key("A"), &entry(rows::COLOR, json!("#222222"))).unwrap();
    assert_eq!(changed.text(rows::COLOR), "#222222");
    assert_eq!(changed.position(), 3, "the other keys stay");
    assert!(updated_at(&conn, "A") > 1);
}

#[test]
fn a_write_to_a_row_that_isnt_there_is_not_found() {
    let mut conn = open_memory();
    let entry = rows::MetaEntry {
        key: rows::COLOR,
        to: json!("#111111"),
    };
    let missing = [
        rows::set_meta(&conn, key("NOPE"), &entry).map(|_| ()),
        rows::set_label(&conn, key("NOPE"), "Nope").map(|_| ()),
        rows::delete(&conn, EnumKind::TaskLabel, "NOPE"),
        crate::db::in_transaction(&mut conn, |tx| {
            rows::set_status(tx, key("NOPE"), EnumStatus::Blocked)
        }),
    ];
    for result in missing {
        let e = result.unwrap_err();
        assert!(e.is(crate::error::errors::NOT_FOUND));
        assert_eq!(e.to_string(), "Label was not found");
    }
}

#[test]
fn a_row_with_broken_meta_is_left_out_of_its_list() {
    let conn = open_memory();
    label(&conn, "GOOD", json!({ "position": 0 }));
    conn.execute(
        "INSERT INTO tend_enums (id, kind, value, label, meta, created_at, updated_at)
         VALUES ('bad', 'TASK_LABEL', 'BAD', 'Bad', 'not json', 0, 0)",
        [],
    )
    .unwrap();
    let values: Vec<String> = rows::list(&conn, EnumKind::TaskLabel)
        .unwrap()
        .into_iter()
        .map(|r| r.value)
        .collect();
    assert_eq!(values, vec!["GOOD"]);
}
