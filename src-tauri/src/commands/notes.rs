//! Notes and note folders.

use tauri::{AppHandle, Runtime, State};

use crate::error::AppResult;
use crate::notes;
use crate::state::AppState;

#[tauri::command(async)]
pub fn skipped_items(state: State<AppState>) -> AppResult<Vec<notes::Skipped>> {
    Ok(state.vault()?.skipped())
}

/// Lets scans rename folders and notes whose names break the rule, then rescans. Returns what is
/// still skipped (names nothing valid can be made from).
#[tauri::command(async)]
pub fn notes_fix_names<R: Runtime>(
    app: AppHandle<R>,
    state: State<AppState>,
) -> AppResult<Vec<notes::Skipped>> {
    let vault = state.vault()?;
    crate::settings::set(
        &vault.db(),
        notes::FIX_NAMES,
        &serde_json::Value::Bool(true),
    )?;
    notes::watcher::rescan_and_emit(&app, &vault)?;
    Ok(vault.skipped())
}

#[tauri::command(async)]
pub fn notes_tree(state: State<AppState>) -> AppResult<notes::Tree> {
    state.with(|db, _| notes::tree(db))
}

#[tauri::command(async)]
pub fn notes_rescan<R: Runtime>(
    app: AppHandle<R>,
    state: State<AppState>,
) -> AppResult<notes::Tree> {
    crate::notes::watcher::rescan_and_emit(&app, &*state.vault()?)?;
    state.with(|db, _| notes::tree(db))
}

#[tauri::command(async)]
pub fn note_read(state: State<AppState>, id: String) -> AppResult<notes::NoteDoc> {
    state.with(|db, v| notes::read(db, &v.notes_dir(), &id))
}

#[tauri::command(async)]
pub fn note_write(state: State<AppState>, id: String, text: String) -> AppResult<notes::NoteRow> {
    state.with(|db, v| notes::write(db, &v.notes_dir(), &id, &text))
}

#[tauri::command(async)]
pub fn note_create(
    state: State<AppState>,
    folder: String,
    title: String,
) -> AppResult<notes::NoteRow> {
    state.with(|db, v| notes::create(db, &v.notes_dir(), &folder, &title))
}

#[tauri::command(async)]
/// Returns the created path (a taken name gets a suffix).
pub fn folder_create(state: State<AppState>, path: String) -> AppResult<String> {
    state.with(|db, v| notes::folder_create(db, &v.notes_dir(), &path))
}

/// An explorer item: a note by id or a folder by path.
#[derive(serde::Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Item {
    Note { id: String },
    Folder { path: String },
}

/// `name` is the new title for a note, the new folder name for a folder. Returns the item's id
/// (notes) or new path (folders).
#[tauri::command(async)]
pub fn item_rename(state: State<AppState>, item: Item, name: String) -> AppResult<String> {
    state.with(|db, v| match item {
        Item::Note { id } => notes::rename_note(db, &v.notes_dir(), &id, &name).map(|n| n.id),
        Item::Folder { path } => notes::rename_folder(db, &v.notes_dir(), &path, &name),
    })
}

#[tauri::command(async)]
pub fn item_move(state: State<AppState>, item: Item, target: String) -> AppResult<String> {
    state.with(|db, v| match item {
        Item::Note { id } => notes::move_note(db, &v.notes_dir(), &id, &target).map(|n| n.id),
        Item::Folder { path } => notes::move_folder(db, &v.notes_dir(), &path, &target),
    })
}

#[tauri::command(async)]
pub fn item_delete(state: State<AppState>, item: Item) -> AppResult<()> {
    state.with(|db, v| match item {
        Item::Note { id } => notes::delete_note(db, &v.notes_dir(), &id),
        Item::Folder { path } => notes::delete_folder(db, &v.notes_dir(), &path),
    })
}
