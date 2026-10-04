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
