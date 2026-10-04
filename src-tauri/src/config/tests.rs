use super::*;

#[test]
fn round_trips_the_last_vault() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(last_vault(dir.path()), None);
    remember_vault(dir.path(), Path::new("/tmp/v")).unwrap();
    assert_eq!(last_vault(dir.path()), Some(PathBuf::from("/tmp/v")));
}
