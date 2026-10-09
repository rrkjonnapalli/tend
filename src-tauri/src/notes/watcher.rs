//! Watches the vault (the notes root) and re-indexes after outside changes (Finder, other editors).

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use notify::RecursiveMode;
use notify_debouncer_full::{new_debouncer, DebounceEventResult};
use tauri::{AppHandle, Emitter, Runtime};

use crate::error::{errors, AppResult};
use crate::names;
use crate::notes;
use crate::state::{Guard, Vault};

pub const CHANGED: &str = "notes:changed";
const DEBOUNCE: Duration = Duration::from_millis(300);

/// `path` is inside `root` under a hidden entry (or outside `root` altogether).
fn is_hidden(root: &Path, path: &Path) -> bool {
    let Ok(rel) = path.strip_prefix(root) else {
        return true;
    };
    rel.components()
        .any(|c| names::is_hidden(&c.as_os_str().to_string_lossy()))
}

/// Rescans the notes folder and records what it skipped. Names are fixed on disk only when the
/// vault allows it.
pub fn rescan(vault: &Vault) -> AppResult<()> {
    let mut db = vault.db();
    let fix = notes::fix_allowed(&db)?;
    let skipped = notes::scan(&mut db, &vault.notes_dir(), fix)?;
    vault.set_skipped(skipped);
    Ok(())
}

/// Lets this vault's scans fix names, then rescans so they are fixed now.
pub fn fix_names<R: Runtime>(app: &AppHandle<R>, vault: &Vault) -> AppResult<()> {
    notes::allow_fix(&vault.db())?;
    rescan_and_emit(app, vault)
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
        let visible = events
            .iter()
            .flat_map(|e| e.paths.iter())
            .any(|p| roots.iter().any(|root| !is_hidden(root, p)));
        if visible {
            changed(&vault);
        }
    })?;
    debouncer.watch(&dir, RecursiveMode::Recursive)?;
    Ok(Box::new(debouncer))
}
