//! Task labels: a fixed list kept in Settings → Tasks, in a set order. A task carries any number
//! of them; deleting a label takes it off every task.

#[cfg(test)]
mod tests;

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

use crate::colors;
use crate::db::in_transaction;
use crate::error::{errors, AppResult};
use crate::util::{new_id, now_ms};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    pub id: String,
    pub name: String,
    pub color: String,
    pub position: i64,
}

const ROW_SQL: &str = "SELECT id, name, color, position FROM task_labels";

fn to_label(r: &rusqlite::Row) -> rusqlite::Result<Label> {
    Ok(Label {
        id: r.get(0)?,
        name: r.get(1)?,
        color: r.get(2)?,
        position: r.get(3)?,
    })
}

pub fn list(conn: &Connection) -> AppResult<Vec<Label>> {
    let labels = conn
        .prepare(&format!("{ROW_SQL} ORDER BY position"))?
        .query_map([], to_label)?
        .collect::<Result<_, _>>()?;
    Ok(labels)
}

fn get(conn: &Connection, id: &str) -> AppResult<Label> {
    conn.query_row(&format!("{ROW_SQL} WHERE id = ?1"), [id], to_label)
        .optional()?
        .ok_or_else(|| errors::NOT_FOUND.with("Label was not found"))
}

/// A non-empty name no other label uses (`except` is the label being renamed).
fn check_name(conn: &Connection, name: &str, except: Option<&str>) -> AppResult<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(errors::LABEL_NAME_EMPTY.err());
    }
    let taken = conn
        .query_row(
            "SELECT 1 FROM task_labels WHERE name = ?1 COLLATE NOCASE AND id IS NOT ?2",
            params![name, except],
            |_| Ok(()),
        )
        .optional()?;
    if taken.is_some() {
        return Err(errors::LABEL_NAME_TAKEN.err());
    }
    Ok(name.to_string())
}

/// Adds a label at the end of the list.
pub fn add(conn: &Connection, name: &str, color: &str) -> AppResult<Label> {
    let name = check_name(conn, name, None)?;
    colors::check(color)?;
    let position: i64 = conn.query_row(
        "SELECT coalesce(max(position) + 1, 0) FROM task_labels",
        [],
        |r| r.get(0),
    )?;
    let id = new_id();
    conn.execute(
        "INSERT INTO task_labels (id, name, color, position, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, name, color, position, now_ms()],
    )?;
    get(conn, &id)
}

pub fn rename(conn: &Connection, id: &str, name: &str) -> AppResult<Label> {
    let name = check_name(conn, name, Some(id))?;
    conn.execute(
        "UPDATE task_labels SET name = ?1 WHERE id = ?2",
        params![name, id],
    )?;
    get(conn, id)
}

pub fn recolor(conn: &Connection, id: &str, color: &str) -> AppResult<Label> {
    colors::check(color)?;
    conn.execute(
        "UPDATE task_labels SET color = ?1 WHERE id = ?2",
        params![color, id],
    )?;
    get(conn, id)
}

/// Sets the order; `ids` must be exactly the current labels.
pub fn reorder(conn: &mut Connection, ids: &[String]) -> AppResult<()> {
    let mut proposed: Vec<&str> = ids.iter().map(String::as_str).collect();
    proposed.sort_unstable();
    let current = list(conn)?;
    let mut existing: Vec<&str> = current.iter().map(|l| l.id.as_str()).collect();
    existing.sort_unstable();
    if proposed != existing {
        return Err(errors::LABEL_ORDER.err());
    }
    in_transaction(conn, |tx| {
        for (position, id) in ids.iter().enumerate() {
            tx.execute(
                "UPDATE task_labels SET position = ?1 WHERE id = ?2",
                params![position as i64, id],
            )?;
        }
        Ok(())
    })
}

/// How many tasks carry the label (shown before deleting it).
pub fn usage(conn: &Connection, id: &str) -> AppResult<i64> {
    let count = conn.query_row(
        "SELECT count(*) FROM task_label_links WHERE label_id = ?1",
        [id],
        |r| r.get(0),
    )?;
    Ok(count)
}

/// Deletes the label and takes it off every task (ON DELETE CASCADE).
pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM task_labels WHERE id = ?1", [id])?;
    Ok(())
}

/// The task's label ids, in the list's order.
pub fn of_task(conn: &Connection, task_id: &str) -> AppResult<Vec<String>> {
    let ids = conn
        .prepare(
            "SELECT l.id FROM task_label_links x JOIN task_labels l ON l.id = x.label_id
             WHERE x.task_id = ?1 ORDER BY l.position",
        )?
        .query_map([task_id], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    Ok(ids)
}

/// Every (task id, label id) pair, for the task lists and the board.
pub fn links(conn: &Connection) -> AppResult<Vec<(String, String)>> {
    let pairs = conn
        .prepare("SELECT task_id, label_id FROM task_label_links")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<_, _>>()?;
    Ok(pairs)
}

/// Replaces the task's labels. Every id must be a configured label.
pub fn set_for_task(
    conn: &mut Connection,
    task_id: &str,
    ids: &[String],
) -> AppResult<Vec<String>> {
    for id in ids {
        get(conn, id)?;
    }
    in_transaction(conn, |tx| {
        tx.execute("DELETE FROM task_label_links WHERE task_id = ?1", [task_id])?;
        for id in ids {
            tx.execute(
                "INSERT OR IGNORE INTO task_label_links (task_id, label_id) VALUES (?1, ?2)",
                params![task_id, id],
            )?;
        }
        Ok(())
    })?;
    of_task(conn, task_id)
}
