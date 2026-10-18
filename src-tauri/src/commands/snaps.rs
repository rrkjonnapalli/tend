//! Snaps and stacks.

use tauri::State;

use crate::db;
use crate::error::AppResult;
use crate::snaps;
use crate::state::AppState;

#[tauri::command(async)]
pub fn snaps_board(state: State<AppState>) -> AppResult<snaps::Board> {
    state.with(|db, _| snaps::board(db))
}

#[tauri::command(async)]
pub fn snap_create(state: State<AppState>, snap: snaps::NewSnap) -> AppResult<snaps::Snap> {
    state.with(|conn, _| db::in_transaction(conn, |tx| snaps::create(tx, &snap)))
}

#[tauri::command(async)]
pub fn snap_update(
    state: State<AppState>,
    id: String,
    patch: snaps::Patch,
) -> AppResult<snaps::Snap> {
    state.with(|conn, _| db::in_transaction(conn, |tx| snaps::update(tx, &id, &patch)))
}

/// A snap dropped on a snap: a new stack holding both, all or nothing.
#[tauri::command(async)]
pub fn stack_pair(
    state: State<AppState>,
    parent: Option<String>,
    snaps: [String; 2],
) -> AppResult<snaps::Stack> {
    let pair = snaps::Pair {
        parent: parent.as_deref(),
        snaps: [&snaps[0], &snaps[1]],
    };
    state.with(|conn, _| db::in_transaction(conn, |tx| snaps::pair(tx, &pair)))
}

#[tauri::command(async)]
pub fn snap_delete(state: State<AppState>, id: String) -> AppResult<()> {
    state.with(|db, _| snaps::delete(db, &id))
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
