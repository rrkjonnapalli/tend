//! Watches `notes/` and re-indexes after outside changes (Finder, other editors).

use std::sync::Arc;
use std::time::Duration;

use notify::RecursiveMode;
use notify_debouncer_full::{new_debouncer, DebounceEventResult};
use tauri::{AppHandle, Emitter};

use crate::error::AppResult;
use crate::notes;
use crate::state::{Guard, Vault};

pub const CHANGED: &str = "notes:changed";
const DEBOUNCE: Duration = Duration::from_millis(300);

/// Rescans the notes folder and records what it skipped.
pub fn rescan(vault: &Vault) -> AppResult<()> {
    let skipped = notes::scan(&mut vault.db(), &vault.notes_dir())?;
    vault.set_skipped(skipped);
    Ok(())
}

/// Rescans, then tells the UI.
pub fn rescan_and_emit(app: &AppHandle, vault: &Vault) -> AppResult<()> {
    rescan(vault)?;
    app.emit(CHANGED, ()).map_err(|e| std::io::Error::other(e.to_string()))?;
    Ok(())
}

/// Starts watching. `changed` runs after each debounced batch of outside changes.
pub fn start(vault: Arc<Vault>, changed: impl Fn(&Vault) + Send + 'static) -> AppResult<Guard> {
    let dir = vault.notes_dir();
    let mut debouncer = new_debouncer(DEBOUNCE, None, move |res: DebounceEventResult| {
        if matches!(&res, Ok(events) if !events.is_empty()) {
            changed(&vault);
        }
    })?;
    debouncer.watch(&dir, RecursiveMode::Recursive)?;
    Ok(Box::new(debouncer))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Instant;

    #[test]
    fn outside_edit_reaches_the_index_within_a_second() {
        let tmp = tempfile::tempdir().unwrap();
        let (_, opened) = crate::vault::open(tmp.path());
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
        std::fs::write(vault.notes_dir().join("a.md"), "# After").unwrap();

        rx.recv_timeout(Duration::from_secs(3)).expect("watcher fired");
        let title = notes::tree(&vault.db()).unwrap().notes[0].title.clone();
        assert_eq!(title, "After");
        assert!(started.elapsed() < Duration::from_millis(1500), "took {:?}", started.elapsed());
    }
}
