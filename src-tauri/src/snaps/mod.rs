//! Snaps: short writing pinned to a board, loose or in a stack (`stacks`). Only the text is needed.
//! The board orders snaps by pinned, then last edit; positions are never stored.

mod stacks;

#[cfg(test)]
mod tests;

use rusqlite::{params, Connection, OptionalExtension, ToSql, Transaction};
use serde::{Deserialize, Serialize};

use crate::colors;
use crate::enums::{Paper, SourceType};
use crate::error::{errors, AppResult};
use crate::tags;
use crate::util::{new_id, now_ms};

pub use stacks::{
    create as stack_create, move_to as stack_move, remove as stack_remove, rename as stack_rename,
    Stack,
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Snap {
    pub id: String,
    pub stack_id: Option<String>,
    pub text: String,
    pub paper: Paper,
    /// The picked colour of a sticky; `None` on other papers.
    pub color: Option<String>,
    pub pinned: bool,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Everything the board shows.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub snaps: Vec<Snap>,
    pub stacks: Vec<Stack>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewSnap {
    pub text: String,
    pub stack_id: Option<String>,
    pub paper: Paper,
    pub color: Option<String>,
    #[serde(default)]
    pub pinned: bool,
}

/// Fields an edit can change; absent fields stay. `stackId: null` puts the snap loose on the
/// board.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Patch {
    pub text: Option<String>,
    #[serde(default, with = "crate::util::nullable")]
    pub stack_id: Option<Option<String>>,
    pub paper: Option<Paper>,
    #[serde(default, with = "crate::util::nullable")]
    pub color: Option<Option<String>>,
    pub pinned: Option<bool>,
}

const COLS: &str = "id, stack_id, text, paper, color, pinned, created_at, updated_at";

fn to_snap(r: &rusqlite::Row) -> rusqlite::Result<Snap> {
    let paper: String = r.get(3)?;
    Ok(Snap {
        id: r.get(0)?,
        stack_id: r.get(1)?,
        text: r.get(2)?,
        paper: Paper::parse(&paper),
        color: r.get(4)?,
        pinned: r.get(5)?,
        tags: Vec::new(),
        created_at: r.get(6)?,
        updated_at: r.get(7)?,
    })
}

/// Every snap (pinned first, then the latest edit) and every stack.
pub fn board(conn: &Connection) -> AppResult<Board> {
    let mut tags = tags::by_record(conn, SourceType::Snap)?;
    let mut snaps: Vec<Snap> = conn
        .prepare(&format!(
            "SELECT {COLS} FROM snaps ORDER BY pinned DESC, updated_at DESC, id"
        ))?
        .query_map([], to_snap)?
        .collect::<Result<_, _>>()?;
    for snap in &mut snaps {
        snap.tags = tags.remove(&snap.id).unwrap_or_default();
    }
    Ok(Board {
        snaps,
        stacks: stacks::list(conn)?,
    })
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Snap> {
    let mut snap = conn
        .query_row(
            &format!("SELECT {COLS} FROM snaps WHERE id = ?1"),
            [id],
            to_snap,
        )
        .optional()?
        .ok_or_else(|| errors::NOT_FOUND.with("Snap was not found"))?;
    snap.tags = tags::of(conn, SourceType::Snap, id)?;
    Ok(snap)
}

/// Text with nothing but spaces is refused; the text is kept exactly as written.
fn check_text(text: &str) -> AppResult<()> {
    if text.trim().is_empty() {
        return Err(errors::SNAP_TEXT_EMPTY.err());
    }
    Ok(())
}

/// The colour kept: a sticky's, checked; none on other papers.
fn kept_color(paper: Paper, color: Option<&str>) -> AppResult<Option<String>> {
    if paper != Paper::Sticky {
        return Ok(None);
    }
    let Some(color) = color else {
        return Ok(None);
    };
    colors::check(color)?;
    Ok(Some(color.to_string()))
}

pub fn create(conn: &Connection, new: &NewSnap) -> AppResult<Snap> {
    check_text(&new.text)?;
    if let Some(stack) = &new.stack_id {
        stacks::get(conn, stack)?;
    }
    let color = kept_color(new.paper, new.color.as_deref())?;
    let id = new_id();
    conn.execute(
        "INSERT INTO snaps (id, stack_id, text, paper, color, pinned, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        params![
            id,
            new.stack_id,
            new.text,
            new.paper.as_str(),
            color,
            new.pinned,
            now_ms()
        ],
    )?;
    get(conn, &id)
}

/// The paper and colour after `patch`: a paper that isn't sticky drops the colour.
fn look_after(current: &Snap, patch: &Patch) -> AppResult<(Paper, Option<String>)> {
    let paper = patch.paper.unwrap_or(current.paper);
    let color = match &patch.color {
        Some(color) => color.clone(),
        None => current.color.clone(),
    };
    Ok((paper, kept_color(paper, color.as_deref())?))
}

/// Saves an edit; the snap moves to the front of its level (its `updated_at` is now).
pub fn update(conn: &mut Connection, id: &str, patch: &Patch) -> AppResult<Snap> {
    let current = get(conn, id)?;
    if let Some(text) = &patch.text {
        check_text(text)?;
    }
    if let Some(Some(stack)) = &patch.stack_id {
        stacks::get(conn, stack)?;
    }
    let (paper, color) = look_after(&current, patch)?;
    let tx = conn.transaction()?;
    if let Some(text) = &patch.text {
        set(&tx, id, "text", text)?;
    }
    if let Some(stack) = &patch.stack_id {
        set(&tx, id, "stack_id", stack)?;
    }
    set(&tx, id, "paper", &paper.as_str())?;
    set(&tx, id, "color", &color)?;
    if let Some(pinned) = patch.pinned {
        set(&tx, id, "pinned", &pinned)?;
    }
    set(&tx, id, "updated_at", &now_ms())?;
    tx.commit()?;
    get(conn, id)
}

/// Writes one column. `column` is always one of the literals in `update`, never user input.
fn set(tx: &Transaction, id: &str, column: &'static str, value: &dyn ToSql) -> AppResult<()> {
    tx.execute(
        &format!("UPDATE snaps SET {column} = ?1 WHERE id = ?2"),
        params![value, id],
    )?;
    Ok(())
}

/// Deletes the snap; its tags come off with it (trigger `snaps_drop_links`).
pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM snaps WHERE id = ?1", [id])?;
    Ok(())
}
