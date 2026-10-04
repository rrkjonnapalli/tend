//! The notes index. Files in `notes/` are the truth; `note_folders` and `notes` mirror them.
//! Every change made inside Tend writes to disk first, then to the database in one transaction,
//! and puts the disk back if the database step fails.

mod edit;
mod scan;
pub mod watcher;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod watcher_tests;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

use crate::error::{AppError, AppResult};
use crate::names;

pub use edit::{
    create, delete_folder, delete_note, folder_create, move_folder, move_note, rename_folder,
    rename_note,
};
pub use scan::scan;

/// Items missing from disk stay in the index this long, then they are purged.
pub const GRACE_MS: i64 = 7 * 24 * 60 * 60 * 1000;
const EXT: &str = ".md";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NoteRow {
    pub id: String,
    pub folder: String,
    /// File name without `.md`.
    pub name: String,
    pub title: String,
    pub modified_at: i64,
}

#[derive(Debug, Serialize)]
pub struct Tree {
    pub folders: Vec<String>,
    pub notes: Vec<NoteRow>,
}

#[derive(Debug, Serialize)]
pub struct NoteDoc {
    #[serde(flatten)]
    pub row: NoteRow,
    pub text: String,
    pub tags: Vec<String>,
}

/// Last-modified time in ms, or 0 when the file can't be read.
fn mtime(path: &Path) -> i64 {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_millis() as i64)
}

/// The first `# ` heading, or the file name without `.md`.
pub fn title_of(text: &str, name: &str) -> String {
    let heading = text
        .lines()
        .find_map(|l| l.strip_prefix("# "))
        .map(str::trim)
        .filter(|t| !t.is_empty());
    match heading {
        Some(title) => title.to_string(),
        None => name.to_string(),
    }
}

fn file_path(dir: &Path, folder: &str, file_name: &str) -> PathBuf {
    dir.join(names::join(folder, file_name))
}

/// Where an indexed note lives on disk.
fn note_path(dir: &Path, note: &NoteRow) -> PathBuf {
    file_path(dir, &note.folder, &format!("{}{EXT}", note.name))
}

const ROW_SQL: &str = "SELECT id, folder, file_name, title, modified_at FROM notes";

fn to_row(r: &rusqlite::Row) -> rusqlite::Result<NoteRow> {
    let file: String = r.get(2)?;
    Ok(NoteRow {
        id: r.get(0)?,
        folder: r.get(1)?,
        name: file.trim_end_matches(EXT).to_string(),
        title: r.get(3)?,
        modified_at: r.get(4)?,
    })
}

pub fn tree(conn: &Connection) -> AppResult<Tree> {
    let folders = conn
        .prepare(
            "SELECT path FROM note_folders WHERE missing_at IS NULL AND path <> '' ORDER BY path",
        )?
        .query_map([], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    let notes = conn
        .prepare(&format!(
            "{ROW_SQL} WHERE missing_at IS NULL ORDER BY folder, file_name"
        ))?
        .query_map([], to_row)?
        .collect::<Result<_, _>>()?;
    Ok(Tree { folders, notes })
}

pub fn row(conn: &Connection, id: &str) -> AppResult<NoteRow> {
    conn.query_row(
        &format!("{ROW_SQL} WHERE id = ?1 AND missing_at IS NULL"),
        [id],
        to_row,
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound("Note".into()))
}

pub fn read(conn: &Connection, dir: &Path, id: &str) -> AppResult<NoteDoc> {
    let row = row(conn, id)?;
    let text = fs::read_to_string(note_path(dir, &row))?;
    let tags = crate::tags::of(conn, crate::tags::Owner::Note, id)?;
    Ok(NoteDoc { row, text, tags })
}

pub fn write(conn: &Connection, dir: &Path, id: &str, text: &str) -> AppResult<NoteRow> {
    let note = row(conn, id)?;
    let path = note_path(dir, &note);
    fs::write(&path, text)?;
    conn.execute(
        "UPDATE notes SET title = ?1, modified_at = ?2 WHERE id = ?3",
        params![title_of(text, &note.name), mtime(&path), id],
    )?;
    row(conn, id)
}
