//! Stacks: folders of snaps that can nest. A stack never goes inside itself or its own sub-stacks;
//! removing one keeps its snaps and sub-stacks, moving them up one level.

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

use crate::db::in_transaction;
use crate::error::{errors, AppResult};
use crate::util::{new_id, now_ms};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Stack {
    pub id: String,
    pub parent_id: Option<String>,
    /// Free text; empty shows as "Untitled stack".
    pub name: String,
    pub updated_at: i64,
}

const COLS: &str = "id, parent_id, name, updated_at";

fn to_stack(r: &rusqlite::Row) -> rusqlite::Result<Stack> {
    Ok(Stack {
        id: r.get(0)?,
        parent_id: r.get(1)?,
        name: r.get(2)?,
        updated_at: r.get(3)?,
    })
}

pub fn list(conn: &Connection) -> AppResult<Vec<Stack>> {
    let stacks = conn
        .prepare(&format!(
            "SELECT {COLS} FROM snap_stacks ORDER BY updated_at DESC, id"
        ))?
        .query_map([], to_stack)?
        .collect::<Result<_, _>>()?;
    Ok(stacks)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Stack> {
    conn.query_row(
        &format!("SELECT {COLS} FROM snap_stacks WHERE id = ?1"),
        [id],
        to_stack,
    )
    .optional()?
    .ok_or_else(|| errors::NOT_FOUND.with("Stack was not found"))
}

/// Adds a stack inside `parent` (`None`: on the board).
pub fn create(conn: &Connection, parent: Option<&str>, name: &str) -> AppResult<Stack> {
    if let Some(parent) = parent {
        get(conn, parent)?;
    }
    let id = new_id();
    conn.execute(
        "INSERT INTO snap_stacks (id, parent_id, name, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?4)",
        params![id, parent, name.trim(), now_ms()],
    )?;
    get(conn, &id)
}

pub fn rename(conn: &Connection, id: &str, name: &str) -> AppResult<Stack> {
    conn.execute(
        "UPDATE snap_stacks SET name = ?1, updated_at = ?2 WHERE id = ?3",
        params![name.trim(), now_ms(), id],
    )?;
    get(conn, id)
}

/// True when `id` is `ancestor` or sits somewhere inside it. The walk stops after as many steps as
/// there are stacks, so a cycle (impossible here, but a sync could bring one) can't loop forever.
fn is_within(conn: &Connection, id: &str, ancestor: &str) -> AppResult<bool> {
    let limit: i64 = conn.query_row("SELECT count(*) FROM snap_stacks", [], |r| r.get(0))?;
    let mut current = Some(id.to_string());
    for _ in 0..=limit {
        let Some(stack) = current else {
            return Ok(false);
        };
        if stack == ancestor {
            return Ok(true);
        }
        current = get(conn, &stack)?.parent_id;
    }
    Ok(false)
}

/// Moves the stack inside `parent` (`None`: onto the board).
pub fn move_to(conn: &Connection, id: &str, parent: Option<&str>) -> AppResult<Stack> {
    get(conn, id)?;
    if let Some(parent) = parent {
        if is_within(conn, parent, id)? {
            return Err(errors::STACK_INTO_SELF.err());
        }
    }
    conn.execute(
        "UPDATE snap_stacks SET parent_id = ?1, updated_at = ?2 WHERE id = ?3",
        params![parent, now_ms(), id],
    )?;
    get(conn, id)
}

/// Removes the stack. Its snaps and sub-stacks move up to its parent.
pub fn remove(conn: &mut Connection, id: &str) -> AppResult<()> {
    let parent = get(conn, id)?.parent_id;
    let now = now_ms();
    in_transaction(conn, |tx| {
        tx.execute(
            "UPDATE snaps SET stack_id = ?1, updated_at = ?2 WHERE stack_id = ?3",
            params![parent, now, id],
        )?;
        tx.execute(
            "UPDATE snap_stacks SET parent_id = ?1, updated_at = ?2 WHERE parent_id = ?3",
            params![parent, now, id],
        )?;
        tx.execute("DELETE FROM snap_stacks WHERE id = ?1", [id])?;
        Ok(())
    })
}
