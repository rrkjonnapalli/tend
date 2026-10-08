use super::*;

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
    let path = dir.path().join("tend.db");
    drop(open(&path).unwrap());
    let (_, opened) = open(&path).unwrap();
    assert_eq!(opened, Opened::Current);
}

#[test]
fn deletes_leave_tombstones_and_activity() {
    let conn = super::open_memory();
    conn.execute(
        "INSERT INTO tags (id, name, created_at, updated_at) VALUES ('t1', 'ops', 5, 5)",
        [],
    )
    .unwrap();
    conn.execute("DELETE FROM tags WHERE id = 't1'", [])
        .unwrap();
    let tomb: (String, String) = conn
        .query_row("SELECT table_name, row_id FROM deletions", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap();
    assert_eq!(tomb, ("tags".into(), "t1".into()));
    let actions: Vec<String> = conn
        .prepare("SELECT action FROM activity WHERE entity_id = 't1' ORDER BY id")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(actions, vec!["insert", "delete"]);
}
