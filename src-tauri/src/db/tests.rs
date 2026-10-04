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
    assert_eq!(ver, schema_version().to_string());
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

#[test]
fn a_v1_database_migrates_to_the_latest_schema() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("app.sqlite");
    let v1 = Connection::open(&path).unwrap();
    v1.execute_batch(MIGRATIONS[0]).unwrap();
    v1.execute(
        "INSERT INTO meta (key, value) VALUES ('schema_version', '1')
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [],
    )
    .unwrap();
    drop(v1);

    let (conn, opened) = open(&path).unwrap();
    assert_eq!(
        opened,
        Opened::Migrated {
            from: 1,
            to: schema_version()
        }
    );
    let labels: i64 = conn
        .query_row("SELECT count(*) FROM task_labels", [], |r| r.get(0))
        .unwrap();
    assert_eq!(labels, 0);
}
