//! Changes made inside Tend: create, rename, move and delete notes and folders.
//! Disk first, then the database; the disk step is undone if the database step fails.

use std::fs;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Transaction};

use super::{file_path, mtime, note_path, row, title_of, NoteRow, EXT};
use crate::db::{commit_or_undo, in_transaction};
use crate::error::{AppError, AppResult};
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

fn note_exists(conn: &Connection, folder: &str, file: &str) -> AppResult<bool> {
    let found = conn
        .query_row(
            "SELECT 1 FROM notes WHERE folder = ?1 AND file_name = ?2 AND missing_at IS NULL",
            params![folder, file],
            |_| Ok(()),
        )
        .optional()?;
    Ok(found.is_some())
}

/// A folder or note called `name` already sits in `folder` (on disk or in the index).
fn clash(conn: &Connection, dir: &Path, folder: &str, name: &str) -> AppResult<bool> {
    let path = names::join(folder, name);
    let file = format!("{name}{EXT}");
    if folder_exists(conn, &path)? || note_exists(conn, folder, &file)? {
        return Ok(true);
    }
    let on_disk = dir.join(&path).exists() || file_path(dir, folder, &file).exists();
    Ok(on_disk)
}

fn check_new(conn: &Connection, dir: &Path, folder: &str, name: &str) -> AppResult<()> {
    names::check(name)?;
    if clash(conn, dir, folder, name)? {
        return Err(AppError::InvalidName(names::CLASH.into()));
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
    tx.execute(
        "DELETE FROM notes WHERE missing_at IS NOT NULL
         AND (folder = ?1 OR substr(folder, 1, length(?1) + 1) = ?1 || '/')",
        [path],
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
    check_new(conn, dir, names::parent(path), names::base(path))?;
    let missing = missing_folders(conn, path)?;
    fs::create_dir_all(dir.join(path))?;
    let now = now_ms();
    in_transaction(conn, |tx| {
        for p in &missing {
            clear_missing(tx, p)?;
            tx.execute(
                "INSERT INTO note_folders (path, created_at) VALUES (?1, ?2)",
                params![p, now],
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

/// The heading a new note starts with: `release_checklist` → `# Release checklist`.
fn heading_for(name: &str) -> String {
    let words = name.replace('_', " ");
    let mut chars = words.chars();
    let Some(first) = chars.next() else {
        return "# \n\n".to_string();
    };
    let title: String = first.to_uppercase().chain(chars).collect();
    format!("# {title}\n\n")
}

pub fn create(conn: &mut Connection, dir: &Path, folder: &str, name: &str) -> AppResult<NoteRow> {
    ensure_folder(conn, dir, folder)?;
    check_new(conn, dir, folder, name)?;
    let file = format!("{name}{EXT}");
    let path = file_path(dir, folder, &file);
    let text = heading_for(name);
    fs::write(&path, &text)?;
    let id = new_id();
    let now = now_ms();
    let modified = mtime(&path);
    commit_or_undo(
        conn,
        || {
            let _ = fs::remove_file(&path);
        },
        |tx| {
            tx.execute(
                "INSERT INTO notes (id, folder, file_name, title, created_at, modified_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![id, folder, file, title_of(&text, name), now, modified],
            )?;
            Ok(())
        },
    )?;
    row(conn, &id)
}

pub fn rename_note(conn: &mut Connection, dir: &Path, id: &str, name: &str) -> AppResult<NoteRow> {
    let note = row(conn, id)?;
    if note.name == name {
        return Ok(note);
    }
    check_new(conn, dir, &note.folder, name)?;
    relocate_note(conn, dir, &note, &note.folder, name)
}

pub fn move_note(conn: &mut Connection, dir: &Path, id: &str, target: &str) -> AppResult<NoteRow> {
    let note = row(conn, id)?;
    if note.folder == target {
        return Err(AppError::rule("same_folder", "It’s already in that folder"));
    }
    ensure_folder(conn, dir, target)?;
    check_new(conn, dir, target, &note.name)?;
    relocate_note(conn, dir, &note, target, &note.name)
}

fn relocate_note(
    conn: &mut Connection,
    dir: &Path,
    note: &NoteRow,
    folder: &str,
    name: &str,
) -> AppResult<NoteRow> {
    let from = note_path(dir, note);
    let file = format!("{name}{EXT}");
    let to = file_path(dir, folder, &file);
    fs::rename(&from, &to)?;
    commit_or_undo(
        conn,
        || {
            let _ = fs::rename(&to, &from);
        },
        |tx| {
            tx.execute(
                "UPDATE notes SET folder = ?1, file_name = ?2 WHERE id = ?3",
                params![folder, file, note.id],
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
    check_new(conn, dir, parent, name)?;
    relocate_folder(conn, dir, path, &names::join(parent, name))
}

pub fn move_folder(
    conn: &mut Connection,
    dir: &Path,
    path: &str,
    target: &str,
) -> AppResult<String> {
    if names::is_under(target, path) {
        return Err(AppError::rule(
            "into_self",
            "A folder can’t move into itself",
        ));
    }
    if names::parent(path) == target {
        return Err(AppError::rule("same_folder", "It’s already in that folder"));
    }
    ensure_folder(conn, dir, target)?;
    let name = names::base(path);
    check_new(conn, dir, target, name)?;
    relocate_folder(conn, dir, path, &names::join(target, name))
}

fn relocate_folder(conn: &mut Connection, dir: &Path, old: &str, new: &str) -> AppResult<String> {
    if old.is_empty() || !folder_exists(conn, old)? {
        return Err(AppError::NotFound("Folder".into()));
    }
    let from = dir.join(old);
    let to = dir.join(new);
    fs::rename(&from, &to)?;
    commit_or_undo(
        conn,
        || {
            let _ = fs::rename(&to, &from);
        },
        |tx| {
            clear_missing(tx, new)?;
            // Notes follow through ON UPDATE CASCADE.
            tx.execute(
                "UPDATE note_folders SET path = ?2 || substr(path, length(?1) + 1)
                 WHERE path = ?1 OR substr(path, 1, length(?1) + 1) = ?1 || '/'",
                params![old, new],
            )?;
            Ok(())
        },
    )?;
    Ok(new.to_string())
}

pub fn delete_note(conn: &mut Connection, dir: &Path, id: &str) -> AppResult<()> {
    let note = row(conn, id)?;
    fs::remove_file(note_path(dir, &note))?;
    in_transaction(conn, |tx| {
        tx.execute("DELETE FROM notes WHERE id = ?1", [id])?;
        Ok(())
    })
}

/// Deletes the folder and everything inside it, from disk and from the index.
pub fn delete_folder(conn: &mut Connection, dir: &Path, path: &str) -> AppResult<()> {
    if path.is_empty() {
        return Err(AppError::rule("root", "The root folder can’t be deleted"));
    }
    if !folder_exists(conn, path)? {
        return Err(AppError::NotFound("Folder".into()));
    }
    fs::remove_dir_all(dir.join(path))?;
    in_transaction(conn, |tx| {
        tx.execute(
            "DELETE FROM notes WHERE folder = ?1 OR substr(folder, 1, length(?1) + 1) = ?1 || '/'",
            [path],
        )?;
        tx.execute(
                "DELETE FROM note_folders WHERE path = ?1 OR substr(path, 1, length(?1) + 1) = ?1 || '/'",
                [path],
            )?;
        Ok(())
    })
}
