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
fn migration_2_gives_every_task_a_short_key_in_the_same_order() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tend.db");
    {
        let mut conn = Connection::open(&path).unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        MIGRATIONS.to_version(&mut conn, 1).unwrap();
        conn.execute_batch(
            "INSERT INTO tasks (id, path, title, status, order_key, created_at, updated_at) VALUES
               ('t1', '', 'First', 'NOT_STARTED', '0V', 0, 0),
               ('t2', '', 'Second', 'NOT_STARTED', 'V', 0, 0),
               ('t3', '', 'Third', 'NOT_STARTED', 'zzzzzzzzzzzV', 0, 0);",
        )
        .unwrap();
    }
    let (conn, opened) = open(&path).unwrap();
    assert_eq!(
        opened,
        Opened::Migrated {
            from: 1,
            to: schema_version()
        }
    );
    let keys: Vec<(String, String)> = conn
        .prepare("SELECT id, order_key FROM tasks ORDER BY order_key")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    let want =
        [("t1", "a0"), ("t2", "a1"), ("t3", "a2")].map(|(a, b)| (a.to_string(), b.to_string()));
    assert_eq!(keys, want);
}
