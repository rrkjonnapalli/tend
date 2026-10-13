//! Tags on notes, tasks and Library items.

use tauri::State;

use crate::enums::{EnumStatus, SourceType};
use crate::error::AppResult;
use crate::links::Source;
use crate::state::AppState;
use crate::tags;

/// The tags that can be picked: blocked ones are left out.
#[tauri::command(async)]
pub fn tags_all(state: State<AppState>) -> AppResult<Vec<String>> {
    state.with(|db, _| tags::all(db))
}

/// Every tag, blocked ones too, with how many records carry each.
#[tauri::command(async)]
pub fn tags_overview(state: State<AppState>) -> AppResult<Vec<tags::TagRow>> {
    state.with(|db, _| tags::overview(db))
}

#[tauri::command(async)]
pub fn tag_set_status(state: State<AppState>, name: String, status: EnumStatus) -> AppResult<()> {
    state.with(|db, _| tags::set_status(db, &name, status))
}

/// Deletes the tag from the pool and from everything that carries it.
#[tauri::command(async)]
pub fn tag_delete(state: State<AppState>, name: String) -> AppResult<()> {
    state.with(|db, _| tags::delete(db, &name))
}

fn set(state: State<AppState>, source: Source, tags: &[String]) -> AppResult<Vec<String>> {
    state.with(|db, _| tags::set(db, source, tags))
}

#[tauri::command(async)]
pub fn note_tags_set(
    state: State<AppState>,
    id: String,
    tags: Vec<String>,
) -> AppResult<Vec<String>> {
    let source = Source {
        kind: SourceType::Note,
        id: &id,
    };
    set(state, source, &tags)
}

#[tauri::command(async)]
pub fn task_tags_set(
    state: State<AppState>,
    id: String,
    tags: Vec<String>,
) -> AppResult<Vec<String>> {
    let source = Source {
        kind: SourceType::Task,
        id: &id,
    };
    set(state, source, &tags)
}

#[tauri::command(async)]
pub fn word_tags_set(
    state: State<AppState>,
    id: String,
    tags: Vec<String>,
) -> AppResult<Vec<String>> {
    let source = Source {
        kind: SourceType::Word,
        id: &id,
    };
    set(state, source, &tags)
}

#[tauri::command(async)]
pub fn snap_tags_set(
    state: State<AppState>,
    id: String,
    tags: Vec<String>,
) -> AppResult<Vec<String>> {
    let source = Source {
        kind: SourceType::Snap,
        id: &id,
    };
    set(state, source, &tags)
}

#[tauri::command(async)]
pub fn ref_tags_set(
    state: State<AppState>,
    id: String,
    tags: Vec<String>,
) -> AppResult<Vec<String>> {
    let source = Source {
        kind: SourceType::Ref,
        id: &id,
    };
    set(state, source, &tags)
}
