//! Tauri command handlers. Each one unwraps the open vault and calls into a domain module;
//! errors cross to the UI as `{ code, message }` (see error.rs).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use serde_json::Value;
use tauri::{AppHandle, Manager, State};

use crate::error::AppResult;
use crate::state::AppState;
use crate::{config, settings, vault};

fn config_dir(app: &AppHandle) -> AppResult<PathBuf> {
    app.path().app_config_dir().map_err(|e| std::io::Error::other(e.to_string()).into())
}

/// Makes `vault` the open vault and starts what runs alongside it.
fn activate(app: &AppHandle, state: &AppState, vault: crate::state::Vault) -> AppResult<()> {
    let vault = Arc::new(vault);
    state.set(vault.clone());
    crate::on_vault_open(app, state, &vault)
}

#[tauri::command]
pub fn vault_open(app: AppHandle, state: State<AppState>, path: String) -> AppResult<vault::Check> {
    let root = PathBuf::from(&path);
    let (check, opened) = vault::open(&root);
    if let Some(v) = opened {
        activate(&app, &state, v)?;
        config::remember_vault(&config_dir(&app)?, &root)?;
    }
    Ok(check)
}

/// Reopens the last vault on launch. Returns its path, or None when it's gone or broken.
#[tauri::command]
pub fn vault_restore(app: AppHandle, state: State<AppState>) -> AppResult<Option<String>> {
    let Some(root) = config::last_vault(&config_dir(&app)?) else { return Ok(None) };
    let (check, opened) = vault::open(&root);
    match opened {
        Some(v) => {
            activate(&app, &state, v)?;
            Ok(Some(check.path))
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub fn vault_current(state: State<AppState>) -> Option<String> {
    state.vault().ok().map(|v| v.root.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn settings_all(state: State<AppState>) -> AppResult<HashMap<String, Value>> {
    settings::all(&state.vault()?.db())
}

#[tauri::command]
pub fn settings_set(state: State<AppState>, key: String, value: Value) -> AppResult<()> {
    settings::set(&state.vault()?.db(), &key, &value)
}
