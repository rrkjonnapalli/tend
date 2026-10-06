//! Attachments: stored files and note links.

use tauri::State;

use crate::attachments;
use crate::error::AppResult;
use crate::state::AppState;

#[tauri::command(async)]
pub fn attachments_list(
    state: State<AppState>,
    owner: String,
    id: String,
) -> AppResult<Vec<attachments::Link>> {
    state.with(|db, _| attachments::list(db, &owner, &id))
}

/// Copies each file into the vault and links it. Stops at the first file that fails.
#[tauri::command(async)]
pub fn attachments_add(
    state: State<AppState>,
    owner: String,
    id: String,
    paths: Vec<String>,
) -> AppResult<()> {
    state.with(|db, v| {
        paths.iter().try_for_each(|p| {
            attachments::add_file(db, &v.root, &owner, &id, std::path::Path::new(p))
        })
    })
}

#[tauri::command(async)]
pub fn attachment_link_note(
    state: State<AppState>,
    owner: String,
    id: String,
    note: String,
) -> AppResult<()> {
    state.with(|db, _| attachments::link_note(db, &owner, &id, &note))
}

#[tauri::command(async)]
pub fn attachment_unlink(state: State<AppState>, link: String) -> AppResult<()> {
    state.with(|db, _| attachments::unlink(db, &link))
}

fn opener_error(e: tauri_plugin_opener::Error) -> crate::error::AppError {
    crate::error::errors::OPEN_FAILED.with(e.to_string())
}

/// Opens the file with the OS default app.
#[tauri::command]
pub fn attachment_open(state: State<AppState>, attachment: String) -> AppResult<()> {
    let path = state.with(|db, v| attachments::file_path(db, &v.root, &attachment))?;
    tauri_plugin_opener::open_path(path, None::<&str>).map_err(opener_error)
}

#[tauri::command]
pub fn attachment_reveal(state: State<AppState>, attachment: String) -> AppResult<()> {
    let path = state.with(|db, v| attachments::file_path(db, &v.root, &attachment))?;
    tauri_plugin_opener::reveal_item_in_dir(path).map_err(opener_error)
}
