//! Task folders, tasks, My day and ordering. Folders are keyed by path under one root ('');
//! renaming a folder rewrites the paths below it, and tasks follow through ON UPDATE CASCADE.

use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::names;
use crate::util::{new_id, now_ms};

pub const DONE: &str = "DONE";
pub const FIRST: &str = "NOT_STARTED";

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

/// Fields a task page edit can change. Absent fields stay as they are; for the optional
/// columns, `Some(None)` clears the value.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Patch {
    pub title: Option<String>,
    pub status: Option<String>,
    pub description: Option<String>,
    #[serde(default, with = "nullable")]
    pub due_date: Option<Option<String>>,
    #[serde(default, with = "nullable")]
    pub due_time: Option<Option<String>>,
    #[serde(default, with = "nullable")]
    pub assignee_id: Option<Option<String>>,
    pub order_key: Option<String>,
}

/// Tells "missing" (leave alone) apart from `null` (clear) in a patch.
mod nullable {
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Option<String>>, D::Error> {
        Option::<String>::deserialize(d).map(Some)
    }
}

const COLS: &str =
    "id, path, parent_id, title, status, due_date, due_time, assignee_id, order_key, created_at, completed_at";

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
    conn.query_row(&format!("SELECT {COLS} FROM tasks WHERE id = ?1"), [id], to_task)
        .optional()?
        .ok_or_else(|| AppError::NotFound("Task".into()))
}

pub fn description(conn: &Connection, id: &str) -> AppResult<String> {
    Ok(conn.query_row("SELECT description FROM tasks WHERE id = ?1", [id], |r| r.get(0))?)
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
    Ok(TaskData { folders, tasks, my_day })
}

// ---------------------------------------------------------------------------------------------
// Folders

fn folder_exists(conn: &Connection, path: &str) -> AppResult<bool> {
    Ok(conn
        .query_row("SELECT 1 FROM task_folders WHERE path = ?1", [path], |_| Ok(()))
        .optional()?
        .is_some())
}

fn check_new_folder(conn: &Connection, parent: &str, name: &str) -> AppResult<()> {
    names::check(name)?;
    if folder_exists(conn, &names::join(parent, name))? {
        return Err(AppError::InvalidName(names::CLASH.into()));
    }
    Ok(())
}

/// Creates `path` and any missing parents.
pub fn folder_create(conn: &mut Connection, path: &str) -> AppResult<()> {
    names::check_path(path)?;
    if path.is_empty() || folder_exists(conn, path)? {
        return Err(AppError::InvalidName(names::CLASH.into()));
    }
    let tx = conn.transaction()?;
    ensure_folder(&tx, path)?;
    tx.commit()?;
    Ok(())
}

fn ensure_folder(tx: &Transaction, path: &str) -> AppResult<()> {
    names::check_path(path)?;
    let now = now_ms();
    let mut cur = String::new();
    for seg in path.split('/').filter(|s| !s.is_empty()) {
        cur = names::join(&cur, seg);
        tx.execute("INSERT OR IGNORE INTO task_folders (path, created_at) VALUES (?1, ?2)", params![cur, now])?;
    }
    Ok(())
}

pub fn folder_rename(conn: &mut Connection, path: &str, name: &str) -> AppResult<String> {
    if names::base(path) == name {
        return Ok(path.to_string());
    }
    let parent = names::parent(path);
    check_new_folder(conn, parent, name)?;
    relocate_folder(conn, path, &names::join(parent, name))
}

pub fn folder_move(conn: &mut Connection, path: &str, target: &str) -> AppResult<String> {
    if names::is_under(target, path) {
        return Err(AppError::rule("into_self", "A folder can’t move into itself"));
    }
    if names::parent(path) == target {
        return Err(AppError::rule("same_folder", "It’s already in that folder"));
    }
    check_new_folder(conn, target, names::base(path))?;
    let tx = conn.transaction()?;
    ensure_folder(&tx, target)?;
    tx.commit()?;
    relocate_folder(conn, path, &names::join(target, names::base(path)))
}

fn relocate_folder(conn: &mut Connection, old: &str, new: &str) -> AppResult<String> {
    if old.is_empty() || !folder_exists(conn, old)? {
        return Err(AppError::NotFound("Folder".into()));
    }
    let tx = conn.transaction()?;
    tx.execute(
        "UPDATE task_folders SET path = ?2 || substr(path, length(?1) + 1)
         WHERE path = ?1 OR substr(path, 1, length(?1) + 1) = ?1 || '/'",
        params![old, new],
    )?;
    tx.commit()?;
    Ok(new.to_string())
}

/// Deletes the folder, its subfolders and every task in them.
pub fn folder_delete(conn: &mut Connection, path: &str) -> AppResult<()> {
    if path.is_empty() {
        return Err(AppError::rule("root", "The root folder can’t be deleted"));
    }
    let tx = conn.transaction()?;
    let under = "path = ?1 OR substr(path, 1, length(?1) + 1) = ?1 || '/'";
    tx.execute(&format!("DELETE FROM tasks WHERE {under}"), [path])?;
    tx.execute(&format!("DELETE FROM task_folders WHERE {under}"), [path])?;
    tx.commit()?;
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// Tasks

fn first_key(conn: &Connection) -> AppResult<Option<String>> {
    Ok(conn.query_row("SELECT min(order_key) FROM tasks", [], |r| r.get(0))?)
}

/// Creates a task at the top of the global order. `order_key` places it elsewhere (board columns).
pub fn create(
    conn: &mut Connection,
    path: &str,
    title: &str,
    parent_id: Option<&str>,
    status: Option<&str>,
    order_key: Option<&str>,
) -> AppResult<Task> {
    let title = title.trim();
    if title.is_empty() {
        return Err(AppError::rule("empty_title", "A task needs a title"));
    }
    let path = match parent_id {
        Some(p) => get(conn, p)?.path,
        None => path.to_string(),
    };
    let key = match order_key {
        Some(k) => k.to_string(),
        None => crate::order::before(first_key(conn)?.as_deref()),
    };
    let status = status.unwrap_or(FIRST);
    let now = now_ms();
    let id = new_id();
    let tx = conn.transaction()?;
    ensure_folder(&tx, &path)?;
    tx.execute(
        "INSERT INTO tasks (id, path, parent_id, title, status, order_key, created_at, updated_at, completed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, ?8)",
        params![id, path, parent_id, title, status, key, now, (status == DONE).then_some(now)],
    )?;
    tx.commit()?;
    get(conn, &id)
}

pub fn update(conn: &Connection, id: &str, patch: &Patch) -> AppResult<Task> {
    let now = now_ms();
    if let Some(title) = &patch.title {
        if title.trim().is_empty() {
            return Err(AppError::rule("empty_title", "A task needs a title"));
        }
        conn.execute("UPDATE tasks SET title = ?1 WHERE id = ?2", params![title.trim(), id])?;
    }
    if let Some(status) = &patch.status {
        conn.execute(
            "UPDATE tasks SET status = ?1,
               completed_at = CASE WHEN ?1 = 'DONE' THEN coalesce(completed_at, ?2) ELSE NULL END
             WHERE id = ?3",
            params![status, now, id],
        )?;
    }
    if let Some(text) = &patch.description {
        conn.execute("UPDATE tasks SET description = ?1 WHERE id = ?2", params![text, id])?;
    }
    if let Some(due) = &patch.due_date {
        conn.execute("UPDATE tasks SET due_date = ?1 WHERE id = ?2", params![due, id])?;
    }
    if let Some(time) = &patch.due_time {
        conn.execute("UPDATE tasks SET due_time = ?1 WHERE id = ?2", params![time, id])?;
    }
    if let Some(who) = &patch.assignee_id {
        conn.execute("UPDATE tasks SET assignee_id = ?1 WHERE id = ?2", params![who, id])?;
    }
    if let Some(key) = &patch.order_key {
        conn.execute("UPDATE tasks SET order_key = ?1 WHERE id = ?2", params![key, id])?;
    }
    conn.execute("UPDATE tasks SET updated_at = ?1 WHERE id = ?2", params![now, id])?;
    get(conn, id)
}

/// Moves a task, and all its subtasks, to another folder.
pub fn move_to(conn: &mut Connection, id: &str, target: &str) -> AppResult<Task> {
    let task = get(conn, id)?;
    if task.parent_id.is_some() {
        return Err(AppError::rule("subtask", "Subtasks move with their parent"));
    }
    let tx = conn.transaction()?;
    ensure_folder(&tx, target)?;
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
        Some(inc) => conn.execute(
            "INSERT INTO my_day (task_id, day, included) VALUES (?1, ?2, ?3)
             ON CONFLICT(task_id, day) DO UPDATE SET included = excluded.included",
            params![id, day, inc as i64],
        )?,
        None => conn.execute("DELETE FROM my_day WHERE task_id = ?1 AND day = ?2", params![id, day])?,
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_memory;

    fn ids(conn: &Connection) -> Vec<String> {
        data(conn, "2026-10-04").unwrap().tasks.into_iter().map(|t| t.title).collect()
    }

    #[test]
    fn new_tasks_go_to_the_top_and_start_not_started() {
        let mut conn = open_memory();
        create(&mut conn, "work", "A", None, None, None).unwrap();
        let b = create(&mut conn, "work", "B", None, None, None).unwrap();
        assert_eq!(ids(&conn), vec!["B", "A"]);
        assert_eq!(b.status, FIRST);
        assert_eq!(data(&conn, "x").unwrap().folders, vec!["work"]);
    }

    #[test]
    fn reorder_writes_only_the_moved_key() {
        let mut conn = open_memory();
        let a = create(&mut conn, "", "A", None, None, None).unwrap();
        let b = create(&mut conn, "", "B", None, None, None).unwrap();
        let after_a = crate::order::between(Some(&a.order_key), None);
        update(&conn, &b.id, &Patch { order_key: Some(after_a), ..Default::default() }).unwrap();
        assert_eq!(ids(&conn), vec!["A", "B"]);
        assert_eq!(get(&conn, &a.id).unwrap().order_key, a.order_key);
    }

    #[test]
    fn done_sets_and_clears_completed_at() {
        let mut conn = open_memory();
        let t = create(&mut conn, "", "A", None, None, None).unwrap();
        let done = update(&conn, &t.id, &Patch { status: Some(DONE.into()), ..Default::default() }).unwrap();
        assert!(done.completed_at.is_some());
        let back = update(&conn, &t.id, &Patch { status: Some("IN_PROGRESS".into()), ..Default::default() }).unwrap();
        assert!(back.completed_at.is_none());
    }

    #[test]
    fn patch_clears_optional_fields_with_null() {
        let mut conn = open_memory();
        let t = create(&mut conn, "", "A", None, None, None).unwrap();
        let p: Patch = serde_json::from_str(r#"{"dueDate":"2026-10-10","dueTime":"17:30"}"#).unwrap();
        let t2 = update(&conn, &t.id, &p).unwrap();
        assert_eq!((t2.due_date.as_deref(), t2.due_time.as_deref()), (Some("2026-10-10"), Some("17:30")));
        let p: Patch = serde_json::from_str(r#"{"dueTime":null}"#).unwrap();
        let t3 = update(&conn, &t.id, &p).unwrap();
        assert_eq!((t3.due_date.as_deref(), t3.due_time), (Some("2026-10-10"), None));
    }

    #[test]
    fn folder_rename_rewrites_paths_below() {
        let mut conn = open_memory();
        create(&mut conn, "work/tend_v0_1", "A", None, None, None).unwrap();
        create(&mut conn, "work_x", "B", None, None, None).unwrap();
        folder_rename(&mut conn, "work", "job").unwrap();
        let d = data(&conn, "x").unwrap();
        assert_eq!(d.folders, vec!["job", "job/tend_v0_1", "work_x"]);
        let paths: Vec<_> = d.tasks.iter().map(|t| t.path.as_str()).collect();
        assert!(paths.contains(&"job/tend_v0_1") && paths.contains(&"work_x"));
        assert!(folder_rename(&mut conn, "job", "work_x").is_err(), "name clash");
        assert!(folder_move(&mut conn, "job", "job/tend_v0_1").is_err(), "into itself");
    }

    #[test]
    fn moving_a_task_moves_its_subtasks() {
        let mut conn = open_memory();
        let p = create(&mut conn, "work", "Parent", None, None, None).unwrap();
        let s = create(&mut conn, "", "Sub", Some(&p.id), None, None).unwrap();
        assert_eq!(s.path, "work", "subtasks live in the parent's folder");
        move_to(&mut conn, &p.id, "home").unwrap();
        assert_eq!(get(&conn, &s.id).unwrap().path, "home");
        assert!(move_to(&mut conn, &s.id, "work").is_err());
    }

    #[test]
    fn folder_delete_removes_tasks_inside() {
        let mut conn = open_memory();
        create(&mut conn, "work/a", "A", None, None, None).unwrap();
        create(&mut conn, "home", "B", None, None, None).unwrap();
        folder_delete(&mut conn, "work").unwrap();
        assert_eq!(ids(&conn), vec!["B"]);
        assert_eq!(data(&conn, "x").unwrap().folders, vec!["home"]);
        assert!(folder_delete(&mut conn, "").is_err());
    }

    #[test]
    fn my_day_overrides() {
        let mut conn = open_memory();
        let t = create(&mut conn, "", "A", None, None, None).unwrap();
        set_my_day(&conn, &t.id, "2026-10-04", Some(true)).unwrap();
        assert_eq!(data(&conn, "2026-10-04").unwrap().my_day, vec![(t.id.clone(), true)]);
        set_my_day(&conn, &t.id, "2026-10-04", Some(false)).unwrap();
        assert_eq!(data(&conn, "2026-10-04").unwrap().my_day, vec![(t.id.clone(), false)]);
        set_my_day(&conn, &t.id, "2026-10-04", None).unwrap();
        assert!(data(&conn, "2026-10-04").unwrap().my_day.is_empty());
        assert!(data(&conn, "2026-10-05").unwrap().my_day.is_empty());
    }
}
