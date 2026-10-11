//! Words and word kinds.

use tauri::State;

use crate::error::AppResult;
use crate::state::AppState;
use crate::words;

#[tauri::command(async)]
pub fn words_list(state: State<AppState>) -> AppResult<Vec<words::Word>> {
    state.with(|db, _| words::list(db))
}

#[tauri::command(async)]
pub fn word_kinds_list(state: State<AppState>) -> AppResult<Vec<words::Kind>> {
    state.with(|db, _| words::kinds(db))
}

#[tauri::command(async)]
pub fn word_create(state: State<AppState>, word: words::NewWord) -> AppResult<words::Word> {
    state.with(|db, _| words::create(db, &word))
}

#[tauri::command(async)]
pub fn word_update(
    state: State<AppState>,
    id: String,
    patch: words::Patch,
) -> AppResult<words::Word> {
    state.with(|db, _| words::update(db, &id, &patch))
}

#[tauri::command(async)]
pub fn word_delete(state: State<AppState>, id: String) -> AppResult<()> {
    state.with(|db, _| words::delete(db, &id))
}
