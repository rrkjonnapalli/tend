mod attachments;
mod commands;
mod comments;
mod config;
mod db;
mod error;
#[cfg(test)]
mod ipc_tests;
mod names;
mod notes;
mod order;
mod people;
mod settings;
mod state;
mod statuses;
mod tags;
mod tasks;
mod util;
mod vault;
mod watcher;

use std::sync::Arc;

use tauri::{AppHandle, Builder, Manager, Runtime};

use crate::config::ConfigDir;
use crate::error::AppResult;
use crate::state::{AppState, Vault};

/// Runs once a vault becomes the open vault: index the notes, then watch them.
fn on_vault_open<R: Runtime>(app: &AppHandle<R>, state: &AppState, vault: &Arc<Vault>) -> AppResult<()> {
    watcher::rescan(vault)?;
    let app = app.clone();
    let guard = watcher::start(vault.clone(), move |v| {
        // A failed rescan leaves the index as it was; the next event or reload retries.
        let _ = watcher::rescan_and_emit(&app, v);
    })?;
    state.set_guard(guard);
    Ok(())
}

/// App state and every command. Shared by the real app and the IPC tests.
pub fn with_commands<R: Runtime>(builder: Builder<R>) -> Builder<R> {
    builder.manage(AppState::default()).invoke_handler(tauri::generate_handler![
        commands::vault_open,
        commands::vault_restore,
        commands::vault_current,
        commands::settings_all,
        commands::settings_set,
        commands::skipped_items,
        commands::notes_tree,
        commands::notes_rescan,
        commands::note_read,
        commands::note_write,
        commands::note_create,
        commands::folder_create,
        commands::item_rename,
        commands::item_move,
        commands::item_delete,
        commands::tags_all,
        commands::note_tags_set,
        commands::statuses_list,
        commands::tasks_data,
        commands::task_get,
        commands::task_create,
        commands::task_update,
        commands::task_move,
        commands::task_delete,
        commands::task_tags_set,
        commands::my_day_set,
        commands::task_folder_create,
        commands::task_folder_rename,
        commands::task_folder_move,
        commands::task_folder_delete,
        commands::comments_list,
        commands::comment_add,
        commands::comment_edit,
        commands::attachments_list,
        commands::attachments_add,
        commands::attachment_link_note,
        commands::attachment_unlink,
        commands::attachment_open,
        commands::attachment_reveal,
        commands::people_list,
    ])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            app.manage(ConfigDir(app.path().app_config_dir()?));
            Ok(())
        });
    with_commands(builder)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
