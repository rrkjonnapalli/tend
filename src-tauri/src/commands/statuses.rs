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
    state.with(|db, _| {
        let change = statuses::Rename {
            value: &value,
            label: &label,
        };
        statuses::rename(db, &change)
    })
}

#[tauri::command(async)]
pub fn status_recolor(
    state: State<AppState>,
    value: String,
    color: String,
) -> AppResult<statuses::Status> {
    state.with(|db, _| {
        let change = statuses::Recolor {
            value: &value,
            color: &color,
        };
        statuses::recolor(db, &change)
    })
}

#[tauri::command(async)]
pub fn status_reorder(state: State<AppState>, values: Vec<String>) -> AppResult<()> {
    state.with(|db, _| statuses::reorder(db, &values))
}

#[tauri::command(async)]
pub fn status_delete(state: State<AppState>, value: String) -> AppResult<()> {
    state.with(|db, _| statuses::delete(db, &value))
}
