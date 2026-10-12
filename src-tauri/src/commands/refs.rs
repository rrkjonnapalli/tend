//! References.

use tauri::State;

use crate::error::AppResult;
use crate::refs;
use crate::state::AppState;

#[tauri::command(async)]
pub fn refs_list(state: State<AppState>) -> AppResult<Vec<refs::Ref>> {
    state.with(|db, _| refs::list(db))
}

#[tauri::command(async)]
pub fn ref_create(state: State<AppState>, reference: refs::NewRef) -> AppResult<refs::Ref> {
    state.with(|db, _| refs::create(db, &reference))
}

#[tauri::command(async)]
pub fn ref_update(state: State<AppState>, id: String, patch: refs::Patch) -> AppResult<refs::Ref> {
    state.with(|db, _| refs::update(db, &id, &patch))
}

#[tauri::command(async)]
pub fn ref_delete(state: State<AppState>, id: String) -> AppResult<()> {
    state.with(|db, _| refs::delete(db, &id))
}

/// The page title of a link, fetched now; `None` when the page has none.
#[tauri::command(async)]
pub fn ref_title(url: String) -> AppResult<Option<String>> {
    refs::fetch_title(&url)
}
