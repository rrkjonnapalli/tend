use super::*;
use crate::db::open_memory;

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
