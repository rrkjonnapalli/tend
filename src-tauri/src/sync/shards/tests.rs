use super::*;
use crate::db::open_memory;

fn person(conn: &Connection, id: &str, name: &str) {
    conn.execute(
        "INSERT INTO people (id, first_name, created_at, updated_at) VALUES (?1, ?2, 0, 0)",
        rusqlite::params![id, name],
    )
    .unwrap();
}

/// A ULID made at `ms`, so its shard year is known.
fn ulid_at(ms: u64, n: u128) -> String {
    Ulid::from_parts(ms, n).to_string()
}

const MS_2025: u64 = 1_735_689_600_000; // 2025-01-01T00:00:00Z
const MS_2026: u64 = 1_767_225_600_000; // 2026-01-01T00:00:00Z

#[test]
fn years_follow_the_utc_calendar() {
    assert_eq!(utc_year(0), 1970);
    assert_eq!(utc_year(MS_2025 as i64 - 1), 2024);
    assert_eq!(utc_year(MS_2025 as i64), 2025);
    assert_eq!(utc_year(MS_2026 as i64 + 40 * MS_PER_DAY), 2026);
    assert_eq!(utc_year(-1), 1969);
}

#[test]
fn rows_go_to_year_shards_and_natural_ids_to_fixed() {
    let conn = open_memory();
    person(&conn, &ulid_at(MS_2025, 1), "Ann");
    person(&conn, &ulid_at(MS_2026, 1), "Bo");
    conn.execute(
        "INSERT INTO tend_enums (id, kind, value, label, created_at, updated_at)
         VALUES (?1, 'TAG', 'OPS', 'ops', 0, 0)",
        [ulid_at(MS_2026, 2)],
    )
    .unwrap();
    let files = render(&snapshot(&conn).unwrap()).unwrap();
    let names: Vec<String> = files.keys().map(|p| p.display().to_string()).collect();
    for expected in [
        "manifest.json",
        "people/2025-001.jsonl",
        "people/2026-001.jsonl",
        "tend_enums/fixed.jsonl",
        "tend_enums/2026-001.jsonl",
        "task_folders/fixed.jsonl",
        "note_folders/fixed.jsonl",
    ] {
        assert!(
            names.contains(&expected.to_string()),
            "{expected} in {names:?}"
        );
    }
    let fixed = &files[&PathBuf::from("tend_enums/fixed.jsonl")];
    assert!(fixed.contains(r#""id":"TASK_STATUS:DONE""#), "default rows");
    assert!(files[&PathBuf::from("tend_enums/2026-001.jsonl")].contains(r#""value":"OPS""#));
}

#[test]
fn local_columns_and_device_settings_stay_out() {
    let conn = open_memory();
    conn.execute(
        "INSERT INTO settings (id, key, value, updated_at) VALUES
           ('tasks.root_name', 'tasks.root_name', '\"work\"', 0),
           ('ui.nav_open', 'ui.nav_open', 'true', 0)",
        [],
    )
    .unwrap();
    let snap = snapshot(&conn).unwrap();
    let settings: Vec<&String> = snap["settings"].keys().collect();
    assert_eq!(settings, vec!["tasks.root_name"]);
    let root = &snap["note_folders"]["root"];
    assert!(!root.contains_key("missing_at"));
}

#[test]
fn a_year_is_cut_into_shards_of_ten_thousand_rows() {
    let mut conn = open_memory();
    let tx = conn.transaction().unwrap();
    for n in 0..=SHARD_ROWS as u128 {
        person(&tx, &ulid_at(MS_2026, n), "x");
    }
    tx.commit().unwrap();
    let files = render(&snapshot(&conn).unwrap()).unwrap();
    let count = |name: &str| files[&PathBuf::from(name)].lines().count();
    assert_eq!(count("people/2026-001.jsonl"), SHARD_ROWS);
    assert_eq!(count("people/2026-002.jsonl"), 1);
}

#[test]
fn files_parse_back_to_the_same_rows() {
    let conn = open_memory();
    person(&conn, &ulid_at(MS_2026, 7), "Ann");
    let snap = snapshot(&conn).unwrap();
    let back = parse(&render(&snap).unwrap()).unwrap();
    assert_eq!(back, snap);
}

#[test]
fn an_unknown_table_is_refused_as_too_new() {
    let mut files = BTreeMap::new();
    files.insert(PathBuf::from("projects/2026-001.jsonl"), String::new());
    assert!(parse(&files).unwrap_err().is(errors::SNAPSHOT_TOO_NEW));
}

#[test]
fn export_writes_only_what_changed_and_removes_what_is_gone() {
    let tmp = tempfile::tempdir().unwrap();
    let conn = open_memory();
    let ann = ulid_at(MS_2026, 1);
    person(&conn, &ann, "Ann");
    assert!(export(&conn, tmp.path()).unwrap() > 0);
    assert_eq!(
        export(&conn, tmp.path()).unwrap(),
        0,
        "same data, nothing written"
    );

    conn.execute(
        "UPDATE people SET first_name = 'Anna' WHERE id = ?1",
        [&ann],
    )
    .unwrap();
    assert_eq!(export(&conn, tmp.path()).unwrap(), 1);

    conn.execute("DELETE FROM people", []).unwrap();
    assert_eq!(export(&conn, tmp.path()).unwrap(), 1);
    assert!(!tmp.path().join(".tend/sync/people").exists());
}

#[test]
fn old_task_keys_are_rekeyed_as_migration_2_does() {
    let task = |key: &str| json!({ "order_key": key }).as_object().unwrap().clone();
    let mut snapshot: Snapshot = BTreeMap::new();
    let tasks = BTreeMap::from([
        ("t1".to_string(), task("VV")),
        ("t2".to_string(), task("U")),
        ("t3".to_string(), task("V")),
    ]);
    snapshot.insert("tasks", tasks);
    rekey_tasks(&mut snapshot);
    let key = |id: &str| {
        snapshot["tasks"][id]["order_key"]
            .as_str()
            .unwrap()
            .to_string()
    };
    assert_eq!([key("t2"), key("t3"), key("t1")], ["a0", "a1", "a2"]);
}

#[test]
fn rekeyed_keys_grow_a_digit_at_62_and_3844() {
    assert_eq!(order_key(61), "az");
    assert_eq!(order_key(62), "b10");
    assert_eq!(order_key(3844), "c100");
}
