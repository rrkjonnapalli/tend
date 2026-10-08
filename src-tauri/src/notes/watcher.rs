//! Watches the vault (the notes root) and re-indexes after outside changes (Finder, other editors).

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use notify::RecursiveMode;
use notify_debouncer_full::{new_debouncer, DebounceEventResult};
use tauri::{AppHandle, Emitter, Runtime};

use crate::error::{errors, AppResult};
use serde_json::Value;

use crate::notes;
use crate::settings;
use crate::state::{Guard, Vault};

/// `path` is inside `root` under a hidden entry (or outside `root` altogether).
fn is_hidden(root: &Path, path: &Path) -> bool {
    let Ok(rel) = path.strip_prefix(root) else {
        return true;
    };
    rel.components()
        .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
}

pub const CHANGED: &str = "notes:changed";
const DEBOUNCE: Duration = Duration::from_millis(300);

/// Rescans the notes folder and records what it skipped. Names are fixed on disk only when the
/// vault allows it (`FIX_NAMES`).
pub fn rescan(vault: &Vault) -> AppResult<()> {
    let mut db = vault.db();
    let fix = settings::get(&db, notes::FIX_NAMES)? == Some(Value::Bool(true));
    let skipped = notes::scan(&mut db, &vault.notes_dir(), fix)?;
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

/// Starts watching the vault. `changed` runs after each debounced batch of outside changes to
/// notes and folders. Changes inside hidden folders (`.tend/`, where the database lives, `.git/`)
/// are ignored, so Tend's own writes never trigger a rescan.
pub fn start(vault: Arc<Vault>, changed: impl Fn(&Vault) + Send + 'static) -> AppResult<Guard> {
    let dir = vault.notes_dir();
    // Events may name the canonical path (/private/var for /var on macOS).
    let roots = [
        dir.clone(),
        dir.canonicalize().unwrap_or_else(|_| dir.clone()),
    ];
    let mut debouncer = new_debouncer(DEBOUNCE, None, move |res: DebounceEventResult| {
        let Ok(events) = res else {
            return;
        };
        let visible = events.iter().flat_map(|e| e.paths.iter()).any(|p| {
            roots
                .iter()
                .any(|root| p.starts_with(root) && !is_hidden(root, p))
        });
        if visible {
            changed(&vault);
        }
    })?;
    debouncer.watch(&dir, RecursiveMode::Recursive)?;
    Ok(Box::new(debouncer))
}
