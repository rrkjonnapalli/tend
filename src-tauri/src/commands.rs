//! Tauri command handlers. Each one unwraps the open vault and calls into a domain module;
//! errors cross to the UI as `{ code, message }` (see error.rs).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use serde_json::Value;
use tauri::{AppHandle, Runtime, State};

use crate::config::ConfigDir;
use crate::error::AppResult;
use crate::state::AppState;
use crate::{config, notes, settings, tags, vault};

/// Makes `vault` the open vault and starts what runs alongside it.
fn activate<R: Runtime>(app: &AppHandle<R>, state: &AppState, vault: crate::state::Vault) -> AppResult<()> {
    let vault = Arc::new(vault);
    state.set(vault.clone());
    crate::on_vault_open(app, state, &vault)
}

#[tauri::command]
pub fn vault_open<R: Runtime>(
    app: AppHandle<R>,
    state: State<AppState>,
    cfg: State<ConfigDir>,
    path: String,
) -> AppResult<vault::Check> {
    let root = PathBuf::from(&path);
    let (check, opened) = vault::open(&root);
    if let Some(v) = opened {
        activate(&app, &state, v)?;
        config::remember_vault(&cfg.0, &root)?;
    }
    Ok(check)
}

/// Reopens the last vault on launch. Returns its path, or None when it's gone or broken.
#[tauri::command]
pub fn vault_restore<R: Runtime>(
    app: AppHandle<R>,
    state: State<AppState>,
    cfg: State<ConfigDir>,
) -> AppResult<Option<String>> {
    let Some(root) = config::last_vault(&cfg.0) else { return Ok(None) };
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
    state.with(|db, _| settings::all(db))
}

#[tauri::command]
pub fn settings_set(state: State<AppState>, key: String, value: Value) -> AppResult<()> {
    state.with(|db, _| settings::set(db, &key, &value))
}

#[tauri::command]
pub fn skipped_items(state: State<AppState>) -> AppResult<Vec<String>> {
    Ok(state.vault()?.skipped())
}

// ---- notes

#[tauri::command]
pub fn notes_tree(state: State<AppState>) -> AppResult<notes::Tree> {
    state.with(|db, _| notes::tree(db))
}

#[tauri::command]
pub fn notes_rescan<R: Runtime>(app: AppHandle<R>, state: State<AppState>) -> AppResult<notes::Tree> {
    crate::watcher::rescan_and_emit(&app, &*state.vault()?)?;
    state.with(|db, _| notes::tree(db))
}

#[tauri::command]
pub fn note_read(state: State<AppState>, id: String) -> AppResult<notes::NoteDoc> {
    state.with(|db, v| notes::read(db, &v.notes_dir(), &id))
}

#[tauri::command]
pub fn note_write(state: State<AppState>, id: String, text: String) -> AppResult<notes::NoteRow> {
    state.with(|db, v| notes::write(db, &v.notes_dir(), &id, &text))
}

#[tauri::command]
pub fn note_create(state: State<AppState>, folder: String, name: String) -> AppResult<notes::NoteRow> {
    state.with(|db, v| notes::create(db, &v.notes_dir(), &folder, &name))
}

#[tauri::command]
pub fn folder_create(state: State<AppState>, path: String) -> AppResult<()> {
    state.with(|db, v| notes::folder_create(db, &v.notes_dir(), &path))
}

/// An explorer item: a note by id or a folder by path.
#[derive(serde::Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Item {
    Note { id: String },
    Folder { path: String },
}

/// Returns the item's new id (notes) or path (folders).
#[tauri::command]
pub fn item_rename(state: State<AppState>, item: Item, name: String) -> AppResult<String> {
    state.with(|db, v| match item {
        Item::Note { id } => notes::rename_note(db, &v.notes_dir(), &id, &name).map(|n| n.id),
        Item::Folder { path } => notes::rename_folder(db, &v.notes_dir(), &path, &name),
    })
}

#[tauri::command]
pub fn item_move(state: State<AppState>, item: Item, target: String) -> AppResult<String> {
    state.with(|db, v| match item {
        Item::Note { id } => notes::move_note(db, &v.notes_dir(), &id, &target).map(|n| n.id),
        Item::Folder { path } => notes::move_folder(db, &v.notes_dir(), &path, &target),
    })
}

#[tauri::command]
pub fn item_delete(state: State<AppState>, item: Item) -> AppResult<()> {
    state.with(|db, v| match item {
        Item::Note { id } => notes::delete_note(db, &v.notes_dir(), &id),
        Item::Folder { path } => notes::delete_folder(db, &v.notes_dir(), &path),
    })
}

// ---- tags

#[tauri::command]
pub fn tags_all(state: State<AppState>) -> AppResult<Vec<String>> {
    state.with(|db, _| tags::all(db))
}

#[tauri::command]
pub fn note_tags_set(state: State<AppState>, id: String, tags: Vec<String>) -> AppResult<Vec<String>> {
    state.with(|db, _| tags::set(db, tags::Owner::Note, &id, &tags))
}
