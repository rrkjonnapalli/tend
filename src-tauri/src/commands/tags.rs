//! Tags on notes, tasks and Library items.

use tauri::State;

use crate::enums::SourceType;
use crate::error::AppResult;
use crate::state::AppState;
use crate::tags;

#[tauri::command(async)]
pub fn tags_all(state: State<AppState>) -> AppResult<Vec<String>> {
    state.with(|db, _| tags::all(db))
}

fn set(
    state: State<AppState>,
    owner: SourceType,
    id: &str,
    tags: &[String],
) -> AppResult<Vec<String>> {
    state.with(|db, _| tags::set(db, owner, id, tags))
}

#[tauri::command(async)]
pub fn note_tags_set(
    state: State<AppState>,
    id: String,
    tags: Vec<String>,
) -> AppResult<Vec<String>> {
    set(state, SourceType::Note, &id, &tags)
}

#[tauri::command(async)]
pub fn task_tags_set(
    state: State<AppState>,
    id: String,
    tags: Vec<String>,
) -> AppResult<Vec<String>> {
    set(state, SourceType::Task, &id, &tags)
}

#[tauri::command(async)]
pub fn word_tags_set(
    state: State<AppState>,
    id: String,
    tags: Vec<String>,
) -> AppResult<Vec<String>> {
    set(state, SourceType::Word, &id, &tags)
}

#[tauri::command(async)]
pub fn snap_tags_set(
    state: State<AppState>,
    id: String,
    tags: Vec<String>,
) -> AppResult<Vec<String>> {
    set(state, SourceType::Snap, &id, &tags)
}

#[tauri::command(async)]
pub fn ref_tags_set(
    state: State<AppState>,
    id: String,
    tags: Vec<String>,
) -> AppResult<Vec<String>> {
    set(state, SourceType::Ref, &id, &tags)
}
