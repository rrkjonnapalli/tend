//! Changes made inside Tend: create, rename, move and delete notes and folders.
//! Disk first, then the database; the disk step is undone if the database step fails.

use std::fs;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Transaction};

use super::{
    attachment_of, file_path, log_path, note_path, row, vault_path, NoteRow, PathKind, Reason, EXT,
    MIME,
};
use crate::db::{commit_or_undo, in_transaction};
use crate::error::{errors, AppResult};
use crate::files;
use crate::names;
use crate::util::{new_id, now_ms};

fn folder_exists(conn: &Connection, path: &str) -> AppResult<bool> {
    if path.is_empty() {
        return Ok(true);
    }
    let found = conn
        .query_row(
            "SELECT 1 FROM note_folders WHERE path = ?1 AND missing_at IS NULL",
            [path],
            |_| Ok(()),
        )
        .optional()?;
    Ok(found.is_some())
}

fn note_exists(conn: &Connection, folder: &str, name: &str) -> AppResult<bool> {
    let found = conn
        .query_row(
            "SELECT 1 FROM attachments WHERE path = ?1 AND missing_at IS NULL",
            [vault_path(folder, name)],
            |_| Ok(()),
        )
        .optional()?;
    Ok(found.is_some())
}

/// Another folder in `parent` already uses `name` (on disk or in the index). `current` is the
/// folder's own name when renaming it, which counts as free.
fn folder_taken(
    conn: &Connection,
    dir: &Path,
    parent: &str,
    name: &str,
    current: Option<&str>,
) -> AppResult<bool> {
    if current == Some(name) {
        return Ok(false);
    }
    let path = names::join(parent, name);
    Ok(folder_exists(conn, &path)? || dir.join(&path).is_dir())
}

/// The same for a note's name in `folder`.
fn note_taken(
    conn: &Connection,
    dir: &Path,
    folder: &str,
    name: &str,
    current: Option<&str>,
) -> AppResult<bool> {
    if current == Some(name) {
        return Ok(false);
    }
    Ok(note_exists(conn, folder, name)? || file_path(dir, folder, name).exists())
}

/// Creates `folder` first when it isn't there yet (moves and new notes can target new folders).
fn ensure_folder(conn: &mut Connection, dir: &Path, folder: &str) -> AppResult<()> {
    if folder_exists(conn, folder)? {
        return Ok(());
    }
    create_path(conn, dir, folder)
}

/// Drops index rows left behind by missing items at `path`, so a new item can take the name.
fn clear_missing(tx: &Transaction, path: &str) -> AppResult<()> {
    tx.execute(
        "DELETE FROM attachments WHERE missing_at IS NOT NULL
         AND (path = ?1 || ?2 OR substr(path, 1, length(?1) + 1) = ?1 || '/')",
        params![path, EXT],
    )?;
    tx.execute(
        "DELETE FROM note_folders WHERE missing_at IS NOT NULL
         AND (path = ?1 OR substr(path, 1, length(?1) + 1) = ?1 || '/')",
        [path],
    )?;
    Ok(())
}

/// Creates the folder `path` typed by the user, and any missing parents. When the last segment is
/// taken it gets a suffix. Returns the path that was created.
pub fn folder_create(conn: &mut Connection, dir: &Path, path: &str) -> AppResult<String> {
    let path = names::accept_path(path)?;
    if path.is_empty() {
        return Err(errors::NAME_EMPTY.err());
    }
    let parent = names::parent(&path);
    let name = names::free(names::base(&path), |n| {
        folder_taken(conn, dir, parent, n, None)
    })?;
    let path = names::join(parent, &name);
    create_path(conn, dir, &path)?;
    Ok(path)
}

/// Makes `path` exist on disk and in the index, parents included.
fn create_path(conn: &mut Connection, dir: &Path, path: &str) -> AppResult<()> {
    let missing = missing_folders(conn, path)?;
    fs::create_dir_all(dir.join(path))?;
    let now = now_ms();
    in_transaction(conn, |tx| {
        for p in &missing {
            clear_missing(tx, p)?;
            tx.execute(
                "INSERT INTO note_folders (id, path, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
                params![new_id(), p, now],
            )?;
        }
        Ok(())
    })
}

/// `path` and each of its parents that the index doesn't have yet, outermost first.
fn missing_folders(conn: &Connection, path: &str) -> AppResult<Vec<String>> {
    let mut missing = Vec::new();
    let mut current = String::new();
    for segment in path.split('/') {
        current = names::join(&current, segment);
        if !folder_exists(conn, &current)? {
            missing.push(current.clone());
        }
    }
    Ok(missing)
}

/// The file name a title asks for: its slug, or `note` when it has no letter or digit.
fn slug_of(title: &str) -> String {
    names::slug(title).unwrap_or_else(|| "note".to_string())
}

fn check_title(title: &str) -> AppResult<&str> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(errors::NOTE_TITLE_EMPTY.err());
    }
    Ok(trimmed)
}

/// A new note titled `title` in `folder`. The file is the title's slug (`meeting.md` for
/// "Meeting") and starts with the title as its heading; the title lives only there.
pub fn create(conn: &mut Connection, dir: &Path, folder: &str, title: &str) -> AppResult<NoteRow> {
    let title = check_title(title)?.to_string();
    let folder = names::accept_path(folder)?;
    let folder = folder.as_str();
    ensure_folder(conn, dir, folder)?;
    let name = names::free(&slug_of(&title), |n| note_taken(conn, dir, folder, n, None))?;
    let name = name.as_str();
    let path = file_path(dir, folder, name);
    fs::write(&path, format!("# {title}\n"))?;
    let facts = files::facts(&path)?;
    let note_id = new_id();
    let attachment_id = new_id();
    let now = now_ms();
    commit_or_undo(
        conn,
        || {
            let _ = fs::remove_file(&path);
        },
        |tx| {
            tx.execute(
                "INSERT INTO attachments
                   (id, path, name, mime_type, size_bytes, sha256, created_at, updated_at, mtime)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, ?8)",
                params![
                    attachment_id,
                    vault_path(folder, name),
                    format!("{name}{EXT}"),
                    MIME,
                    facts.size,
                    facts.sha256,
                    now,
                    facts.mtime
                ],
            )?;
            tx.execute(
                "INSERT INTO notes (id, attachment_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
                params![note_id, attachment_id, now],
            )?;
            Ok(())
        },
    )?;
    row(conn, &note_id)
}

/// Renames the note's file to the slug of `name` (links point at the note's id, so nothing breaks).
/// The body, and so the title, is left as it is.
pub fn rename_note(conn: &mut Connection, dir: &Path, id: &str, name: &str) -> AppResult<NoteRow> {
    let wanted = names::accept(name)?;
    let note = row(conn, id)?;
    if note.name == wanted {
        return Ok(note);
    }
    let current = Some(note.name.as_str());
    let name = names::free(&wanted, |n| note_taken(conn, dir, &note.folder, n, current))?;
    relocate_note(conn, dir, &note, &note.folder, &name, Reason::Rename)
}

/// Moves the note into `target`, keeping its file name; a name taken there gets a suffix.
pub fn move_note(conn: &mut Connection, dir: &Path, id: &str, target: &str) -> AppResult<NoteRow> {
    let target = names::accept_path(target)?;
    let note = row(conn, id)?;
    if note.folder == target {
        return Err(errors::SAME_FOLDER.err());
    }
    ensure_folder(conn, dir, &target)?;
    let name = names::free(&note.name, |n| note_taken(conn, dir, &target, n, None))?;
    relocate_note(conn, dir, &note, &target, &name, Reason::Move)
}

fn relocate_note(
    conn: &mut Connection,
    dir: &Path,
    note: &NoteRow,
    folder: &str,
    name: &str,
    reason: Reason,
) -> AppResult<NoteRow> {
    let from = note_path(dir, note);
    let to = file_path(dir, folder, name);
    fs::rename(&from, &to)?;
    let attachment_id = attachment_of(conn, &note.id)?;
    commit_or_undo(
        conn,
        || {
            let _ = fs::rename(&to, &from);
        },
        |tx| {
            let now = now_ms();
            let new_path = vault_path(folder, name);
            tx.execute(
                "UPDATE attachments SET path = ?1, name = ?2, updated_at = ?3 WHERE id = ?4",
                params![new_path, format!("{name}{EXT}"), now, attachment_id],
            )?;
            let old_path = vault_path(&note.folder, &note.name);
            log_path(tx, PathKind::Note, reason, &old_path, &new_path, now)
        },
    )?;
    row(conn, &note.id)
}

pub fn rename_folder(
    conn: &mut Connection,
    dir: &Path,
    path: &str,
    name: &str,
) -> AppResult<String> {
    let typed = names::accept(name)?;
    let current = names::base(path);
    if current == typed {
        return Ok(path.to_string());
    }
    let parent = names::parent(path);
    let name = names::free(&typed, |n| {
        folder_taken(conn, dir, parent, n, Some(current))
    })?;
    relocate_folder(conn, dir, path, &names::join(parent, &name), Reason::Rename)
}

/// Moves the folder into `target`; a name taken there gets a suffix.
pub fn move_folder(
    conn: &mut Connection,
    dir: &Path,
    path: &str,
    target: &str,
) -> AppResult<String> {
    let target = names::accept_path(target)?;
    if names::is_under(&target, path) {
        return Err(errors::INTO_SELF.err());
    }
    if names::parent(path) == target {
        return Err(errors::SAME_FOLDER.err());
    }
    ensure_folder(conn, dir, &target)?;
    let name = names::free(names::base(path), |n| {
        folder_taken(conn, dir, &target, n, None)
    })?;
    relocate_folder(conn, dir, path, &names::join(&target, &name), Reason::Move)
}

fn relocate_folder(
    conn: &mut Connection,
    dir: &Path,
    old: &str,
    new: &str,
    reason: Reason,
) -> AppResult<String> {
    if old.is_empty() || !folder_exists(conn, old)? {
        return Err(errors::NOT_FOUND.with("Folder was not found"));
    }
    let from = dir.join(old);
    let to = dir.join(new);
    fs::rename(&from, &to)?;
    let now = now_ms();
    commit_or_undo(
        conn,
        || {
            let _ = fs::rename(&to, &from);
        },
        |tx| {
            clear_missing(tx, new)?;
            tx.execute(
                "UPDATE note_folders SET path = ?2 || substr(path, length(?1) + 1), updated_at = ?3
                 WHERE path = ?1 OR substr(path, 1, length(?1) + 1) = ?1 || '/'",
                params![old, new, now],
            )?;
            // Note bodies under the folder move with it.
            tx.execute(
                "UPDATE attachments SET path = ?2 || substr(path, length(?1) + 1), updated_at = ?3
                 WHERE substr(path, 1, length(?1)) = ?1",
                params![format!("{old}/"), format!("{new}/"), now],
            )?;
            log_path(tx, PathKind::Folder, reason, old, new, now)
        },
    )?;
    Ok(new.to_string())
}

/// Removes the body and the note (its attachment row goes, and the note with it).
pub fn delete_note(conn: &mut Connection, dir: &Path, id: &str) -> AppResult<()> {
    let note = row(conn, id)?;
    let attachment_id = attachment_of(conn, id)?;
    fs::remove_file(note_path(dir, &note))?;
    in_transaction(conn, |tx| {
        tx.execute("DELETE FROM attachments WHERE id = ?1", [&attachment_id])?;
        Ok(())
    })
}

/// Deletes the folder and everything inside it, from disk and from the index.
pub fn delete_folder(conn: &mut Connection, dir: &Path, path: &str) -> AppResult<()> {
    if path.is_empty() {
        return Err(errors::ROOT_FOLDER.err());
    }
    if !folder_exists(conn, path)? {
        return Err(errors::NOT_FOUND.with("Folder was not found"));
    }
    fs::remove_dir_all(dir.join(path))?;
    in_transaction(conn, |tx| {
        tx.execute(
            "DELETE FROM attachments WHERE substr(path, 1, length(?1)) = ?1",
            [format!("{path}/")],
        )?;
        tx.execute(
            "DELETE FROM note_folders WHERE path = ?1 OR substr(path, 1, length(?1) + 1) = ?1 || '/'",
            [path],
        )?;
        Ok(())
    })
}
