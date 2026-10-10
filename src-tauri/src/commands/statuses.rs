//! Task statuses.

use tauri::State;

use crate::error::AppResult;
use crate::state::AppState;
use crate::statuses;

#[tauri::command(async)]
pub fn statuses_list(state: State<AppState>) -> AppResult<Vec<statuses::Status>> {
    state.with(|db, _| statuses::list(db))
}

#[tauri::command(async)]
pub fn status_add(
    state: State<AppState>,
    label: String,
    value: String,
    color: String,
) -> AppResult<statuses::Status> {
    state.with(|db, _| {
        statuses::add(
            db,
            &statuses::NewStatus {
                label: &label,
                value: &value,
                color: &color,
            },
        )
    })
}

#[tauri::command(async)]
pub fn status_rename(
    state: State<AppState>,
    value: String,
    label: String,
) -> AppResult<statuses::Status> {
    state.with(|db, _| statuses::rename(db, &value, &label))
}

#[tauri::command(async)]
pub fn status_recolor(
    state: State<AppState>,
    value: String,
    color: String,
) -> AppResult<statuses::Status> {
    state.with(|db, _| statuses::recolor(db, &value, &color))
}

#[tauri::command(async)]
pub fn status_reorder(state: State<AppState>, values: Vec<String>) -> AppResult<()> {
    state.with(|db, _| statuses::reorder(db, &values))
}

#[tauri::command(async)]
pub fn status_delete(state: State<AppState>, value: String) -> AppResult<()> {
    state.with(|db, _| statuses::delete(db, &value))
}
