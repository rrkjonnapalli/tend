//! Notes. A note is a record (`notes`) whose body is a markdown file in the vault, held by an
//! attachment (`attachments.path`, relative to the vault, which is the notes root). The app shows a
//! note by its file name; the title is part of the file (its first `# heading`). The files are the
//! truth. Every change made inside Tend writes to disk first, then to the database in one
//! transaction, and puts the disk back if the database step fails. Every rename or move of a file or
//! folder is written to `path_log`.

mod edit;
mod scan;
pub mod watcher;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod watcher_tests;

use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::Serialize;

use crate::error::{errors, AppResult};
use crate::files;
use crate::names;
use crate::settings;
use crate::util::now_ms;
use serde_json::Value;

pub use edit::{
    create, delete_folder, delete_note, folder_create, move_folder, move_note, rename_folder,
    rename_note,
};
pub use scan::{scan, Skipped};

/// Setting: the vault allows scans to rename folders and notes whose names break the rule.
const FIX_NAMES: &str = "notes.fix_names";

/// Lets scans rename folders and notes whose names break the rule (asked once per vault).
pub fn allow_fix(conn: &Connection) -> AppResult<()> {
    settings::set(conn, FIX_NAMES, &Value::Bool(true))
}

pub fn fix_allowed(conn: &Connection) -> AppResult<bool> {
    Ok(settings::get(conn, FIX_NAMES)? == Some(Value::Bool(true)))
}

/// What a `path_log` row moved.
#[derive(Clone, Copy)]
pub(crate) enum PathKind {
    Folder,
    Note,
}

impl PathKind {
    /// Also the fallback name of an item whose name has no letter or digit.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            PathKind::Folder => "folder",
            PathKind::Note => "note",
        }
    }
}

/// Why a `path_log` row moved it.
#[derive(Clone, Copy)]
pub(crate) enum Reason {
    /// A scan gave a name found on disk a valid one.
    Fix,
    Rename,
    Move,
    /// A sync brought in another device's rename or move.
    Sync,
}

impl Reason {
    fn as_str(self) -> &'static str {
        match self {
            Reason::Fix => "fix",
            Reason::Rename => "rename",
            Reason::Move => "move",
            Reason::Sync => "sync",
        }
    }
}

/// One rename or move of a folder or note file; paths are relative to the vault.
pub(crate) struct Moved {
    pub kind: PathKind,
    pub reason: Reason,
    pub old: String,
    pub new: String,
}

/// Records a rename or move, so an earlier layout can be restored.
pub(crate) fn log_path(tx: &Transaction, moved: &Moved, at: i64) -> AppResult<()> {
    tx.execute(
        "INSERT INTO path_log (kind, reason, old_path, new_path, at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            moved.kind.as_str(),
            moved.reason.as_str(),
            moved.old,
            moved.new,
            at
        ],
    )?;
    Ok(())
}

/// Items missing from disk stay in the index this long, then they are purged.
pub const GRACE_MS: i64 = 7 * 24 * 60 * 60 * 1000;
/// The extension of note files.
pub const EXT: &str = ".md";
const MIME: &str = "text/markdown";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NoteRow {
    pub id: String,
    pub folder: String,
    /// File name without `.md`: what the explorer and lists show.
    pub name: String,
    pub modified_at: i64,
    /// The body's hash as last indexed: what a save expects to replace (`check_unchanged`).
    pub sha256: String,
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

/// The attachment path of a note body, relative to the vault: "work/plan.md".
fn vault_path(folder: &str, name: &str) -> String {
    names::join(folder, &format!("{name}{EXT}"))
}

/// "work/plan.md" → ("work", "plan"): a note's folder and name from its body's path.
pub fn split_vault_path(path: &str) -> (String, String) {
    let file = names::base(path);
    let name = file.strip_suffix(EXT).unwrap_or(file).to_string();
    (names::parent(path).to_string(), name)
}

fn file_path(dir: &Path, folder: &str, name: &str) -> PathBuf {
    dir.join(vault_path(folder, name))
}

/// Where an indexed note's body is on disk.
fn note_path(dir: &Path, note: &NoteRow) -> PathBuf {
    file_path(dir, &note.folder, &note.name)
}

const ROW_SQL: &str = "SELECT n.id, a.path, max(n.updated_at, a.updated_at), a.sha256
    FROM notes n JOIN attachments a ON a.id = n.attachment_id";

fn to_row(r: &rusqlite::Row) -> rusqlite::Result<NoteRow> {
    let path: String = r.get(1)?;
    let (folder, name) = split_vault_path(&path);
    Ok(NoteRow {
        id: r.get(0)?,
        folder,
        name,
        modified_at: r.get(2)?,
        sha256: r.get(3)?,
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
    let tags = crate::tags::of(conn, crate::enums::SourceType::Note, id)?;
    Ok(NoteDoc { row, text, tags })
}

/// Refuses a save over a body that changed since the page read it (`base`, its hash then): another
/// editor or a sync wrote it. A body that is gone from disk has nothing to lose.
pub fn check_unchanged(conn: &Connection, dir: &Path, id: &str, base: &str) -> AppResult<()> {
    let note = row(conn, id)?;
    let now = files::sha256(&note_path(dir, &note)).ok();
    let changed = now.is_some_and(|sha| sha != base);
    if changed {
        return Err(errors::NOTE_CHANGED.err());
    }
    Ok(())
}

/// Writes the body and records the file's new facts.
pub fn write(conn: &Connection, dir: &Path, id: &str, text: &str) -> AppResult<NoteRow> {
    let note = row(conn, id)?;
    let path = note_path(dir, &note);
    files::write_atomic(&path, text.as_bytes())?;
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
