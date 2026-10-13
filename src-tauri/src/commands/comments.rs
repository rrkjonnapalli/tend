//! Comments on tasks.

use tauri::State;

use crate::comments;
use crate::error::AppResult;
use crate::state::AppState;

#[tauri::command(async)]
pub fn comments_list(state: State<AppState>, task: String) -> AppResult<Vec<comments::Comment>> {
    state.with(|db, _| comments::list(db, &task))
}

#[tauri::command(async)]
pub fn comment_add(state: State<AppState>, task: String, body: String) -> AppResult<()> {
    state.with(|db, _| {
        let new = comments::NewComment {
            task: &task,
            body: &body,
        };
        comments::add(db, &new)
    })
}

#[tauri::command(async)]
pub fn comment_edit(state: State<AppState>, id: String, body: String) -> AppResult<()> {
    state.with(|db, _| {
        let change = comments::Edit {
            id: &id,
            body: &body,
        };
        comments::edit(db, &change)
    })
}
