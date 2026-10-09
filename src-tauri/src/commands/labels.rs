//! Task labels: the list in Settings → Tasks, and the labels on a task.

use tauri::State;

use crate::error::AppResult;
use crate::labels;
use crate::state::AppState;

#[tauri::command(async)]
pub fn labels_list(state: State<AppState>) -> AppResult<Vec<labels::Label>> {
    state.with(|db, _| labels::list(db))
}

#[tauri::command(async)]
pub fn label_add(
    state: State<AppState>,
    name: String,
    value: String,
    color: String,
) -> AppResult<labels::Label> {
    state.with(|db, _| labels::add(db, &name, &value, &color))
}

#[tauri::command(async)]
pub fn label_rename(state: State<AppState>, id: String, name: String) -> AppResult<labels::Label> {
    state.with(|db, _| labels::rename(db, &id, &name))
}

#[tauri::command(async)]
pub fn label_recolor(
    state: State<AppState>,
    id: String,
    color: String,
) -> AppResult<labels::Label> {
    state.with(|db, _| labels::recolor(db, &id, &color))
}

#[tauri::command(async)]
pub fn label_reorder(state: State<AppState>, ids: Vec<String>) -> AppResult<()> {
    state.with(|db, _| labels::reorder(db, &ids))
}

/// How many tasks carry the label, for the delete confirmation.
#[tauri::command(async)]
pub fn label_usage(state: State<AppState>, id: String) -> AppResult<i64> {
    state.with(|db, _| labels::usage(db, &id))
}

#[tauri::command(async)]
pub fn label_delete(state: State<AppState>, id: String) -> AppResult<()> {
    state.with(|db, _| labels::delete(db, &id))
}

/// Replaces a task's labels; returns them in the list's order.
#[tauri::command(async)]
pub fn task_labels_set(
    state: State<AppState>,
    id: String,
    labels: Vec<String>,
) -> AppResult<Vec<String>> {
    state.with(|db, _| crate::labels::set_for_task(db, &id, &labels))
}
