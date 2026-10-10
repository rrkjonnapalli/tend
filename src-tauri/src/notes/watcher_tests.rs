use super::watcher::{rescan, start};
use crate::notes;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

/// A vault with one note, scanned, and a watcher that reports each rescan.
fn watched(
    text: &str,
) -> (
    tempfile::TempDir,
    Arc<crate::state::Vault>,
    mpsc::Receiver<()>,
    crate::state::Guard,
) {
    let tmp = tempfile::tempdir().unwrap();
    let (_, opened) = crate::vault::open(tmp.path(), false);
    let vault = Arc::new(opened.unwrap());
    std::fs::write(vault.notes_dir().join("a.md"), text).unwrap();
    rescan(&vault).unwrap();
    let (tx, rx) = mpsc::channel();
    let guard = start(vault.clone(), move |v| {
        rescan(v).unwrap();
        let _ = tx.send(());
    })
    .unwrap();
    std::thread::sleep(Duration::from_millis(100));
    (tmp, vault, rx, guard)
}

#[test]
fn outside_edit_of_an_open_note_reaches_the_index() {
    let (_tmp, vault, rx, _guard) = watched("# Before");
    let before = notes::tree(&vault.db()).unwrap().notes[0].clone();
    std::thread::sleep(Duration::from_millis(20));
    std::fs::write(vault.notes_dir().join("a.md"), "# After\n\nmore").unwrap();

    rx.recv_timeout(Duration::from_secs(3))
        .expect("watcher fired");
    let after = notes::row(&vault.db(), &before.id).unwrap();
    assert!(after.modified_at > before.modified_at);
    let doc = notes::read(&vault.db(), &vault.notes_dir(), &before.id).unwrap();
    assert_eq!(doc.text, "# After\n\nmore");
}

#[test]
fn writes_under_hidden_folders_never_trigger_a_rescan() {
    let (_tmp, vault, rx, _guard) = watched("# A");
    std::fs::write(vault.root.join(".tend/scratch.md"), "x").unwrap();
    assert!(
        rx.recv_timeout(Duration::from_millis(1000)).is_err(),
        "a write inside .tend/ must not wake the watcher"
    );
}

#[test]
fn outside_file_reaches_the_index_within_a_second() {
    let (_tmp, vault, rx, _guard) = watched("# Before");
    let started = Instant::now();
    std::fs::write(vault.notes_dir().join("b.md"), "# After").unwrap();

    rx.recv_timeout(Duration::from_secs(3))
        .expect("watcher fired");
    let count = notes::tree(&vault.db()).unwrap().notes.len();
    assert_eq!(count, 2);
    assert!(
        started.elapsed() < Duration::from_millis(1500),
        "took {:?}",
        started.elapsed()
    );
}

#[test]
fn tends_own_save_does_not_trigger_a_rescan() {
    let (_tmp, vault, rx, _guard) = watched("# A");
    let id = notes::tree(&vault.db()).unwrap().notes[0].id.clone();
    notes::write(&vault.db(), &vault.notes_dir(), &id, "# A\n\nsaved by Tend").unwrap();
    assert!(
        rx.recv_timeout(Duration::from_millis(1000)).is_err(),
        "the save already updated the index"
    );
}
