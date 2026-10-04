use super::*;
use crate::db::open_memory;

#[test]
fn saves_feedback_rows() {
    let conn = open_memory();
    add(&conn, "idea", "  dark mode  ", "0.1.0").unwrap();
    assert!(add(&conn, "rant", "x", "0.1.0").is_err());
    assert!(add(&conn, "bug", " ", "0.1.0").is_err());
    let (kind, body, v): (String, String, String) = conn
        .query_row("SELECT kind, body, app_version FROM feedback", [], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })
        .unwrap();
    assert_eq!(
        (kind.as_str(), body.as_str(), v.as_str()),
        ("idea", "dark mode", "0.1.0")
    );
}
