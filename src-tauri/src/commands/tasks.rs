//! Tasks, task folders and My day.

use tauri::State;

use crate::error::AppResult;
use crate::state::AppState;
use crate::{tags, tasks};

/// Everything the task lists need, with My day overrides for `day` (the UI's local today).
#[tauri::command(async)]
pub fn tasks_data(state: State<AppState>, day: String) -> AppResult<tasks::TaskData> {
    state.with(|db, _| tasks::data(db, &day))
}

/// One task with the fields only its page shows.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskDetail {
    #[serde(flatten)]
    task: tasks::Task,
    description: String,
    tags: Vec<String>,
    /// Label ids, in the list's order.
    labels: Vec<String>,
}

#[tauri::command(async)]
pub fn task_get(state: State<AppState>, id: String) -> AppResult<TaskDetail> {
    state.with(|db, _| {
        Ok(TaskDetail {
            task: tasks::get(db, &id)?,
            description: tasks::description(db, &id)?,
            tags: tags::of(db, tags::Owner::Task, &id)?,
            labels: crate::labels::of_task(db, &id)?,
        })
    })
}

#[tauri::command(async)]
pub fn task_create(state: State<AppState>, task: tasks::NewTask) -> AppResult<tasks::Task> {
    state.with(|db, _| tasks::create(db, &task))
}

#[tauri::command(async)]
pub fn task_update(
    state: State<AppState>,
    id: String,
    patch: tasks::Patch,
) -> AppResult<tasks::Task> {
    state.with(|db, _| tasks::update(db, &id, &patch))
}

#[tauri::command(async)]
pub fn task_move(state: State<AppState>, id: String, target: String) -> AppResult<tasks::Task> {
    state.with(|db, _| tasks::move_to(db, &id, &target))
}

#[tauri::command(async)]
pub fn task_delete(state: State<AppState>, id: String) -> AppResult<()> {
    state.with(|db, _| tasks::delete(db, &id))
}

#[tauri::command(async)]
pub fn my_day_set(
    state: State<AppState>,
    id: String,
    day: String,
    included: Option<bool>,
) -> AppResult<()> {
    state.with(|db, _| tasks::set_my_day(db, &id, &day, included))
}

#[tauri::command(async)]
pub fn task_folder_create(state: State<AppState>, path: String) -> AppResult<()> {
    state.with(|db, _| tasks::folder_create(db, &path))
}

#[tauri::command(async)]
pub fn task_folder_rename(state: State<AppState>, path: String, name: String) -> AppResult<String> {
    state.with(|db, _| tasks::folder_rename(db, &path, &name))
}

#[tauri::command(async)]
pub fn task_folder_move(state: State<AppState>, path: String, target: String) -> AppResult<String> {
    state.with(|db, _| tasks::folder_move(db, &path, &target))
}

#[tauri::command(async)]
pub fn task_folder_delete(state: State<AppState>, path: String) -> AppResult<()> {
    state.with(|db, _| tasks::folder_delete(db, &path))
}
