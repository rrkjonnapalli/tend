//! Stacks: folders of snaps that can nest. A stack never goes inside itself or its own sub-stacks;
//! removing one keeps its snaps and sub-stacks, moving them up one level.

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

use crate::db::in_transaction;
use crate::error::{errors, AppError, AppResult};
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
    .ok_or_else(not_found)
}

fn not_found() -> AppError {
    errors::NOT_FOUND.with("Stack was not found")
}

/// A stack to add inside `parent` (`None`: on the board).
pub struct NewStack<'a> {
    pub parent: Option<&'a str>,
    pub name: &'a str,
}

pub fn create(conn: &Connection, new: &NewStack) -> AppResult<Stack> {
    if let Some(parent) = new.parent {
        get(conn, parent)?;
    }
    let added = conn.query_row(
        &format!(
            "INSERT INTO snap_stacks (id, parent_id, name, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?4) RETURNING {COLS}"
        ),
        params![new_id(), new.parent, new.name.trim(), now_ms()],
        to_stack,
    )?;
    Ok(added)
}

/// A stack and the name it gets.
pub struct Rename<'a> {
    pub id: &'a str,
    pub name: &'a str,
}

pub fn rename(conn: &Connection, change: &Rename) -> AppResult<Stack> {
    conn.query_row(
        &format!(
            "UPDATE snap_stacks SET name = ?1, updated_at = ?2 WHERE id = ?3 RETURNING {COLS}"
        ),
        params![change.name.trim(), now_ms(), change.id],
        to_stack,
    )
    .optional()?
    .ok_or_else(not_found)
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

/// A stack and where it goes: inside `parent`, or onto the board (`None`).
pub struct Move<'a> {
    pub id: &'a str,
    pub parent: Option<&'a str>,
}

pub fn move_to(conn: &Connection, change: &Move) -> AppResult<Stack> {
    if let Some(parent) = change.parent {
        if is_within(conn, parent, change.id)? {
            return Err(errors::STACK_INTO_SELF.err());
        }
    }
    conn.query_row(
        &format!(
            "UPDATE snap_stacks SET parent_id = ?1, updated_at = ?2 WHERE id = ?3
             RETURNING {COLS}"
        ),
        params![change.parent, now_ms(), change.id],
        to_stack,
    )
    .optional()?
    .ok_or_else(not_found)
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
