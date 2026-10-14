use super::*;

/// A repository the user made: one commit of an empty tree.
fn user_repo(root: &Path) -> Repository {
    let repo = Repository::init(root).unwrap();
    {
        let tree = repo
            .find_tree(repo.index().unwrap().write_tree().unwrap())
            .unwrap();
        let who = Signature::now("Someone", "someone@example.com").unwrap();
        repo.commit(Some("HEAD"), &who, &who, "Mine", &tree, &[])
            .unwrap();
    }
    repo
}

#[test]
fn a_folder_without_git_gets_a_marked_repo_that_disconnect_removes() {
    let tmp = tempfile::tempdir().unwrap();
    Repo::take(tmp.path()).unwrap();
    assert!(is_marked(&Repository::open(tmp.path()).unwrap()));
    remove_if_managed(tmp.path()).unwrap();
    assert!(!tmp.path().join(GIT_DIR).exists());
    assert!(
        tmp.path().join(GITIGNORE).exists(),
        "the .gitignore isn't Git data"
    );
}

#[test]
fn an_empty_repo_without_a_remote_becomes_tends() {
    let tmp = tempfile::tempdir().unwrap();
    Repository::init(tmp.path()).unwrap();
    Repo::take(tmp.path()).unwrap();
    assert!(is_marked(&Repository::open(tmp.path()).unwrap()));
}

#[test]
fn a_repo_with_a_commit_or_a_remote_is_refused_and_kept() {
    let committed = tempfile::tempdir().unwrap();
    user_repo(committed.path());
    let linked = tempfile::tempdir().unwrap();
    Repository::init(linked.path())
        .unwrap()
        .remote("origin", "https://example.com/notes.git")
        .unwrap();
    for root in [committed.path(), linked.path()] {
        let refused = Repo::take(root).err().unwrap();
        assert!(refused.is(errors::SYNC_REPO_EXISTS));
        assert!(!root.join(GITIGNORE).exists(), "nothing is written first");
        remove_if_managed(root).unwrap();
        assert!(root.join(GIT_DIR).exists());
    }
}
