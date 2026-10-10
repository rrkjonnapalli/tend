//! Opening the vault.

use std::path::PathBuf;
use std::sync::Arc;

use tauri::{AppHandle, Runtime, State};

use crate::config::ConfigDir;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::{config, vault};

/// Makes `vault` the open vault and starts what runs alongside it.
pub(crate) fn activate<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    vault: crate::state::Vault,
) -> AppResult<()> {
    let vault = Arc::new(vault);
    state.set(vault.clone());
    crate::on_vault_open(app, state, &vault)
}

/// Async so it runs off the main thread: macOS may ask the user for folder access first,
/// and the window must keep painting while it waits.
#[tauri::command]
pub async fn vault_open<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    cfg: State<'_, ConfigDir>,
    path: String,
    confirmed: Option<bool>,
) -> AppResult<vault::Check> {
    let root = PathBuf::from(&path);
    let (check, opened) = vault::open(&root, confirmed.unwrap_or(false));
    if let Some(v) = opened {
        activate(&app, &state, v)?;
        config::remember_vault(&cfg.0, &root)?;
    }
    Ok(check)
}

/// What launch found: no remembered vault (`path` empty), the vault opened, or why it couldn't.
#[derive(serde::Serialize)]
pub struct Restore {
    path: Option<String>,
    error: Option<AppError>,
}

/// Reopens the last vault on launch. Async for the same reason as `vault_open`.
#[tauri::command]
pub async fn vault_restore<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    cfg: State<'_, ConfigDir>,
) -> AppResult<Restore> {
    let Some(root) = config::last_vault(&cfg.0) else {
        return Ok(Restore {
            path: None,
            error: None,
        });
    };
    let path = Some(root.to_string_lossy().into_owned());
    match vault::reopen(&root) {
        Ok(v) => {
            activate(&app, &state, v)?;
            Ok(Restore { path, error: None })
        }
        Err(error) => Ok(Restore {
            path,
            error: Some(error),
        }),
    }
}
