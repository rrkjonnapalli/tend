//! Attachments: files in the vault, each with its path, size and sha256, and links from a task,
//! note, comment or person to one. Added files are copied into `.tend/attachments/<id>.<ext>`; a
//! note's body is an attachment too (its file in the vault), so linking a note links its body. The UI only ever
//! sees ids.

#[cfg(test)]
mod tests;

use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::Serialize;

use crate::db::commit_or_undo;
use crate::enums::{SourceType, TargetType};
use crate::error::{errors, AppResult};
use crate::files::{self, FileFacts};
use crate::links::{self, Source, Target};
use crate::util::{new_id, now_ms};

/// Where added files are copied, relative to the vault root.
const STORE: &str = crate::vault::ATTACHMENTS_DIR;

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub id: String,
    /// "note" when the attachment is a note's body, else "file".
    pub kind: String,
    pub attachment_id: Option<String>,
    pub note_id: Option<String>,
    /// File name, or the note's name (its file name without `.md`).
    pub name: String,
    pub mime_type: Option<String>,
    pub size_bytes: Option<i64>,
    /// The note's folder, for note links.
    pub folder: Option<String>,
    pub created_at: i64,
}

fn check_owner(owner: &str) -> AppResult<SourceType> {
    SourceType::parse(owner)
        .ok_or_else(|| errors::OWNER_TYPE.with(format!("Unknown owner type {owner}")))
}

/// A file copied into the store, ready to be recorded.
pub struct Stored {
    id: String,
    path: String,
    name: String,
    mime_type: String,
    facts: FileFacts,
}

/// A file that can be attached: a plain file under the size limit.
pub fn check_file(src: &Path, limit: files::Limit) -> AppResult<()> {
    let meta = fs::metadata(src).map_err(|_| errors::NOT_FOUND.with("File was not found"))?;
    if !meta.is_file() {
        return Err(errors::NOT_FOUND.with("Only files can be attached"));
    }
    if !limit.allows(meta.len()) {
        let mb = limit.mb();
        return Err(
            errors::FILE_TOO_BIG.with(format!("Files of {mb} MB or more can’t be attached."))
        );
    }
    Ok(())
}

/// A file to copy into a vault's store.
pub struct Incoming<'a> {
    /// The vault.
    pub root: &'a Path,
    /// The file being attached.
    pub src: &'a Path,
    pub limit: files::Limit,
}

/// Copies the file into the store and hashes it, without the database (this is the slow part). A
/// failed copy leaves nothing behind. Link it with `link_copy`.
pub fn copy_in(&Incoming { root, src, limit }: &Incoming) -> AppResult<Stored> {
    check_file(src, limit)?;
    let name = src
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or_else(|| errors::NOT_FOUND.with("File was not found"))?;
    let id = new_id();
    let path = format!("{STORE}/{id}{}", extension(src));
    let dest = root.join(&path);
    fs::create_dir_all(root.join(STORE))?;
    let facts = fs::copy(src, &dest)
        .map_err(Into::into)
        .and_then(|_| files::facts(&dest));
    match facts {
        Ok(facts) => Ok(Stored {
            id,
            path,
            name,
            mime_type: mime_of(src),
            facts,
        }),
        Err(e) => {
            let _ = fs::remove_file(&dest);
            Err(e)
        }
    }
}

/// A copied file and what it is attached to.
pub struct Copied<'a> {
    /// The vault.
    pub root: &'a Path,
    pub file: &'a Stored,
    /// A `SourceType` value: `TASK` or `PERSON`.
    pub owner: &'a str,
    pub owner_id: &'a str,
}

/// Records a copied file and links it to its owner; the copy is removed if that fails, so the
/// store never keeps a file the database doesn't know.
pub fn link_copy(conn: &mut Connection, copied: &Copied) -> AppResult<()> {
    let Copied {
        root,
        file,
        owner,
        owner_id,
    } = *copied;
    let owner = check_owner(owner)?;
    let dest = root.join(&file.path);
    let remove_copy = || {
        let _ = fs::remove_file(&dest);
    };
    commit_or_undo(conn, remove_copy, |tx| record(tx, file, owner, owner_id))
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

fn record(tx: &Transaction, file: &Stored, owner: SourceType, owner_id: &str) -> AppResult<()> {
    let now = now_ms();
    tx.execute(
        "INSERT INTO attachments
           (id, path, name, mime_type, size_bytes, sha256, created_at, updated_at, mtime)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, ?8)",
        params![
            file.id,
            file.path,
            file.name,
            file.mime_type,
            file.facts.size,
            file.facts.sha256,
            now,
            file.facts.mtime
        ],
    )?;
    insert_link(tx, owner, owner_id, &file.id, now)
}

fn insert_link(
    conn: &Connection,
    owner: SourceType,
    owner_id: &str,
    attachment_id: &str,
    now: i64,
) -> AppResult<()> {
    let source = Source {
        kind: owner,
        id: owner_id,
    };
    links::insert(conn, source, Target::attachment(attachment_id), now)
}

/// Links the note's body. Linking the same note twice keeps one link.
pub fn link_note(conn: &Connection, owner: &str, owner_id: &str, note_id: &str) -> AppResult<()> {
    let owner = check_owner(owner)?;
    let attachment_id: String = conn
        .query_row(
            "SELECT attachment_id FROM notes WHERE id = ?1",
            [note_id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| errors::NOT_FOUND.with("Note was not found"))?;
    insert_link(conn, owner, owner_id, &attachment_id, now_ms())
}

fn to_link(r: &rusqlite::Row) -> rusqlite::Result<Link> {
    let note_id: Option<String> = r.get(2)?;
    let path: String = r.get(3)?;
    let is_note = note_id.is_some();
    let (folder, note_name) = crate::notes::split_vault_path(&path);
    let name = if is_note { note_name } else { r.get(4)? };
    Ok(Link {
        id: r.get(0)?,
        kind: if is_note { "note" } else { "file" }.to_string(),
        attachment_id: (!is_note).then(|| r.get(1)).transpose()?,
        note_id,
        name,
        mime_type: (!is_note).then(|| r.get(5)).transpose()?,
        size_bytes: (!is_note).then(|| r.get(6)).transpose()?,
        folder: is_note.then_some(folder),
        created_at: r.get(7)?,
    })
}

pub fn list(conn: &Connection, owner: &str, owner_id: &str) -> AppResult<Vec<Link>> {
    let owner = check_owner(owner)?.as_str();
    let rows = conn
        .prepare(
            "SELECT l.id, a.id, n.id, a.path, a.name, a.mime_type,
                    a.size_bytes, l.created_at
             FROM tend_links l
             JOIN attachments a ON a.id = l.target_value
             LEFT JOIN notes n ON n.attachment_id = a.id
             WHERE l.target_type = ?1 AND l.source_type = ?2 AND l.source_id = ?3
             ORDER BY l.created_at, l.rowid",
        )?
        .query_map(
            params![TargetType::Attachment.as_str(), owner, owner_id],
            to_link,
        )?
        .collect::<Result<_, _>>()?;
    Ok(rows)
}

/// Removes the link only; the stored file stays.
pub fn unlink(conn: &Connection, link_id: &str) -> AppResult<()> {
    links::delete_by_id(conn, link_id)
}

/// Where an attachment's file lives on disk.
pub fn file_path(conn: &Connection, root: &Path, attachment_id: &str) -> AppResult<PathBuf> {
    let path: String = conn
        .query_row(
            "SELECT path FROM attachments WHERE id = ?1",
            [attachment_id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| errors::NOT_FOUND.with("Attachment was not found"))?;
    // A row that came by sync could point anywhere; only open what is inside the vault.
    if !files::stays_in_vault(&path) {
        return Err(errors::OPEN_FAILED.with("This attachment's path is outside the vault"));
    }
    Ok(root.join(path))
}
