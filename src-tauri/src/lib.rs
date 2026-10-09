mod attachments;
mod colors;
mod commands;
mod comments;
mod config;
mod db;
mod error;
mod feedback;
mod files;
#[cfg(test)]
mod ipc_tests;
mod labels;
#[cfg(target_os = "macos")]
mod macos;
mod names;
mod notes;
mod order;
mod people;
mod settings;
mod state;
mod statuses;
mod sync;
mod tags;
mod tasks;
mod timers;
mod util;
mod vault;

use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Builder, Manager, Runtime};

use crate::config::ConfigDir;
use crate::error::AppResult;
use crate::state::{AppState, Vault};

/// Runs once a vault becomes the open vault: index the notes, then watch them.
fn on_vault_open<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    vault: &Arc<Vault>,
) -> AppResult<()> {
    notes::watcher::rescan(vault)?;
    let watcher_app = app.clone();
    let guard = notes::watcher::start(vault.clone(), move |v| {
        // A failed rescan leaves the index as it was; the next event or reload retries.
        let _ = notes::watcher::rescan_and_emit(&watcher_app, v);
    })?;
    state.set_guard(guard);
    start_sync(app, state, vault);
    Ok(())
}

/// Starts the sync runner when the vault is connected; its first run comes right after the scan.
pub(crate) fn start_sync<R: Runtime>(app: &AppHandle<R>, state: &AppState, vault: &Vault) {
    let Some(cfg) = app.try_state::<ConfigDir>() else {
        return;
    };
    if config::sync_config(&cfg.0, &vault.root).is_none() {
        return;
    }
    let place = sync::runner::Place {
        root: vault.root.clone(),
        config_dir: cfg.0.clone(),
    };
    state.set_runner(Some(sync::runner::Runner::start(app.clone(), place)));
}

/// How long quitting waits for the last sync.
const CLOSE_SYNC_LIMIT: Duration = Duration::from_secs(10);

/// On quit, a connected vault syncs once more (at most `CLOSE_SYNC_LIMIT`), then the app exits.
fn on_exit<R: Runtime>(app: &AppHandle<R>, api: &tauri::ExitRequestApi) {
    let state = app.state::<AppState>();
    let connected = state.with_runner(|_| ()).is_some();
    if !connected || !state.start_closing() {
        return;
    }
    api.prevent_exit();
    let app = app.clone();
    std::thread::spawn(move || {
        app.state::<AppState>()
            .with_runner(|r| r.close(CLOSE_SYNC_LIMIT));
        app.exit(0);
    });
}

/// App state and every command. Shared by the real app and the IPC tests.
pub fn with_commands<R: Runtime>(builder: Builder<R>) -> Builder<R> {
    builder
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::vault::vault_open,
            commands::vault::vault_restore,
            commands::sync::sync_status,
            commands::sync::sync_connect,
            commands::sync::sync_restore,
            commands::sync::sync_now,
            commands::sync::sync_set_interval,
            commands::sync::sync_token_set,
            commands::sync::sync_disconnect,
            commands::app::settings_all,
            commands::app::settings_set,
            commands::notes::skipped_items,
            commands::notes::notes_tree,
            commands::notes::notes_rescan,
            commands::notes::notes_fix_names,
            commands::notes::note_read,
            commands::notes::note_write,
            commands::notes::note_create,
            commands::notes::folder_create,
            commands::notes::item_rename,
            commands::notes::item_move,
            commands::notes::item_delete,
            commands::tags::tags_all,
            commands::tags::note_tags_set,
            commands::labels::labels_list,
            commands::labels::label_add,
            commands::labels::label_rename,
            commands::labels::label_recolor,
            commands::labels::label_reorder,
            commands::labels::label_usage,
            commands::labels::label_delete,
            commands::labels::task_labels_set,
            commands::statuses::statuses_list,
            commands::statuses::status_add,
            commands::statuses::status_rename,
            commands::statuses::status_recolor,
            commands::statuses::status_reorder,
            commands::statuses::status_delete,
            commands::app::feedback_send,
            commands::timers::timers_list,
            commands::timers::timer_start,
            commands::timers::timer_add_minute,
            commands::timers::timer_cancel,
            commands::timers::timer_ring,
            commands::timers::timers_finished_while_closed,
            commands::tasks::tasks_data,
            commands::tasks::task_get,
            commands::tasks::task_create,
            commands::tasks::task_update,
            commands::tasks::task_move,
            commands::tasks::task_delete,
            commands::tags::task_tags_set,
            commands::tasks::my_day_set,
            commands::tasks::task_folder_create,
            commands::tasks::task_folder_rename,
            commands::tasks::task_folder_move,
            commands::tasks::task_folder_delete,
            commands::comments::comments_list,
            commands::comments::comment_add,
            commands::comments::comment_edit,
            commands::attachments::attachments_list,
            commands::attachments::attachments_add,
            commands::attachments::attachment_link_note,
            commands::attachments::attachment_unlink,
            commands::attachments::attachment_open,
            commands::attachments::attachment_reveal,
            commands::people::people_list,
            commands::people::person_create,
            commands::people::person_get,
            commands::people::person_update,
            commands::people::person_delete,
            commands::people::contact_add,
            commands::people::contact_update,
            commands::people::contact_delete,
        ])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            macos::show_notifications_in_foreground();
            app.manage(ConfigDir(app.path().app_config_dir()?));
            Ok(())
        });
    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_window_state::Builder::default().build());
    let app = with_commands(builder)
        .build(tauri::generate_context!())
        .expect("error while building tauri application");
    app.run(|handle, event| {
        if let tauri::RunEvent::ExitRequested { api, .. } = &event {
            on_exit(handle, api);
        }
    });
}
