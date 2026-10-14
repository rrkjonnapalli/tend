use rusqlite_migration::M;

use super::*;
use crate::db::{self, columns, tables};

/// This app's migrations plus a made-up step 2, standing in for the next real one.
fn with_step_two() -> Migrations<'static> {
    let mut steps = db::STEPS.to_vec();
    steps.push(M::up(
        "ALTER TABLE people ADD COLUMN step_two TEXT NOT NULL DEFAULT '';
         UPDATE people SET step_two = lower(first_name);",
    ));
    Migrations::new(steps)
}

fn person(conn: &Connection, id: &str, name: &str) {
    conn.execute(
        "INSERT INTO people (id, first_name, created_at, updated_at) VALUES (?1, ?2, 0, 0)",
        rusqlite::params![id, name],
    )
    .unwrap();
}

#[test]
fn old_rows_become_what_the_migrated_database_holds() {
    let migrations = with_step_two();
    let mut conn = Connection::open_in_memory().unwrap();
    migrations.to_version(&mut conn, 1).unwrap();
    person(&conn, "01J00000000000000000000001", "Ann");
    let old = shards::snapshot(&conn).unwrap();

    migrations.to_latest(&mut conn).unwrap();
    let upgraded = upgrade(&old, 1, &migrations).unwrap();

    assert_eq!(upgraded, shards::snapshot(&conn).unwrap());
    let ann = &upgraded[tables::PEOPLE]["01J00000000000000000000001"];
    assert_eq!(ann["step_two"], "ann");
}

#[test]
fn a_default_row_the_old_snapshot_changed_keeps_its_change() {
    let migrations = with_step_two();
    let mut conn = Connection::open_in_memory().unwrap();
    migrations.to_version(&mut conn, 1).unwrap();
    let mut old = shards::snapshot(&conn).unwrap();
    let enums = old.get_mut(tables::ENUMS).unwrap();
    let id = enums.keys().next().unwrap().clone();
    enums.get_mut(&id).unwrap()[columns::LABEL] = "renamed".into();

    let upgraded = upgrade(&old, 1, &migrations).unwrap();

    assert_eq!(upgraded[tables::ENUMS][&id][columns::LABEL], "renamed");
    assert_eq!(upgraded[tables::ENUMS].len(), old[tables::ENUMS].len());
}
