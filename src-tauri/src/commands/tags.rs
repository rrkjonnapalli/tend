//! Tags on notes and tasks.

use tauri::State;

use crate::error::AppResult;
use crate::state::AppState;
use crate::tags;

#[tauri::command(async)]
pub fn tags_all(state: State<AppState>) -> AppResult<Vec<String>> {
    state.with(|db, _| tags::all(db))
}

#[tauri::command(async)]
pub fn note_tags_set(
    state: State<AppState>,
    id: String,
    tags: Vec<String>,
) -> AppResult<Vec<String>> {
    state.with(|db, _| tags::set(db, tags::Owner::Note, &id, &tags))
}

#[tauri::command(async)]
pub fn task_tags_set(
    state: State<AppState>,
    id: String,
    tags: Vec<String>,
) -> AppResult<Vec<String>> {
    state.with(|db, _| tags::set(db, tags::Owner::Task, &id, &tags))
}
