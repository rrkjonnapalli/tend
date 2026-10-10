use super::*;
use crate::db::open_memory;
use serde_json::json;

#[test]
fn set_and_overwrite() {
    let conn = open_memory();
    set(&conn, "ui.nav_open", &json!(false)).unwrap();
    set(&conn, "ui.nav_open", &json!(true)).unwrap();
    let all = all(&conn).unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all["ui.nav_open"], json!(true));
}

#[test]
fn the_file_size_limit_is_1_to_90_mb_and_shared() {
    let conn = crate::db::open_memory();
    assert_eq!(crate::files::max_bytes(&conn).unwrap(), 30 * 1024 * 1024);
    for bad in [json!(0), json!(91), json!(2.5), json!("40")] {
        assert!(set(&conn, crate::files::LIMIT_KEY, &bad)
            .unwrap_err()
            .is(crate::error::errors::FILE_LIMIT));
    }
    set(&conn, crate::files::LIMIT_KEY, &json!(50)).unwrap();
    assert_eq!(crate::files::max_bytes(&conn).unwrap(), 50 * 1024 * 1024);
    assert!(SHARED_KEYS.contains(&crate::files::LIMIT_KEY));
}
