use super::*;

#[test]
fn round_trips_the_last_vault() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(last_vault(dir.path()), None);
    remember_vault(dir.path(), Path::new("/tmp/v")).unwrap();
    assert_eq!(last_vault(dir.path()), Some(PathBuf::from("/tmp/v")));
}

#[test]
fn sync_settings_are_kept_per_vault_next_to_the_last_vault() {
    let dir = tempfile::tempdir().unwrap();
    let vault = Path::new("/tmp/v");
    let sync = SyncConfig {
        url: "https://example.com/v.git".into(),
        branch: "main".into(),
        username: None,
        author_name: "Tend".into(),
        author_email: "tend@localhost".into(),
        interval_minutes: 15,
    };
    set_sync_config(dir.path(), vault, Some(sync.clone())).unwrap();
    remember_vault(dir.path(), vault).unwrap();
    assert_eq!(sync_config(dir.path(), vault), Some(sync));
    assert_eq!(sync_config(dir.path(), Path::new("/tmp/other")), None);
    set_sync_config(dir.path(), vault, None).unwrap();
    assert_eq!(sync_config(dir.path(), vault), None);
    assert_eq!(last_vault(dir.path()), Some(vault.to_path_buf()));
}

#[test]
fn readers_never_see_a_half_written_config() {
    let dir = tempfile::tempdir().unwrap();
    let vault = Path::new("/tmp/v");
    let sync = SyncConfig {
        url: "https://example.com/v.git".into(),
        branch: "main".into(),
        username: None,
        author_name: "Tend".into(),
        author_email: "tend@localhost".into(),
        interval_minutes: 15,
    };
    set_sync_config(dir.path(), vault, Some(sync.clone())).unwrap();
    let writer = {
        let dir = dir.path().to_path_buf();
        std::thread::spawn(move || {
            for _ in 0..300 {
                remember_vault(&dir, Path::new("/tmp/v")).unwrap();
            }
        })
    };
    for _ in 0..300 {
        assert_eq!(sync_config(dir.path(), vault), Some(sync.clone()));
    }
    writer.join().unwrap();
}
