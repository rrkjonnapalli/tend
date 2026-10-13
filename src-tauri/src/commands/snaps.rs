//! Snaps and stacks.

use tauri::State;

use crate::error::AppResult;
use crate::snaps;
use crate::state::AppState;

#[tauri::command(async)]
pub fn snaps_board(state: State<AppState>) -> AppResult<snaps::Board> {
    state.with(|db, _| snaps::board(db))
}

#[tauri::command(async)]
pub fn snap_create(state: State<AppState>, snap: snaps::NewSnap) -> AppResult<snaps::Snap> {
    state.with(|db, _| snaps::create(db, &snap))
}

#[tauri::command(async)]
pub fn snap_update(
    state: State<AppState>,
    id: String,
    patch: snaps::Patch,
) -> AppResult<snaps::Snap> {
    state.with(|db, _| snaps::update(db, &id, &patch))
}

#[tauri::command(async)]
pub fn snap_delete(state: State<AppState>, id: String) -> AppResult<()> {
    state.with(|db, _| snaps::delete(db, &id))
}

#[tauri::command(async)]
pub fn stack_create(
    state: State<AppState>,
    parent: Option<String>,
    name: String,
) -> AppResult<snaps::Stack> {
    state.with(|db, _| {
        let new = snaps::NewStack {
            parent: parent.as_deref(),
            name: &name,
        };
        snaps::stack_create(db, &new)
    })
}

#[tauri::command(async)]
pub fn stack_rename(state: State<AppState>, id: String, name: String) -> AppResult<snaps::Stack> {
    state.with(|db, _| {
        let change = snaps::StackRename {
            id: &id,
            name: &name,
        };
        snaps::stack_rename(db, &change)
    })
}

#[tauri::command(async)]
pub fn stack_move(
    state: State<AppState>,
    id: String,
    parent: Option<String>,
) -> AppResult<snaps::Stack> {
    state.with(|db, _| {
        let change = snaps::StackMove {
            id: &id,
            parent: parent.as_deref(),
        };
        snaps::stack_move(db, &change)
    })
}

/// Removes the stack; its snaps and sub-stacks move up a level.
#[tauri::command(async)]
pub fn stack_remove(state: State<AppState>, id: String) -> AppResult<()> {
    state.with(|db, _| snaps::stack_remove(db, &id))
}
