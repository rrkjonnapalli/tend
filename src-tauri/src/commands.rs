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
use crate::{
    attachments, comments, config, feedback, notes, people, settings, statuses, tags, tasks,
    timers, vault,
};

/// Makes `vault` the open vault and starts what runs alongside it.
fn activate<R: Runtime>(
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
) -> AppResult<vault::Check> {
    let root = PathBuf::from(&path);
    let (check, opened) = vault::open(&root);
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
    error: Option<String>,
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
pub fn notes_rescan<R: Runtime>(
    app: AppHandle<R>,
    state: State<AppState>,
) -> AppResult<notes::Tree> {
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
pub fn note_create(
    state: State<AppState>,
    folder: String,
    name: String,
) -> AppResult<notes::NoteRow> {
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
pub fn note_tags_set(
    state: State<AppState>,
    id: String,
    tags: Vec<String>,
) -> AppResult<Vec<String>> {
    state.with(|db, _| tags::set(db, tags::Owner::Note, &id, &tags))
}

#[tauri::command]
pub fn task_tags_set(
    state: State<AppState>,
    id: String,
    tags: Vec<String>,
) -> AppResult<Vec<String>> {
    state.with(|db, _| tags::set(db, tags::Owner::Task, &id, &tags))
}

// ---- statuses

#[tauri::command]
pub fn statuses_list(state: State<AppState>) -> AppResult<Vec<statuses::Status>> {
    state.with(|db, _| statuses::list(db))
}

#[tauri::command]
pub fn status_add(
    state: State<AppState>,
    label: String,
    value: String,
    color: String,
) -> AppResult<statuses::Status> {
    state.with(|db, _| statuses::add(db, &label, &value, &color))
}

#[tauri::command]
pub fn status_rename(
    state: State<AppState>,
    value: String,
    label: String,
) -> AppResult<statuses::Status> {
    state.with(|db, _| statuses::rename(db, &value, &label))
}

#[tauri::command]
pub fn status_recolor(
    state: State<AppState>,
    value: String,
    color: String,
) -> AppResult<statuses::Status> {
    state.with(|db, _| statuses::recolor(db, &value, &color))
}

#[tauri::command]
pub fn status_reorder(state: State<AppState>, values: Vec<String>) -> AppResult<()> {
    state.with(|db, _| statuses::reorder(db, &values))
}

#[tauri::command]
pub fn status_delete(state: State<AppState>, value: String) -> AppResult<()> {
    state.with(|db, _| statuses::delete(db, &value))
}

#[tauri::command]
pub fn feedback_send<R: Runtime>(
    app: AppHandle<R>,
    state: State<AppState>,
    kind: String,
    body: String,
) -> AppResult<()> {
    let version = app.package_info().version.to_string();
    state.with(|db, _| feedback::add(db, &kind, &body, &version))
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
pub fn task_update(
    state: State<AppState>,
    id: String,
    patch: tasks::Patch,
) -> AppResult<tasks::Task> {
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
pub fn my_day_set(
    state: State<AppState>,
    id: String,
    day: String,
    included: Option<bool>,
) -> AppResult<()> {
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

// ---- comments

#[tauri::command]
pub fn comments_list(state: State<AppState>, task: String) -> AppResult<Vec<comments::Comment>> {
    state.with(|db, _| comments::list(db, &task))
}

#[tauri::command]
pub fn comment_add(state: State<AppState>, task: String, body: String) -> AppResult<()> {
    state.with(|db, _| comments::add(db, &task, &body))
}

#[tauri::command]
pub fn comment_edit(state: State<AppState>, id: String, body: String) -> AppResult<()> {
    state.with(|db, _| comments::edit(db, &id, &body))
}

// ---- attachments

#[tauri::command]
pub fn attachments_list(
    state: State<AppState>,
    owner: String,
    id: String,
) -> AppResult<Vec<attachments::Link>> {
    state.with(|db, _| attachments::list(db, &owner, &id))
}

/// Copies each file into the vault and links it. Stops at the first file that fails.
#[tauri::command]
pub fn attachments_add(
    state: State<AppState>,
    owner: String,
    id: String,
    paths: Vec<String>,
) -> AppResult<()> {
    state.with(|db, v| {
        let store = v.attachments_dir();
        paths.iter().try_for_each(|p| {
            attachments::add_file(db, &store, &owner, &id, std::path::Path::new(p))
        })
    })
}

#[tauri::command]
pub fn attachment_link_note(
    state: State<AppState>,
    owner: String,
    id: String,
    note: String,
) -> AppResult<()> {
    state.with(|db, _| attachments::link_note(db, &owner, &id, &note))
}

#[tauri::command]
pub fn attachment_unlink(state: State<AppState>, link: String) -> AppResult<()> {
    state.with(|db, _| attachments::unlink(db, &link))
}

fn opener_error(e: tauri_plugin_opener::Error) -> crate::error::AppError {
    std::io::Error::other(e.to_string()).into()
}

/// Opens the file with the OS default app.
#[tauri::command]
pub fn attachment_open(state: State<AppState>, attachment: String) -> AppResult<()> {
    let path = state.with(|db, v| attachments::file_path(db, &v.attachments_dir(), &attachment))?;
    tauri_plugin_opener::open_path(path, None::<&str>).map_err(opener_error)
}

#[tauri::command]
pub fn attachment_reveal(state: State<AppState>, attachment: String) -> AppResult<()> {
    let path = state.with(|db, v| attachments::file_path(db, &v.attachments_dir(), &attachment))?;
    tauri_plugin_opener::reveal_item_in_dir(path).map_err(opener_error)
}

// ---- people

#[tauri::command]
pub fn people_list(state: State<AppState>) -> AppResult<Vec<people::PersonRow>> {
    state.with(|db, _| people::list(db))
}

#[tauri::command]
pub fn person_create(state: State<AppState>, first: String, last: String) -> AppResult<String> {
    state.with(|db, _| people::create(db, &first, &last))
}

#[tauri::command]
pub fn person_get(state: State<AppState>, id: String) -> AppResult<people::Person> {
    state.with(|db, _| people::get(db, &id))
}

#[tauri::command]
pub fn person_update(
    state: State<AppState>,
    id: String,
    patch: people::Patch,
) -> AppResult<people::Person> {
    state.with(|db, _| people::update(db, &id, &patch))
}

#[tauri::command]
pub fn person_delete(state: State<AppState>, id: String) -> AppResult<()> {
    state.with(|db, _| people::delete(db, &id))
}

#[tauri::command]
pub fn contact_add(
    state: State<AppState>,
    person: String,
    kind: String,
    value: String,
    label: String,
) -> AppResult<()> {
    state.with(|db, _| people::contact_add(db, &person, &kind, &value, &label))
}

#[tauri::command]
pub fn contact_update(
    state: State<AppState>,
    id: String,
    value: String,
    label: String,
) -> AppResult<()> {
    state.with(|db, _| people::contact_update(db, &id, &value, &label))
}

#[tauri::command]
pub fn contact_delete(state: State<AppState>, id: String) -> AppResult<()> {
    state.with(|db, _| people::contact_delete(db, &id))
}

// ---- timers

#[tauri::command]
pub fn timers_list(state: State<AppState>) -> AppResult<Vec<timers::Timer>> {
    state.with(|db, _| timers::list(db))
}

#[tauri::command]
pub fn timer_start(state: State<AppState>, title: String, ms: i64) -> AppResult<timers::Timer> {
    state.with(|db, _| timers::start(db, &title, ms))
}

#[tauri::command]
pub fn timer_add_minute(state: State<AppState>, id: String) -> AppResult<()> {
    state.with(|db, _| timers::add_minute(db, &id))
}

#[tauri::command]
pub fn timer_cancel(state: State<AppState>, id: String) -> AppResult<()> {
    state.with(|db, _| timers::cancel(db, &id))
}

#[tauri::command]
pub fn timer_ring(state: State<AppState>, id: String) -> AppResult<()> {
    state.with(|db, _| timers::ring(db, &id))
}

#[tauri::command]
pub fn timers_finished_while_closed(state: State<AppState>) -> AppResult<Vec<timers::Timer>> {
    state.with(|db, _| timers::finished_while_closed(db))
}
