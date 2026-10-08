//! Notes. A note is a record (`notes`) whose body is a markdown file under `notes/`, held by an
//! attachment (`attachments.path`). The files are the truth: a note's name is its file name
//! without `.md`. Every change made inside Tend writes to disk first, then to the
//! database in one transaction, and puts the disk back if the database step fails.

mod edit;
mod scan;
pub mod watcher;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod watcher_tests;

use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

use crate::error::{errors, AppResult};
use crate::files;
use crate::names;
use crate::util::now_ms;

pub use edit::{
    create, delete_folder, delete_note, folder_create, move_folder, move_note, rename_folder,
    rename_note,
};
pub use scan::scan;

/// Items missing from disk stay in the index this long, then they are purged.
pub const GRACE_MS: i64 = 7 * 24 * 60 * 60 * 1000;
const EXT: &str = ".md";
/// Note bodies live under this folder of the vault; attachment paths start with it.
const PREFIX: &str = "notes/";
const MIME: &str = "text/markdown";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NoteRow {
    pub id: String,
    pub folder: String,
    /// File name without `.md`; what the app shows as the note's title.
    pub name: String,
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

/// `folder` + `name` → the path relative to `notes/` ("work/plan.md").
fn rel_path(folder: &str, name: &str) -> String {
    names::join(folder, &format!("{name}{EXT}"))
}

/// The attachment path of a note body: "notes/work/plan.md".
fn vault_path(folder: &str, name: &str) -> String {
    format!("{PREFIX}{}", rel_path(folder, name))
}

/// "notes/work/plan.md" → ("work", "plan"): a note's folder and name from its body's path.
pub fn split_vault_path(path: &str) -> (String, String) {
    let rel = path.strip_prefix(PREFIX).unwrap_or(path);
    let file = names::base(rel);
    let name = file.strip_suffix(EXT).unwrap_or(file).to_string();
    (names::parent(rel).to_string(), name)
}

fn file_path(dir: &Path, folder: &str, name: &str) -> PathBuf {
    dir.join(rel_path(folder, name))
}

/// Where an indexed note's body is on disk.
fn note_path(dir: &Path, note: &NoteRow) -> PathBuf {
    file_path(dir, &note.folder, &note.name)
}

const ROW_SQL: &str = "SELECT n.id, a.path, max(n.updated_at, a.updated_at)
    FROM notes n JOIN attachments a ON a.id = n.attachment_id";

fn to_row(r: &rusqlite::Row) -> rusqlite::Result<NoteRow> {
    let path: String = r.get(1)?;
    let (folder, name) = split_vault_path(&path);
    Ok(NoteRow {
        id: r.get(0)?,
        folder,
        name,
        modified_at: r.get(2)?,
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
            "{ROW_SQL} WHERE a.missing_at IS NULL ORDER BY a.path"
        ))?
        .query_map([], to_row)?
        .collect::<Result<_, _>>()?;
    Ok(Tree { folders, notes })
}

pub fn row(conn: &Connection, id: &str) -> AppResult<NoteRow> {
    conn.query_row(
        &format!("{ROW_SQL} WHERE n.id = ?1 AND a.missing_at IS NULL"),
        [id],
        to_row,
    )
    .optional()?
    .ok_or_else(|| errors::NOT_FOUND.with("Note was not found"))
}

fn attachment_of(conn: &Connection, note_id: &str) -> AppResult<String> {
    let id = conn.query_row(
        "SELECT attachment_id FROM notes WHERE id = ?1",
        [note_id],
        |r| r.get(0),
    )?;
    Ok(id)
}

pub fn read(conn: &Connection, dir: &Path, id: &str) -> AppResult<NoteDoc> {
    let row = row(conn, id)?;
    let text = fs::read_to_string(note_path(dir, &row))?;
    let tags = crate::tags::of(conn, crate::tags::Owner::Note, id)?;
    Ok(NoteDoc { row, text, tags })
}

/// Writes the body and records the file's new facts.
pub fn write(conn: &Connection, dir: &Path, id: &str, text: &str) -> AppResult<NoteRow> {
    let note = row(conn, id)?;
    let path = note_path(dir, &note);
    fs::write(&path, text)?;
    let facts = files::facts(&path)?;
    let now = now_ms();
    conn.execute(
        "UPDATE attachments SET size_bytes = ?1, sha256 = ?2, mtime = ?3, updated_at = ?4
         WHERE id = ?5",
        params![
            facts.size,
            facts.sha256,
            facts.mtime,
            now,
            attachment_of(conn, id)?
        ],
    )?;
    row(conn, id)
}

/// A note row with a body path and no file, for tests in other modules.
#[cfg(test)]
pub fn seed(conn: &Connection, id: &str, path: &str) {
    conn.execute(
        "INSERT INTO attachments (id, path, name, mime_type, size_bytes, sha256, created_at, updated_at)
         VALUES (?1, ?2, ?2, 'text/markdown', 0, '', 0, 0)",
        params![format!("a_{id}"), path],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO notes (id, attachment_id, created_at, updated_at) VALUES (?1, ?2, 0, 0)",
        params![id, format!("a_{id}")],
    )
    .unwrap();
}
