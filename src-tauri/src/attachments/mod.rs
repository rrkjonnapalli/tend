//! Attachments: files copied into `.attachments/<id>.<ext>` with their sha256, and links from a
//! task, note, comment or person to a file or to a note. The UI only ever sees ids.

#[cfg(test)]
mod tests;

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::db::commit_or_undo;
use crate::error::{AppError, AppResult};
use crate::util::{new_id, now_ms};

const OWNERS: [&str; 4] = ["task", "note", "comment", "person"];

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub id: String,
    pub kind: String,
    pub attachment_id: Option<String>,
    pub note_id: Option<String>,
    /// File name, or the note's name.
    pub name: String,
    pub mime_type: Option<String>,
    pub size_bytes: Option<i64>,
    /// The note's folder, for note links.
    pub folder: Option<String>,
    pub created_at: i64,
}

fn check_owner(owner: &str) -> AppResult<()> {
    if !OWNERS.contains(&owner) {
        return Err(AppError::rule(
            "bad_owner",
            format!("Unknown owner type {owner}"),
        ));
    }
    Ok(())
}

fn sha256(path: &Path) -> AppResult<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

/// A file copied into the store, ready to be recorded.
struct Stored {
    id: String,
    original_name: String,
    stored_name: String,
    mime_type: String,
    size_bytes: i64,
    sha256: String,
}

/// Copies `src` into the store and links it to the owner. The copy is removed if a later step
/// fails, so the store never keeps a file the database doesn't know.
pub fn add_file(
    conn: &mut Connection,
    store: &Path,
    owner: &str,
    owner_id: &str,
    src: &Path,
) -> AppResult<()> {
    check_owner(owner)?;
    let original_name = src
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or_else(|| AppError::NotFound("File".into()))?;
    let id = new_id();
    let stored_name = format!("{id}{}", extension(src));
    let dest = store.join(&stored_name);
    fs::create_dir_all(store)?;
    fs::copy(src, &dest)?;
    let remove_copy = || {
        let _ = fs::remove_file(&dest);
    };
    let (size_bytes, sha256) = match measure(&dest) {
        Ok(measured) => measured,
        Err(e) => {
            remove_copy();
            return Err(e);
        }
    };
    let file = Stored {
        id,
        original_name,
        stored_name,
        mime_type: mime_of(src),
        size_bytes,
        sha256,
    };
    commit_or_undo(conn, remove_copy, |tx| record(tx, &file, owner, owner_id))
}

/// `.pdf` for `Invoice.PDF`; empty when the file has no extension.
fn extension(src: &Path) -> String {
    match src.extension() {
        Some(ext) => format!(".{}", ext.to_string_lossy().to_lowercase()),
        None => String::new(),
    }
}

fn mime_of(src: &Path) -> String {
    mime_guess::from_path(src)
        .first_or_octet_stream()
        .essence_str()
        .to_string()
}

/// Size in bytes and sha256 of the stored copy.
fn measure(path: &Path) -> AppResult<(i64, String)> {
    let size = fs::metadata(path)?.len() as i64;
    let hash = sha256(path)?;
    Ok((size, hash))
}

fn record(tx: &Transaction, file: &Stored, owner: &str, owner_id: &str) -> AppResult<()> {
    let now = now_ms();
    tx.execute(
        "INSERT INTO attachments (id, original_name, stored_name, mime_type, size_bytes, sha256, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            file.id,
            file.original_name,
            file.stored_name,
            file.mime_type,
            file.size_bytes,
            file.sha256,
            now
        ],
    )?;
    tx.execute(
        "INSERT INTO attachment_links (id, owner_type, owner_id, kind, attachment_id, created_at)
         VALUES (?1, ?2, ?3, 'file', ?4, ?5)",
        params![new_id(), owner, owner_id, file.id, now],
    )?;
    Ok(())
}

pub fn link_note(conn: &Connection, owner: &str, owner_id: &str, note_id: &str) -> AppResult<()> {
    check_owner(owner)?;
    let linked = conn
        .query_row(
            "SELECT 1 FROM attachment_links WHERE owner_type = ?1 AND owner_id = ?2 AND note_id = ?3",
            params![owner, owner_id, note_id],
            |_| Ok(()),
        )
        .optional()?;
    if linked.is_some() {
        return Ok(());
    }
    conn.execute(
        "INSERT INTO attachment_links (id, owner_type, owner_id, kind, note_id, created_at)
         VALUES (?1, ?2, ?3, 'note', ?4, ?5)",
        params![new_id(), owner, owner_id, note_id, now_ms()],
    )?;
    Ok(())
}

pub fn list(conn: &Connection, owner: &str, owner_id: &str) -> AppResult<Vec<Link>> {
    let rows = conn
        .prepare(
            "SELECT l.id, l.kind, l.attachment_id, l.note_id,
                    coalesce(a.original_name, replace(n.file_name, '.md', '')), a.mime_type, a.size_bytes,
                    n.folder, l.created_at
             FROM attachment_links l
             LEFT JOIN attachments a ON a.id = l.attachment_id
             LEFT JOIN notes n ON n.id = l.note_id
             WHERE l.owner_type = ?1 AND l.owner_id = ?2
             ORDER BY l.created_at, l.rowid",
        )?
        .query_map(params![owner, owner_id], |r| {
            Ok(Link {
                id: r.get(0)?,
                kind: r.get(1)?,
                attachment_id: r.get(2)?,
                note_id: r.get(3)?,
                name: r.get(4)?,
                mime_type: r.get(5)?,
                size_bytes: r.get(6)?,
                folder: r.get(7)?,
                created_at: r.get(8)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(rows)
}

/// Removes the link only; the stored file stays.
pub fn unlink(conn: &Connection, link_id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM attachment_links WHERE id = ?1", [link_id])?;
    Ok(())
}

/// Where an attachment's file lives on disk.
pub fn file_path(conn: &Connection, store: &Path, attachment_id: &str) -> AppResult<PathBuf> {
    let stored: String = conn
        .query_row(
            "SELECT stored_name FROM attachments WHERE id = ?1",
            [attachment_id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound("Attachment".into()))?;
    Ok(store.join(stored))
}
