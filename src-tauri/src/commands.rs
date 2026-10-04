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
use crate::{config, notes, settings, statuses, tags, tasks, vault};

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

#[tauri::command]
pub fn task_tags_set(state: State<AppState>, id: String, tags: Vec<String>) -> AppResult<Vec<String>> {
    state.with(|db, _| tags::set(db, tags::Owner::Task, &id, &tags))
}

// ---- statuses

#[tauri::command]
pub fn statuses_list(state: State<AppState>) -> AppResult<Vec<statuses::Status>> {
    state.with(|db, _| statuses::list(db))
}

// ---- tasks

/// Everything the task lists need, with My day overrides for `day` (the UI's local today).
#[tauri::command]
pub fn tasks_data(state: State<AppState>, day: String) -> AppResult<tasks::TaskData> {
    state.with(|db, _| tasks::data(db, &day))
}

/// One task with the fields only its page shows.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskDetail {
    #[serde(flatten)]
    task: tasks::Task,
    description: String,
    tags: Vec<String>,
}

#[tauri::command]
pub fn task_get(state: State<AppState>, id: String) -> AppResult<TaskDetail> {
    state.with(|db, _| {
        Ok(TaskDetail {
            task: tasks::get(db, &id)?,
            description: tasks::description(db, &id)?,
            tags: tags::of(db, tags::Owner::Task, &id)?,
        })
    })
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewTask {
    path: String,
    title: String,
    parent_id: Option<String>,
    status: Option<String>,
    order_key: Option<String>,
}

#[tauri::command]
pub fn task_create(state: State<AppState>, task: NewTask) -> AppResult<tasks::Task> {
    state.with(|db, _| {
        tasks::create(
            db,
            &task.path,
            &task.title,
            task.parent_id.as_deref(),
            task.status.as_deref(),
            task.order_key.as_deref(),
        )
    })
}

#[tauri::command]
pub fn task_update(state: State<AppState>, id: String, patch: tasks::Patch) -> AppResult<tasks::Task> {
    state.with(|db, _| tasks::update(db, &id, &patch))
}

#[tauri::command]
pub fn task_move(state: State<AppState>, id: String, target: String) -> AppResult<tasks::Task> {
    state.with(|db, _| tasks::move_to(db, &id, &target))
}

#[tauri::command]
pub fn task_delete(state: State<AppState>, id: String) -> AppResult<()> {
    state.with(|db, _| tasks::delete(db, &id))
}

#[tauri::command]
pub fn my_day_set(state: State<AppState>, id: String, day: String, included: Option<bool>) -> AppResult<()> {
    state.with(|db, _| tasks::set_my_day(db, &id, &day, included))
}

#[tauri::command]
pub fn task_folder_create(state: State<AppState>, path: String) -> AppResult<()> {
    state.with(|db, _| tasks::folder_create(db, &path))
}

#[tauri::command]
pub fn task_folder_rename(state: State<AppState>, path: String, name: String) -> AppResult<String> {
    state.with(|db, _| tasks::folder_rename(db, &path, &name))
}

#[tauri::command]
pub fn task_folder_move(state: State<AppState>, path: String, target: String) -> AppResult<String> {
    state.with(|db, _| tasks::folder_move(db, &path, &target))
}

#[tauri::command]
pub fn task_folder_delete(state: State<AppState>, path: String) -> AppResult<()> {
    state.with(|db, _| tasks::folder_delete(db, &path))
}
