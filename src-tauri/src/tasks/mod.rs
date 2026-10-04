//! Tasks, My day and ordering. Folders are in `folders`.

mod folders;

#[cfg(test)]
mod tests;

use rusqlite::{params, Connection, OptionalExtension, ToSql, Transaction};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::statuses::{DONE, FIRST};
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
}

/// A task to create. A subtask takes its parent's folder; `order_key` places it somewhere
/// other than the top of the list (board columns).
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewTask {
    pub path: String,
    pub title: String,
    pub parent_id: Option<String>,
    pub status: Option<String>,
    pub order_key: Option<String>,
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

pub fn get(conn: &Connection, id: &str) -> AppResult<Task> {
    conn.query_row(
        &format!("SELECT {COLS} FROM tasks WHERE id = ?1"),
        [id],
        to_task,
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound("Task".into()))
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
        .prepare(&format!("SELECT {COLS} FROM tasks ORDER BY order_key"))?
        .query_map([], to_task)?
        .collect::<Result<_, _>>()?;
    let my_day = conn
        .prepare("SELECT task_id, included FROM my_day WHERE day = ?1")?
        .query_map([day], |r| Ok((r.get(0)?, r.get::<_, i64>(1)? == 1)))?
        .collect::<Result<_, _>>()?;
    Ok(TaskData {
        folders,
        tasks,
        my_day,
    })
}

fn check_title(title: &str) -> AppResult<&str> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(AppError::rule("empty_title", "A task needs a title"));
    }
    Ok(trimmed)
}

/// A key that sorts before every task, so a new task lands at the top.
fn top_key(conn: &Connection) -> AppResult<String> {
    let first: Option<String> =
        conn.query_row("SELECT min(order_key) FROM tasks", [], |r| r.get(0))?;
    Ok(crate::order::before(first.as_deref()))
}

pub fn create(conn: &mut Connection, new: &NewTask) -> AppResult<Task> {
    let title = check_title(&new.title)?;
    let path = match &new.parent_id {
        Some(parent) => get(conn, parent)?.path,
        None => new.path.clone(),
    };
    let key = match &new.order_key {
        Some(k) => k.clone(),
        None => top_key(conn)?,
    };
    let status = new.status.as_deref().unwrap_or(FIRST);
    let now = now_ms();
    let completed_at = (status == DONE).then_some(now);
    let id = new_id();
    let tx = conn.transaction()?;
    folders::ensure(&tx, &path)?;
    tx.execute(
        "INSERT INTO tasks (id, path, parent_id, title, status, order_key, created_at, updated_at, completed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, ?8)",
        params![id, path, new.parent_id, title, status, key, now, completed_at],
    )?;
    tx.commit()?;
    get(conn, &id)
}

pub fn update(conn: &mut Connection, id: &str, patch: &Patch) -> AppResult<Task> {
    if let Some(title) = &patch.title {
        check_title(title)?;
    }
    let now = now_ms();
    let tx = conn.transaction()?;
    if let Some(title) = &patch.title {
        set(&tx, id, "title", &title.trim())?;
    }
    if let Some(status) = &patch.status {
        set_status(&tx, id, status, now)?;
    }
    if let Some(text) = &patch.description {
        set(&tx, id, "description", text)?;
    }
    if let Some(due) = &patch.due_date {
        set(&tx, id, "due_date", due)?;
    }
    if let Some(time) = &patch.due_time {
        set(&tx, id, "due_time", time)?;
    }
    if let Some(who) = &patch.assignee_id {
        set(&tx, id, "assignee_id", who)?;
    }
    if let Some(key) = &patch.order_key {
        set(&tx, id, "order_key", key)?;
    }
    set(&tx, id, "updated_at", &now)?;
    tx.commit()?;
    get(conn, id)
}

/// Writes one column. `column` is always one of the literals in `update`, never user input.
fn set(tx: &Transaction, id: &str, column: &'static str, value: &dyn ToSql) -> AppResult<()> {
    tx.execute(
        &format!("UPDATE tasks SET {column} = ?1 WHERE id = ?2"),
        params![value, id],
    )?;
    Ok(())
}

/// Moving into Done stamps `completed_at` (kept if it was already done); leaving Done clears it.
fn set_status(tx: &Transaction, id: &str, status: &str, now: i64) -> AppResult<()> {
    tx.execute(
        "UPDATE tasks SET status = ?1,
           completed_at = CASE WHEN ?1 = ?2 THEN coalesce(completed_at, ?3) ELSE NULL END
         WHERE id = ?4",
        params![status, DONE, now, id],
    )?;
    Ok(())
}

/// Moves a task, and all its subtasks, to another folder.
pub fn move_to(conn: &mut Connection, id: &str, target: &str) -> AppResult<Task> {
    let task = get(conn, id)?;
    if task.parent_id.is_some() {
        return Err(AppError::rule("subtask", "Subtasks move with their parent"));
    }
    let tx = conn.transaction()?;
    folders::ensure(&tx, target)?;
    tx.execute(
        "WITH RECURSIVE tree(id) AS (
           SELECT ?1 UNION ALL SELECT t.id FROM tasks t JOIN tree ON t.parent_id = tree.id)
         UPDATE tasks SET path = ?2, updated_at = ?3 WHERE id IN (SELECT id FROM tree)",
        params![id, target, now_ms()],
    )?;
    tx.commit()?;
    get(conn, id)
}

pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM tasks WHERE id = ?1", [id])?;
    Ok(())
}

/// Adds the task to My day for `day` (`Some(true)`), removes it (`Some(false)`), or drops the
/// manual choice so the default (due today) applies again (`None`).
pub fn set_my_day(conn: &Connection, id: &str, day: &str, included: Option<bool>) -> AppResult<()> {
    match included {
        Some(include) => conn.execute(
            "INSERT INTO my_day (task_id, day, included) VALUES (?1, ?2, ?3)
             ON CONFLICT(task_id, day) DO UPDATE SET included = excluded.included",
            params![id, day, include],
        )?,
        None => conn.execute(
            "DELETE FROM my_day WHERE task_id = ?1 AND day = ?2",
            params![id, day],
        )?,
    };
    Ok(())
}
