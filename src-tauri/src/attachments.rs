//! Attachments: files copied into `.attachments/<id>.<ext>` with their sha256, and links from a
//! task, note, comment or person to a file or to a note. The UI only ever sees ids.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use sha2::{Digest, Sha256};

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
    if OWNERS.contains(&owner) {
        Ok(())
    } else {
        Err(AppError::rule(
            "bad_owner",
            format!("Unknown owner type {owner}"),
        ))
    }
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

/// Copies `src` into the store and links it to the owner. The copy is removed if the database step fails.
pub fn add_file(
    conn: &mut Connection,
    store: &Path,
    owner: &str,
    owner_id: &str,
    src: &Path,
) -> AppResult<()> {
    check_owner(owner)?;
    let name = src
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or_else(|| AppError::NotFound("File".into()))?;
    let id = new_id();
    let ext = src
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()))
        .unwrap_or_default();
    let stored = format!("{id}{ext}");
    let dest = store.join(&stored);
    fs::create_dir_all(store)?;
    fs::copy(src, &dest)?;
    let size = fs::metadata(&dest)?.len() as i64;
    let mime = mime_guess::from_path(src)
        .first_or_octet_stream()
        .essence_str()
        .to_string();
    let hash = sha256(&dest)?;
    let now = now_ms();
    let result = (|| -> AppResult<()> {
        let tx = conn.transaction()?;
        tx.execute(
            "INSERT INTO attachments (id, original_name, stored_name, mime_type, size_bytes, sha256, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, name, stored, mime, size, hash, now],
        )?;
        tx.execute(
            "INSERT INTO attachment_links (id, owner_type, owner_id, kind, attachment_id, created_at)
             VALUES (?1, ?2, ?3, 'file', ?4, ?5)",
            params![new_id(), owner, owner_id, id, now],
        )?;
        tx.commit()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&dest);
    }
    result
}

pub fn link_note(conn: &Connection, owner: &str, owner_id: &str, note_id: &str) -> AppResult<()> {
    check_owner(owner)?;
    let exists = conn
        .query_row(
            "SELECT 1 FROM attachment_links WHERE owner_type = ?1 AND owner_id = ?2 AND note_id = ?3",
            params![owner, owner_id, note_id],
            |_| Ok(()),
        )
        .optional()?
        .is_some();
    if !exists {
        conn.execute(
            "INSERT INTO attachment_links (id, owner_type, owner_id, kind, note_id, created_at)
             VALUES (?1, ?2, ?3, 'note', ?4, ?5)",
            params![new_id(), owner, owner_id, note_id, now_ms()],
        )?;
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_memory;

    #[test]
    fn files_are_copied_hashed_and_linked() {
        let tmp = tempfile::tempdir().unwrap();
        let store = tmp.path().join(".attachments");
        let src = tmp.path().join("Invoice March.PDF");
        fs::write(&src, b"hello").unwrap();
        let mut conn = open_memory();
        let t = crate::tasks::create(&mut conn, "", "T", None, None, None).unwrap();

        add_file(&mut conn, &store, "task", &t.id, &src).unwrap();
        let links = list(&conn, "task", &t.id).unwrap();
        assert_eq!(links.len(), 1);
        let l = &links[0];
        assert_eq!(
            (l.kind.as_str(), l.name.as_str(), l.size_bytes),
            ("file", "Invoice March.PDF", Some(5))
        );
        assert_eq!(l.mime_type.as_deref(), Some("application/pdf"));
        let path = file_path(&conn, &store, l.attachment_id.as_deref().unwrap()).unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"hello");
        assert!(path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .ends_with(".pdf"));
        let hash: String = conn
            .query_row("SELECT sha256 FROM attachments", [], |r| r.get(0))
            .unwrap();
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );

        unlink(&conn, &l.id).unwrap();
        assert!(list(&conn, "task", &t.id).unwrap().is_empty());
        assert!(path.exists(), "unlinking keeps the file");
        assert!(add_file(&mut conn, &store, "nope", &t.id, &src).is_err());
    }

    #[test]
    fn notes_link_once() {
        let mut conn = open_memory();
        let t = crate::tasks::create(&mut conn, "", "T", None, None, None).unwrap();
        conn.execute(
            "INSERT INTO note_folders (path, created_at) VALUES ('work', 0)",
            [],
        )
        .unwrap();
        conn.execute("INSERT INTO notes (id, folder, file_name, title, created_at, modified_at) VALUES ('n', 'work', 'plan.md', 'Plan', 0, 0)", [])
            .unwrap();
        link_note(&conn, "task", &t.id, "n").unwrap();
        link_note(&conn, "task", &t.id, "n").unwrap();
        let links = list(&conn, "task", &t.id).unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(
            (links[0].name.as_str(), links[0].folder.as_deref()),
            ("plan", Some("work"))
        );
    }
}
