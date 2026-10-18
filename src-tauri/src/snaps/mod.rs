//! Snaps: short writing pinned to a board, loose or in a stack (`stacks`). Only the text is needed.
//! The board orders snaps by pinned, then last edit; positions are never stored.

mod stacks;

#[cfg(test)]
mod tests;

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::colors;
use crate::db::{columns, tables, Returning, Update};
use crate::enums::{Paper, SourceType};
use crate::error::{errors, AppError, AppResult};
use crate::tags;
use crate::util::{new_id, now_ms};

pub use stacks::{
    move_to as stack_move, remove as stack_remove, rename as stack_rename, Move as StackMove,
    NewStack, Rename as StackRename, Stack,
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
    /// Put the snap in a new stack instead of `stack_id`.
    #[serde(default)]
    pub start_stack: Option<StartStack>,
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
    /// Move the snap into a new stack instead of `stack_id`.
    #[serde(default)]
    pub start_stack: Option<StartStack>,
}

/// A new untitled stack a saved snap starts, inside `parent` (`None`: on the board). Made in the
/// same transaction as the snap, so a failed save leaves no empty stack.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartStack {
    pub parent: Option<String>,
}

/// A snap dropped on a snap: both go into a new untitled stack inside `parent`.
pub struct Pair<'a> {
    pub parent: Option<&'a str>,
    pub snaps: [&'a str; 2],
}

const COLS: &str = "id, stack_id, text, paper, color, pinned, created_at, updated_at";

/// What a write to `snaps` answers with.
const WRITTEN: Returning<Snap> = Returning {
    columns: COLS,
    map: to_snap,
};

fn not_found() -> AppError {
    errors::NOT_FOUND.with("Snap was not found")
}

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

/// The new stack's id. Callers run in one transaction, so the stack and the snap land together.
fn started(conn: &Connection, start: &StartStack) -> AppResult<String> {
    let new = NewStack {
        parent: start.parent.as_deref(),
        name: "",
    };
    Ok(stacks::create(conn, &new)?.id)
}

pub fn create(conn: &Connection, new: &NewSnap) -> AppResult<Snap> {
    check_text(&new.text)?;
    let stack_id = match &new.start_stack {
        Some(start) => Some(started(conn, start)?),
        None => new.stack_id.clone(),
    };
    if let Some(stack) = &stack_id {
        stacks::get(conn, stack)?;
    }
    let color = kept_color(new.paper, new.color.as_deref())?;
    let added = conn.query_row(
        &format!(
            "INSERT INTO snaps (id, stack_id, text, paper, color, pinned, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7) RETURNING {COLS}"
        ),
        params![
            new_id(),
            stack_id,
            new.text,
            new.paper.as_str(),
            color,
            new.pinned,
            now_ms()
        ],
        to_snap,
    )?;
    // A new snap carries no tags yet.
    Ok(added)
}

/// The paper and colour after a `patch` that names either: a paper that isn't sticky drops the
/// colour, so whichever the patch leaves out comes from the snap as it is.
fn look_after(conn: &Connection, id: &str, patch: &Patch) -> AppResult<(Paper, Option<String>)> {
    let current: Option<(String, Option<String>)> = conn
        .query_row("SELECT paper, color FROM snaps WHERE id = ?1", [id], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .optional()?;
    let (current_paper, current_color) = current.ok_or_else(not_found)?;
    let paper = patch.paper.unwrap_or(Paper::parse(&current_paper));
    let color = match &patch.color {
        Some(color) => color.clone(),
        None => current_color,
    };
    Ok((paper, kept_color(paper, color.as_deref())?))
}

/// Saves an edit; the snap moves to the front of its level (its `updated_at` is now).
pub fn update(conn: &Connection, id: &str, patch: &Patch) -> AppResult<Snap> {
    if let Some(text) = &patch.text {
        check_text(text)?;
    }
    let stack_id = match &patch.start_stack {
        Some(start) => Some(Some(started(conn, start)?)),
        None => patch.stack_id.clone(),
    };
    if let Some(Some(stack)) = &stack_id {
        stacks::get(conn, stack)?;
    }
    // Only when asked: a paper this version doesn't know must survive a pin or a move.
    let names_look = patch.paper.is_some() || patch.color.is_some();
    let look = if names_look {
        Some(look_after(conn, id, patch)?)
    } else {
        None
    };
    let mut update = Update::new(tables::SNAPS);
    update.set_some("text", patch.text.as_ref());
    update.set_some(columns::STACK_ID, stack_id.as_ref());
    if let Some((paper, color)) = &look {
        update.set("paper", paper.as_str());
        update.set("color", color);
    }
    update.set_some("pinned", patch.pinned);
    let mut updated = update.run(conn, id, WRITTEN)?.ok_or_else(not_found)?;
    updated.tags = tags::of(conn, SourceType::Snap, id)?;
    Ok(updated)
}

/// Makes the stack and moves both snaps in; callers run it in one transaction.
pub fn pair(conn: &Connection, pair: &Pair) -> AppResult<Stack> {
    let new = NewStack {
        parent: pair.parent,
        name: "",
    };
    let stack = stacks::create(conn, &new)?;
    let into = Patch {
        stack_id: Some(Some(stack.id.clone())),
        ..Default::default()
    };
    for id in pair.snaps {
        update(conn, id, &into)?;
    }
    Ok(stack)
}

/// Deletes the snap; its tags come off with it (trigger `snaps_drop_links`).
pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    let deleted = conn.execute("DELETE FROM snaps WHERE id = ?1", [id])?;
    if deleted == 0 {
        return Err(not_found());
    }
    Ok(())
}
