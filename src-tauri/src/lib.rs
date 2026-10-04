mod commands;
mod config;
mod db;
mod error;
mod settings;
mod state;
mod util;
mod vault;

use tauri::AppHandle;

use crate::error::AppResult;
use crate::state::{AppState, Vault};

/// Runs once a vault becomes the open vault.
fn on_vault_open(_app: &AppHandle, _state: &AppState, _vault: &Vault) -> AppResult<()> {
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::vault_open,
            commands::vault_restore,
            commands::vault_current,
            commands::settings_all,
            commands::settings_set,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
