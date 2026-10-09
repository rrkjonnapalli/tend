use super::*;

#[test]
fn migrations_are_valid() {
    MIGRATIONS.validate().unwrap();
}

#[test]
fn creates_schema_and_meta() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tend.db");
    let (conn, opened) = open(&path).unwrap();
    assert_eq!(opened, Opened::Created);
    let app: String = conn
        .query_row("SELECT value FROM meta WHERE key='app_id'", [], |r| {
            r.get(0)
        })
        .unwrap();
    let ver: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(app, "tend");
    assert_eq!(ver, schema_version());
    let fk: i64 = conn
        .query_row("PRAGMA foreign_keys", [], |r| r.get(0))
        .unwrap();
    assert_eq!(fk, 1);
}

#[test]
fn reopens_existing_without_migrating() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tend.db");
    drop(open(&path).unwrap());
    let (_, opened) = open(&path).unwrap();
    assert_eq!(opened, Opened::Current);
}
