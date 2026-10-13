//! Tasks, My day and ordering. Folders are in `folders`.

mod folders;

#[cfg(test)]
mod tests;

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::db::{columns, tables, Returning, Update};
use crate::error::{errors, AppError, AppResult};
use crate::names;
use crate::statuses::{self, DONE, FIRST};
use crate::util::{new_id, now_ms};

pub use folders::{
    create as folder_create, delete as folder_delete, move_to as folder_move,
    rename as folder_rename,
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub path: String,
    pub parent_id: Option<String>,
    pub title: String,
    pub status: String,
    pub due_date: Option<String>,
    pub due_time: Option<String>,
    pub assignee_id: Option<String>,
    pub order_key: String,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskData {
    pub folders: Vec<String>,
    pub tasks: Vec<Task>,
    /// Manual My day overrides for the given day: task id → included.
    pub my_day: Vec<(String, bool)>,
    /// Every (task id, label id) pair.
    pub labels: Vec<(String, String)>,
}

/// A task to create. A subtask takes its parent's folder. `order_key` places it: the UI makes the
/// key (src/lib/util/order-key.ts); a new task goes above the first one unless placed elsewhere.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewTask {
    pub path: String,
    pub title: String,
    pub parent_id: Option<String>,
    pub status: Option<String>,
    pub order_key: String,
    /// `YYYY-MM-DD`; a task added in My day is due today.
    pub due_date: Option<String>,
}

/// Fields a task page edit can change. Absent fields stay as they are; for the optional
/// columns, `Some(None)` clears the value.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Patch {
    pub title: Option<String>,
    pub status: Option<String>,
    pub description: Option<String>,
    #[serde(default, with = "crate::util::nullable")]
    pub due_date: Option<Option<String>>,
    #[serde(default, with = "crate::util::nullable")]
    pub due_time: Option<Option<String>>,
    #[serde(default, with = "crate::util::nullable")]
    pub assignee_id: Option<Option<String>>,
    pub order_key: Option<String>,
}

const COLS: &str = "id, path, parent_id, title, status, due_date, due_time, assignee_id, \
                    order_key, created_at, completed_at";

fn to_task(r: &rusqlite::Row) -> rusqlite::Result<Task> {
    Ok(Task {
        id: r.get(0)?,
        path: r.get(1)?,
        parent_id: r.get(2)?,
        title: r.get(3)?,
        status: r.get(4)?,
        due_date: r.get(5)?,
        due_time: r.get(6)?,
        assignee_id: r.get(7)?,
        order_key: r.get(8)?,
        created_at: r.get(9)?,
        completed_at: r.get(10)?,
    })
}

/// What a write to `tasks` answers with.
const WRITTEN: Returning<Task> = Returning {
    columns: COLS,
    map: to_task,
};

fn not_found() -> AppError {
    errors::NOT_FOUND.with("Task was not found")
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Task> {
    conn.query_row(
        &format!("SELECT {COLS} FROM tasks WHERE id = ?1"),
        [id],
        to_task,
    )
    .optional()?
    .ok_or_else(not_found)
}

pub fn description(conn: &Connection, id: &str) -> AppResult<String> {
    let text = conn.query_row("SELECT description FROM tasks WHERE id = ?1", [id], |r| {
        r.get(0)
    })?;
    Ok(text)
}

pub fn data(conn: &Connection, day: &str) -> AppResult<TaskData> {
    let folders = conn
        .prepare("SELECT path FROM task_folders WHERE path <> '' ORDER BY path")?
        .query_map([], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    let tasks = conn
        .prepare(&format!("SELECT {COLS} FROM tasks ORDER BY order_key, id"))?
        .query_map([], to_task)?
        .collect::<Result<_, _>>()?;
    let my_day = conn
        .prepare("SELECT task_id, included FROM my_day WHERE day = ?1")?
        .query_map([day], |r| Ok((r.get(0)?, r.get::<_, i64>(1)? == 1)))?
        .collect::<Result<_, _>>()?;
    let labels = crate::labels::links(conn)?;
    Ok(TaskData {
        folders,
        tasks,
        my_day,
        labels,
    })
}

fn check_title(title: &str) -> AppResult<&str> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(errors::TASK_TITLE_EMPTY.err());
    }
    Ok(trimmed)
}

/// The folder a new subtask goes in: its parent's. Subtasks are one level deep, so the parent
/// must be a top-level task.
fn parent_path(conn: &Connection, parent_id: &str) -> AppResult<String> {
    let parent = get(conn, parent_id)?;
    if parent.parent_id.is_some() {
        return Err(errors::SUBTASK_DEPTH.err());
    }
    Ok(parent.path)
}

pub fn create(conn: &mut Connection, new: &NewTask) -> AppResult<Task> {
    let title = check_title(&new.title)?;
    check_due((new.due_date.as_deref(), None))?;
    let path = match &new.parent_id {
        Some(parent) => parent_path(conn, parent)?,
        None => names::accept_path(&new.path)?,
    };
    let status = new.status.as_deref().unwrap_or(FIRST);
    statuses::check_usable(conn, status)?;
    let now = now_ms();
    let completed_at = (status == DONE).then_some(now);
    let tx = conn.transaction()?;
    folders::ensure(&tx, &path)?;
    let added = tx.query_row(
        &format!(
            "INSERT INTO tasks (id, path, parent_id, title, status, order_key, due_date,
                                created_at, updated_at, completed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8, ?9) RETURNING {COLS}"
        ),
        params![
            new_id(),
            path,
            new.parent_id,
            title,
            status,
            new.order_key,
            new.due_date,
            now,
            completed_at
        ],
        to_task,
    )?;
    tx.commit()?;
    Ok(added)
}

pub fn update(conn: &Connection, id: &str, patch: &Patch) -> AppResult<Task> {
    if let Some(title) = &patch.title {
        check_title(title)?;
    }
    check_due(new_due(patch))?;
    let mut update = Update::new(tables::TASKS);
    update.set_some(columns::TITLE, patch.title.as_deref().map(str::trim));
    if let Some(status) = &patch.status {
        statuses::check_usable(conn, status)?;
        set_status(&mut update, status);
    }
    update.set_some("description", patch.description.as_ref());
    update.set_some("due_date", patch.due_date.as_ref());
    update.set_some("due_time", patch.due_time.as_ref());
    update.set_some("assignee_id", patch.assignee_id.as_ref());
    update.set_some(columns::ORDER_KEY, patch.order_key.as_ref());
    update.run(conn, id, WRITTEN)?.ok_or_else(not_found)
}

/// The due date and time a patch sets (`None` where it leaves or clears them).
fn new_due(patch: &Patch) -> (Option<&str>, Option<&str>) {
    let date = patch.due_date.as_ref().and_then(|d| d.as_deref());
    let time = patch.due_time.as_ref().and_then(|t| t.as_deref());
    (date, time)
}

/// A due date must be a real day and a due time `HH:MM`, as the database keeps them.
fn check_due((date, time): (Option<&str>, Option<&str>)) -> AppResult<()> {
    if date.is_some_and(|d| !crate::dates::is_date(d)) {
        return Err(errors::DUE_DATE_INVALID.err());
    }
    if time.is_some_and(|t| !crate::dates::is_time(t)) {
        return Err(errors::DUE_TIME_INVALID.err());
    }
    Ok(())
}

/// Moving into Done stamps `completed_at` (kept if it was already done); leaving Done clears it.
fn set_status<'a>(update: &mut Update<'a>, status: &'a str) {
    update.set(columns::STATUS, status);
    if status == DONE {
        update.set_if_null("completed_at", now_ms());
    } else {
        update.set("completed_at", None::<i64>);
    }
}

/// A task and the folder it moves to.
pub struct Move<'a> {
    pub id: &'a str,
    pub target: &'a str,
}

/// Moves a task, and all its subtasks, to another folder.
pub fn move_to(conn: &mut Connection, change: &Move) -> AppResult<Task> {
    // The task as it is says whether it may move: a subtask stays with its parent.
    let task = get(conn, change.id)?;
    if task.parent_id.is_some() {
        return Err(errors::SUBTASK_MOVE.err());
    }
    let target = names::accept_path(change.target)?;
    let tx = conn.transaction()?;
    folders::ensure(&tx, &target)?;
    tx.execute(
        "WITH RECURSIVE tree(id) AS (
           SELECT ?1 UNION ALL SELECT t.id FROM tasks t JOIN tree ON t.parent_id = tree.id)
         UPDATE tasks SET path = ?2, updated_at = ?3 WHERE id IN (SELECT id FROM tree)",
        params![change.id, target, now_ms()],
    )?;
    tx.commit()?;
    Ok(Task {
        path: target,
        ..task
    })
}

pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    let deleted = conn.execute("DELETE FROM tasks WHERE id = ?1", [id])?;
    if deleted == 0 {
        return Err(not_found());
    }
    Ok(())
}

/// A task's manual My day choice for one day: in (`Some(true)`), out (`Some(false)`), or none, so
/// the default (due today) applies again (`None`).
pub struct DayChoice<'a> {
    pub id: &'a str,
    pub day: &'a str,
    pub included: Option<bool>,
}

pub fn set_my_day(conn: &Connection, choice: &DayChoice) -> AppResult<()> {
    let DayChoice { id, day, included } = *choice;
    match included {
        Some(include) => conn.execute(
            "INSERT INTO my_day (id, task_id, day, included, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(task_id, day) DO UPDATE SET included = excluded.included,
               updated_at = excluded.updated_at",
            params![new_id(), id, day, include, now_ms()],
        )?,
        // No choice to drop is fine: the default already applies.
        None => conn.execute(
            "DELETE FROM my_day WHERE task_id = ?1 AND day = ?2",
            params![id, day],
        )?,
    };
    Ok(())
}
