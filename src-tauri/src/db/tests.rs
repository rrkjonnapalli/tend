use super::*;

#[test]
fn creates_schema_and_meta() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("app.sqlite");
    let (conn, opened) = open(&path).unwrap();
    assert_eq!(opened, Opened::Created);
    let app: String = conn
        .query_row("SELECT value FROM meta WHERE key='app_id'", [], |r| {
            r.get(0)
        })
        .unwrap();
    let ver: String = conn
        .query_row(
            "SELECT value FROM meta WHERE key='schema_version'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(app, "tend");
    assert_eq!(ver, "1");
    let fk: i64 = conn
        .query_row("PRAGMA foreign_keys", [], |r| r.get(0))
        .unwrap();
    assert_eq!(fk, 1);
}

#[test]
fn reopens_existing_without_migrating() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("app.sqlite");
    drop(open(&path).unwrap());
    let (_, opened) = open(&path).unwrap();
    assert_eq!(opened, Opened::Current);
}
