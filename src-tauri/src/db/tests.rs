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

#[test]
fn a_transaction_holds_the_write_lock_from_its_start() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tend.db");
    let (mut app, _) = open(&path).unwrap();
    let other = Connection::open(&path).unwrap();
    other.busy_timeout(std::time::Duration::ZERO).unwrap();
    let tx = app.transaction().unwrap();
    // Nothing written yet, but the other connection already has to wait.
    let write = other.execute("UPDATE sync_state SET last_result = 'x' WHERE id = 1", []);
    assert!(write.is_err());
    drop(tx);
    assert!(other
        .execute("UPDATE sync_state SET last_result = 'x' WHERE id = 1", [])
        .is_ok());
}

#[test]
fn backup_copies_the_database_and_replaces_the_last_copy() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tend.db");
    let (conn, _) = open(&path).unwrap();
    std::fs::write(dir.path().join("tend.db.bak"), "old copy").unwrap();
    let copy = backup(&conn, &path).unwrap();
    assert_eq!(copy, dir.path().join("tend.db.bak"));
    assert_eq!(check(&copy).unwrap(), schema_version());
}
