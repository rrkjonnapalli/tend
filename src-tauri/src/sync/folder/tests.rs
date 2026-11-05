use super::*;

fn of(url: &str) -> AppResult<PathBuf> {
    phone_vault(Path::new("/data"), url)
}

#[test]
fn a_remote_becomes_host_owner_repo_lowercased() {
    let want = PathBuf::from("/data/vaults/github.com/rniverse/notes");
    assert_eq!(of("https://github.com/Rniverse/Notes.git").unwrap(), want);
    assert_eq!(of("https://github.com/rniverse/notes").unwrap(), want);
    assert_eq!(of("  https://github.com/rniverse/notes/ ").unwrap(), want);
    assert_eq!(of("https://x-access-token@github.com/rniverse/notes.git").unwrap(), want);
}

#[test]
fn the_host_keeps_two_hosts_apart() {
    assert_ne!(of("https://github.com/a/b").unwrap(), of("https://gitlab.com/a/b").unwrap());
}

#[test]
fn anything_but_host_owner_repo_is_refused() {
    for bad in [
        "",
        "git@github.com:a/b.git",
        "https://github.com/a",
        "https://github.com/a/b/c",
        "https://github.com/../b",
        "https://github.com/a/..",
        "https://github.com//b",
        "ftp://github.com/a/b",
    ] {
        let err = of(bad).unwrap_err();
        assert_eq!(err.key(), errors::SYNC_URL.key, "{bad}");
    }
}
