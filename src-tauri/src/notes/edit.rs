//! Changes made inside Tend: create, rename, move and delete notes and folders.
//! Disk first, then the database; the disk step is undone if the database step fails.

use std::fs;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Transaction};

use super::{attachment_of, file_path, note_path, row, vault_path, NoteRow, EXT, MIME, PREFIX};
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

/// `name` follows the rule and no other folder in `parent` uses it (ignoring case, on disk or
/// in the index). `current` is the folder's own name when renaming it, which counts as free.
fn check_folder_name(
    conn: &Connection,
    dir: &Path,
    parent: &str,
    name: &str,
    current: Option<&str>,
) -> AppResult<()> {
    names::check(name)?;
    if current.is_some_and(|c| c.eq_ignore_ascii_case(name)) {
        return Ok(());
    }
    let path = names::join(parent, name);
    if folder_exists(conn, &path)? || dir.join(&path).is_dir() {
        return Err(errors::NAME_TAKEN.err());
    }
    Ok(())
}

/// The same for a note's name in `folder`.
fn check_note_name(
    conn: &Connection,
    dir: &Path,
    folder: &str,
    name: &str,
    current: Option<&str>,
) -> AppResult<()> {
    names::check(name)?;
    if current.is_some_and(|c| c.eq_ignore_ascii_case(name)) {
        return Ok(());
    }
    if note_exists(conn, folder, name)? || file_path(dir, folder, name).exists() {
        return Err(errors::NAME_TAKEN.err());
    }
    Ok(())
}

/// Creates `folder` first when it isn't there yet (moves and new notes can target new folders).
fn ensure_folder(conn: &mut Connection, dir: &Path, folder: &str) -> AppResult<()> {
    if folder_exists(conn, folder)? {
        return Ok(());
    }
    folder_create(conn, dir, folder)
}

/// Drops index rows left behind by missing items at `path`, so a new item can take the name.
fn clear_missing(tx: &Transaction, path: &str) -> AppResult<()> {
    let prefix = format!("{PREFIX}{path}");
    tx.execute(
        "DELETE FROM attachments WHERE missing_at IS NOT NULL
         AND (path = ?1 || ?2 OR substr(path, 1, length(?1) + 1) = ?1 || '/')",
        params![prefix, EXT],
    )?;
    tx.execute(
        "DELETE FROM note_folders WHERE missing_at IS NOT NULL
         AND (path = ?1 OR substr(path, 1, length(?1) + 1) = ?1 || '/')",
        [path],
    )?;
    Ok(())
}

/// Creates `path` and any missing parents. Each segment must follow the name rule.
pub fn folder_create(conn: &mut Connection, dir: &Path, path: &str) -> AppResult<()> {
    names::check_path(path)?;
    if path.is_empty() {
        return Ok(());
    }
    check_folder_name(conn, dir, names::parent(path), names::base(path), None)?;
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

/// A new, empty note called `name` in `folder` (the file is `<name>.md`).
pub fn create(conn: &mut Connection, dir: &Path, folder: &str, name: &str) -> AppResult<NoteRow> {
    check_note_name(conn, dir, folder, name, None)?;
    ensure_folder(conn, dir, folder)?;
    let path = file_path(dir, folder, name);
    fs::write(&path, "")?;
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

/// Renames the note's file (links point at the note's id, so nothing breaks).
pub fn rename_note(conn: &mut Connection, dir: &Path, id: &str, name: &str) -> AppResult<NoteRow> {
    let note = row(conn, id)?;
    if note.name == name {
        return Ok(note);
    }
    check_note_name(conn, dir, &note.folder, name, Some(&note.name))?;
    relocate_note(conn, dir, &note, &note.folder, name)
}

pub fn move_note(conn: &mut Connection, dir: &Path, id: &str, target: &str) -> AppResult<NoteRow> {
    let note = row(conn, id)?;
    if note.folder == target {
        return Err(errors::SAME_FOLDER.err());
    }
    names::check_path(target)?;
    check_note_name(conn, dir, target, &note.name, None)?;
    ensure_folder(conn, dir, target)?;
    let name = note.name.clone();
    relocate_note(conn, dir, &note, target, &name)
}

fn relocate_note(
    conn: &mut Connection,
    dir: &Path,
    note: &NoteRow,
    folder: &str,
    name: &str,
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
            tx.execute(
                "UPDATE attachments SET path = ?1, name = ?2, updated_at = ?3 WHERE id = ?4",
                params![
                    vault_path(folder, name),
                    format!("{name}{EXT}"),
                    now_ms(),
                    attachment_id
                ],
            )?;
            Ok(())
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
    if names::base(path) == name {
        return Ok(path.to_string());
    }
    let parent = names::parent(path);
    check_folder_name(conn, dir, parent, name, Some(names::base(path)))?;
    relocate_folder(conn, dir, path, &names::join(parent, name))
}

pub fn move_folder(
    conn: &mut Connection,
    dir: &Path,
    path: &str,
    target: &str,
) -> AppResult<String> {
    if names::is_under(target, path) {
        return Err(errors::INTO_SELF.err());
    }
    if names::parent(path) == target {
        return Err(errors::SAME_FOLDER.err());
    }
    names::check_path(target)?;
    let name = names::base(path);
    check_folder_name(conn, dir, target, name, None)?;
    ensure_folder(conn, dir, target)?;
    relocate_folder(conn, dir, path, &names::join(target, name))
}

fn relocate_folder(conn: &mut Connection, dir: &Path, old: &str, new: &str) -> AppResult<String> {
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
                params![format!("{PREFIX}{old}/"), format!("{PREFIX}{new}/"), now],
            )?;
            Ok(())
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
            [format!("{PREFIX}{path}/")],
        )?;
        tx.execute(
            "DELETE FROM note_folders WHERE path = ?1 OR substr(path, 1, length(?1) + 1) = ?1 || '/'",
            [path],
        )?;
        Ok(())
    })
}
