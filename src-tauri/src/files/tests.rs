use super::*;

#[test]
fn an_atomic_write_replaces_the_file_and_leaves_no_copy() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("plan.md");
    fs::write(&path, "old").unwrap();
    write_atomic(&path, b"new").unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "new");
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn a_failed_atomic_write_leaves_the_old_file_whole() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("plan.md");
    fs::write(&path, "old").unwrap();
    // A folder where the hidden copy goes makes the write fail before the file is touched.
    fs::create_dir(dir.path().join(".plan.md.tmp")).unwrap();
    assert!(write_atomic(&path, b"new").is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), "old");
}
