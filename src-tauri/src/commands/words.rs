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

/// What the dictionary `source` says about `title`; `None` when it doesn't have it.
#[tauri::command(async)]
pub fn word_lookup(
    title: String,
    source: words::DictionarySource,
) -> AppResult<Option<words::Dictionary>> {
    words::lookup(&title, source)
}

#[tauri::command(async)]
pub fn word_kind_add(
    state: State<AppState>,
    label: String,
    value: String,
) -> AppResult<words::Kind> {
    state.with(|db, _| {
        let kind = words::KindNamed {
            value: &value,
            label: &label,
        };
        words::kind_add(db, &kind)
    })
}

#[tauri::command(async)]
pub fn word_kind_rename(
    state: State<AppState>,
    value: String,
    label: String,
) -> AppResult<words::Kind> {
    state.with(|db, _| {
        let kind = words::KindNamed {
            value: &value,
            label: &label,
        };
        words::kind_rename(db, &kind)
    })
}

#[tauri::command(async)]
pub fn word_kind_reorder(state: State<AppState>, values: Vec<String>) -> AppResult<()> {
    state.with(|db, _| words::kind_reorder(db, &values))
}

#[tauri::command(async)]
pub fn word_kind_delete(state: State<AppState>, value: String) -> AppResult<()> {
    state.with(|db, _| words::kind_delete(db, &value))
}
