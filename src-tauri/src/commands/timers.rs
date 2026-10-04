//! Timers.

use tauri::State;

use crate::error::AppResult;
use crate::state::AppState;
use crate::timers;

#[tauri::command(async)]
pub fn timers_list(state: State<AppState>) -> AppResult<Vec<timers::Timer>> {
    state.with(|db, _| timers::list(db))
}

#[tauri::command(async)]
pub fn timer_start(state: State<AppState>, title: String, ms: i64) -> AppResult<timers::Timer> {
    state.with(|db, _| timers::start(db, &title, ms))
}

#[tauri::command(async)]
pub fn timer_add_minute(state: State<AppState>, id: String) -> AppResult<()> {
    state.with(|db, _| timers::add_minute(db, &id))
}

#[tauri::command(async)]
pub fn timer_cancel(state: State<AppState>, id: String) -> AppResult<()> {
    state.with(|db, _| timers::cancel(db, &id))
}

#[tauri::command(async)]
pub fn timer_ring(state: State<AppState>, id: String) -> AppResult<()> {
    state.with(|db, _| timers::ring(db, &id))
}

#[tauri::command(async)]
pub fn timers_finished_while_closed(state: State<AppState>) -> AppResult<Vec<timers::Timer>> {
    state.with(|db, _| timers::finished_while_closed(db))
}
