use super::*;
use rusqlite::Connection;

fn statuses(c: &Check) -> Vec<StepStatus> {
    c.steps.iter().map(|s| s.status).collect()
}

#[test]
fn empty_folder_creates_everything() {
    let dir = tempfile::tempdir().unwrap();
    let (check, vault) = open(dir.path(), false);
    assert!(check.ok);
    assert!(vault.is_some());
    assert_eq!(statuses(&check), vec![StepStatus::Created; 3]);
    assert!(dir.path().join("notes").is_dir());
    assert!(dir.path().join(".attachments").is_dir());
    assert!(dir.path().join(".tend/tend.db").is_file());
}

#[test]
fn existing_vault_opens_and_counts() {
    let dir = tempfile::tempdir().unwrap();
    drop(open(dir.path(), false));
    fs::create_dir_all(dir.path().join("notes/work")).unwrap();
    fs::write(dir.path().join("notes/a.md"), "# A").unwrap();
    fs::write(dir.path().join("notes/work/b.md"), "# B").unwrap();
    fs::write(dir.path().join("notes/skip.txt"), "x").unwrap();
    let (check, vault) = open(dir.path(), false);
    assert!(check.ok && vault.is_some());
    assert_eq!(statuses(&check), vec![StepStatus::Found; 3]);
    assert_eq!(check.steps[0].detail, "found · 2 notes");
    assert_eq!(check.steps[2].detail, "schema matches");
}

fn incompatible(setup: &str) {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join(".tend")).unwrap();
    let path = dir.path().join(".tend/tend.db");
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch(setup).unwrap();
    drop(conn);
    let before = fs::read(&path).unwrap();

    let (check, vault) = open(dir.path(), false);
    assert!(!check.ok);
    assert!(vault.is_none());
    assert_eq!(check.steps[2].status, StepStatus::Error);
    assert_eq!(check.steps[2].detail, "incompatible schema");
    assert_eq!(
        fs::read(&path).unwrap(),
        before,
        "incompatible db must not change"
    );
}

#[test]
fn wrong_app_id_is_incompatible() {
    incompatible(
        "CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
         INSERT INTO meta VALUES ('app_id','other'),('schema_version','1');",
    );
}

#[test]
fn higher_version_is_incompatible() {
    incompatible(
        "CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
         INSERT INTO meta VALUES ('app_id','tend'),('schema_version','99');",
    );
}

#[test]
fn missing_meta_is_incompatible() {
    incompatible("CREATE TABLE things (id INTEGER PRIMARY KEY);");
}

fn reopen_err(root: &Path) -> String {
    match reopen(root) {
        Err(e) => e,
        Ok(_) => panic!("expected {} to fail", root.display()),
    }
}

#[test]
fn reopen_explains_what_went_wrong() {
    let dir = tempfile::tempdir().unwrap();
    let gone = dir.path().join("gone");
    assert!(reopen_err(&gone).contains("isn’t there"));
    assert!(!gone.exists(), "a missing vault is never recreated");

    assert!(reopen(dir.path()).is_ok());
    let bad = tempfile::tempdir().unwrap();
    fs::create_dir_all(bad.path().join(".tend")).unwrap();
    fs::write(bad.path().join(".tend/tend.db"), "junk").unwrap();
    assert_eq!(
        reopen_err(bad.path()),
        "An incompatible database exists in .tend/tend.db."
    );
}

#[cfg(unix)]
#[test]
fn reopen_reports_missing_permission() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let locked = dir.path().join("locked");
    fs::create_dir_all(&locked).unwrap();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
    let err = reopen_err(&locked);
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(err.contains("permission"), "{err}");
}

#[test]
fn non_sqlite_file_is_incompatible() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join(".tend")).unwrap();
    fs::write(dir.path().join(".tend/tend.db"), "not a database").unwrap();
    let (check, vault) = open(dir.path(), false);
    assert!(!check.ok && vault.is_none());
    assert_eq!(
        fs::read_to_string(dir.path().join(".tend/tend.db")).unwrap(),
        "not a database"
    );
}

fn fix_names(vault: &Vault) -> Option<serde_json::Value> {
    crate::settings::get(&vault.db(), crate::notes::FIX_NAMES).unwrap()
}

#[test]
fn only_a_new_empty_vault_fixes_names_without_asking() {
    let fresh = tempfile::tempdir().unwrap();
    fs::create_dir_all(fresh.path().join("notes")).unwrap();
    fs::write(fresh.path().join("notes/.DS_Store"), "").unwrap();
    let vault = open(fresh.path(), false).1.unwrap();
    assert_eq!(fix_names(&vault), Some(serde_json::Value::Bool(true)));

    let existing = tempfile::tempdir().unwrap();
    fs::create_dir_all(existing.path().join("notes")).unwrap();
    fs::write(existing.path().join("notes/Café.md"), "x").unwrap();
    let vault = open(existing.path(), false).1.unwrap();
    assert_eq!(fix_names(&vault), None, "files of its own: ask first");
}

#[test]
fn a_folder_with_other_things_asks_before_becoming_a_vault() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("taxes.pdf"), "x").unwrap();
    fs::create_dir_all(dir.path().join("Photos")).unwrap();
    fs::write(dir.path().join(".DS_Store"), "").unwrap();
    let (check, vault) = open(dir.path(), false);
    assert_eq!(check.confirm, Some(Confirm::NotAVault { items: 2 }));
    assert!(vault.is_none() && check.steps.is_empty());
    assert!(
        !dir.path().join("notes").exists(),
        "nothing is written before the user agrees"
    );

    let (check, vault) = open(dir.path(), true);
    assert!(check.ok && vault.is_some());
    assert!(dir.path().join("notes").is_dir());
}

#[test]
fn a_vault_whose_notes_folder_is_gone_asks_first() {
    let dir = tempfile::tempdir().unwrap();
    let vault = open(dir.path(), false).1.unwrap();
    fs::write(dir.path().join("notes/a.md"), "x").unwrap();
    crate::notes::scan(&mut vault.db(), &vault.notes_dir(), false).unwrap();
    drop(vault);
    fs::rename(dir.path().join("notes"), dir.path().join("moved")).unwrap();

    let (check, opened) = open(dir.path(), false);
    assert_eq!(check.confirm, Some(Confirm::NotesMissing { notes: 1 }));
    assert!(opened.is_none() && !dir.path().join("notes").exists());
    assert!(reopen_err(dir.path()).contains("notes/ is missing (1 note)"));
    assert!(open(dir.path(), true).1.is_some(), "start without them");
}

#[test]
fn notes_folder_alone_opens_without_asking() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("notes")).unwrap();
    fs::write(dir.path().join("notes/a.md"), "x").unwrap();
    fs::write(dir.path().join("README.txt"), "x").unwrap();
    assert_eq!(inspect(dir.path()), None);
}
