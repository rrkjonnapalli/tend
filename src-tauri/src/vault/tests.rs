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
    let expected = [StepStatus::Found, StepStatus::Created, StepStatus::Created];
    assert_eq!(statuses(&check), expected);
    assert_eq!(check.steps[0].detail, "0 notes");
    assert!(dir.path().join(".tend/attachments").is_dir());
    assert!(dir.path().join(".tend/tend.db").is_file());
    assert!(
        !dir.path().join("notes").exists(),
        "the vault itself is the notes root"
    );
}

#[test]
fn existing_vault_opens_and_counts() {
    let dir = tempfile::tempdir().unwrap();
    drop(open(dir.path(), false));
    fs::create_dir_all(dir.path().join("work")).unwrap();
    fs::write(dir.path().join("a.md"), "# A").unwrap();
    fs::write(dir.path().join("work/b.md"), "# B").unwrap();
    fs::write(dir.path().join("skip.txt"), "x").unwrap();
    fs::write(dir.path().join(".tend/hidden.md"), "x").unwrap();
    let (check, vault) = open(dir.path(), false);
    assert!(check.ok && vault.is_some());
    assert_eq!(statuses(&check), vec![StepStatus::Found; 3]);
    assert_eq!(check.steps[0].detail, "2 notes");
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
         INSERT INTO meta VALUES ('app_id','other');
         PRAGMA user_version = 1;",
    );
}

#[test]
fn higher_version_is_incompatible() {
    incompatible(
        "CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
         INSERT INTO meta VALUES ('app_id','tend');
         PRAGMA user_version = 99;",
    );
}

#[test]
fn missing_meta_is_incompatible() {
    incompatible("CREATE TABLE things (id INTEGER PRIMARY KEY);");
}

fn reopen_err(root: &Path) -> AppError {
    match reopen(root) {
        Err(e) => e,
        Ok(_) => panic!("expected {} to fail", root.display()),
    }
}

#[test]
fn reopen_explains_what_went_wrong() {
    let dir = tempfile::tempdir().unwrap();
    let gone = dir.path().join("gone");
    assert!(reopen_err(&gone).is(errors::VAULT_GONE));
    assert!(!gone.exists(), "a missing vault is never recreated");

    assert!(reopen(dir.path()).is_ok());
    let bad = tempfile::tempdir().unwrap();
    fs::create_dir_all(bad.path().join(".tend")).unwrap();
    fs::write(bad.path().join(".tend/tend.db"), "junk").unwrap();
    assert!(reopen_err(bad.path()).is(errors::INCOMPATIBLE_DB));
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
    assert!(err.is(errors::NO_PERMISSION), "{err}");
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

fn fix_names(vault: &Vault) -> bool {
    crate::notes::fix_allowed(&vault.db()).unwrap()
}

#[test]
fn a_new_empty_vault_fixes_names_without_asking() {
    let fresh = tempfile::tempdir().unwrap();
    fs::write(fresh.path().join(".DS_Store"), "").unwrap();
    let vault = open(fresh.path(), false).1.unwrap();
    assert!(fix_names(&vault));
}

#[test]
fn a_folder_with_things_in_it_asks_first_and_agreeing_allows_fixes() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("taxes.pdf"), "x").unwrap();
    fs::create_dir_all(dir.path().join("Work")).unwrap();
    fs::write(dir.path().join("Work/Plan.md"), "x").unwrap();
    fs::write(dir.path().join(".DS_Store"), "").unwrap();
    let (check, vault) = open(dir.path(), false);
    assert_eq!(
        check.confirm,
        Some(Confirm::NotAVault { items: 2, notes: 1 })
    );
    assert!(vault.is_none() && check.steps.is_empty());
    assert!(
        !dir.path().join(".tend").exists(),
        "nothing is written before the user agrees"
    );

    let (check, vault) = open(dir.path(), true);
    assert!(check.ok);
    let vault = vault.unwrap();
    assert!(fix_names(&vault));
    assert_eq!(inspect(dir.path()), None, "now it is a vault");
}

#[cfg(unix)]
#[test]
fn a_symlink_loop_does_not_hang_the_count() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("work")).unwrap();
    fs::write(dir.path().join("work/a.md"), "# A").unwrap();
    std::os::unix::fs::symlink(dir.path(), dir.path().join("work/loop")).unwrap();
    assert_eq!(count_files(dir.path(), Some(".md")), 1);
}
