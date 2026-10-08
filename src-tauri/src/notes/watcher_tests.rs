use super::watcher::{rescan, start};
use crate::notes;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

#[test]
fn outside_file_reaches_the_index_within_a_second() {
    let tmp = tempfile::tempdir().unwrap();
    let (_, opened) = crate::vault::open(tmp.path(), false);
    let vault = Arc::new(opened.unwrap());
    std::fs::write(vault.notes_dir().join("a.md"), "# Before").unwrap();
    rescan(&vault).unwrap();

    let (tx, rx) = mpsc::channel();
    let _guard = start(vault.clone(), move |v| {
        rescan(v).unwrap();
        let _ = tx.send(());
    })
    .unwrap();
    std::thread::sleep(Duration::from_millis(100));
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
