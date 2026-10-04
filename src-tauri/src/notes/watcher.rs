//! Watches `notes/` and re-indexes after outside changes (Finder, other editors).

use std::sync::Arc;
use std::time::Duration;

use notify::RecursiveMode;
use notify_debouncer_full::{new_debouncer, DebounceEventResult};
use tauri::{AppHandle, Emitter, Runtime};

use crate::error::{errors, AppResult};
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
pub fn rescan_and_emit<R: Runtime>(app: &AppHandle<R>, vault: &Vault) -> AppResult<()> {
    rescan(vault)?;
    app.emit(CHANGED, ())
        .map_err(|e| errors::WATCH.with(e.to_string()))?;
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
