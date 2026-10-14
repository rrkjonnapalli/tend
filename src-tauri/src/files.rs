//! Facts about a file on disk that the attachments index keeps: hash, size and mtime.

use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::time::UNIX_EPOCH;

use sha2::{Digest, Sha256};

use crate::error::AppResult;

#[cfg(test)]
mod tests;

const READ_CHUNK: usize = 64 * 1024;

const MB: u64 = 1024 * 1024;

/// The setting (shared through sync, so every device agrees) for the file size limit, in MB:
/// a note this big isn't indexed, a file this big can't be attached, and sync never commits one.
pub const LIMIT_KEY: &str = "files.max_mb";
pub const DEFAULT_LIMIT_MB: u64 = 30;
/// 0 is not allowed: every note would count as too big and drop out of the vault.
pub const LIMIT_MB_RANGE: std::ops::RangeInclusive<u64> = 1..=90;

/// The default limit in bytes, for tests.
#[cfg(test)]
pub const MAX_FILE_BYTES: u64 = DEFAULT_LIMIT_MB * MB;

/// The vault's file size limit (Settings → General): a file this size or larger is left alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limit(u64);

impl Limit {
    /// The limit the vault has set, or the default.
    pub fn of(conn: &rusqlite::Connection) -> AppResult<Limit> {
        let set = crate::settings::get(conn, LIMIT_KEY)?.and_then(|v| v.as_u64());
        let mb = set
            .filter(|mb| LIMIT_MB_RANGE.contains(mb))
            .unwrap_or(DEFAULT_LIMIT_MB);
        Ok(Limit(mb * MB))
    }

    /// True for a file Tend works with: under the limit.
    pub fn allows(self, size: u64) -> bool {
        size < self.0
    }

    pub fn mb(self) -> u64 {
        self.0 / MB
    }

    #[cfg(test)]
    pub const DEFAULT: Limit = Limit(MAX_FILE_BYTES);
}

/// A vault-relative path that stays inside the vault: relative, no `..`, and no hidden part
/// (`.git/`, `.gitignore`) except the attachment store. The empty path is the vault root. Rows
/// that came by sync are checked with this before Tend writes or opens what they point at.
pub fn stays_in_vault(path: &str) -> bool {
    use std::path::Component;
    let p = Path::new(path);
    let plain = p.components().all(|c| matches!(c, Component::Normal(_)));
    if p.starts_with(crate::vault::ATTACHMENTS_DIR) {
        return plain;
    }
    let shown = p
        .components()
        .all(|c| !crate::names::is_hidden(&c.as_os_str().to_string_lossy()));
    plain && shown
}

/// The entries of `dir` a vault walk looks at, with their names: hidden ones (`.tend/`, `.git/`)
/// and symlinks are left out, so a link loop can't make a walk endless. None if `dir` can't be read.
pub fn shown_entries(dir: &Path) -> Vec<(String, fs::FileType)> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            Some((name, e.file_type().ok()?))
        })
        .filter(|(name, kind)| !crate::names::is_hidden(name) && !kind.is_symlink())
        .collect()
}

pub fn sha256(path: &Path) -> AppResult<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; READ_CHUNK];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex(hasher))
}

/// The hash of bytes in memory, the same way `sha256` hashes a file.
pub fn sha256_of(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex(hasher)
}

fn hex(hasher: Sha256) -> String {
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Replaces the file at `path` with `bytes` all at once: they go to a hidden file beside it, reach
/// the disk, then take its name. A crash or a full disk midway leaves the old file whole, never a
/// cut-off one. A file being replaced keeps its permissions.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> AppResult<()> {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let temp = path.with_file_name(format!(".{name}.tmp"));
    let written = replace_with(&temp, bytes, path);
    if written.is_err() {
        // Only the hidden copy can be broken; the file at `path` is as it was.
        let _ = fs::remove_file(&temp);
    }
    written
}

fn replace_with(temp: &Path, bytes: &[u8], target: &Path) -> AppResult<()> {
    let mut file = fs::File::create(temp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    if let Ok(meta) = fs::metadata(target) {
        fs::set_permissions(temp, meta.permissions())?;
    }
    fs::rename(temp, target)?;
    Ok(())
}

/// Moves a file or folder to the system Trash, where it can be got back. On macOS through
/// `NSFileManager`: Finder's way would ask for permission to control Finder.
#[cfg(all(not(test), desktop))]
pub fn to_trash(path: &Path) -> AppResult<()> {
    let mut trash = trash::TrashContext::default();
    #[cfg(target_os = "macos")]
    {
        use trash::macos::{DeleteMethod, TrashContextExtMacos};
        trash.set_delete_method(DeleteMethod::NsFileManager);
    }
    trash
        .delete(path)
        .map_err(|e| crate::error::errors::TRASH_FAILED.with(e.to_string()))
}

/// Phones have no Trash a user can open; tests delete for real so they never fill it.
// spike: mobile deletes for good. Decide: this, or a `.tend/trash/` folder Tend empties itself.
#[cfg(any(test, mobile))]
pub fn to_trash(path: &Path) -> AppResult<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)?;
        return Ok(());
    }
    fs::remove_file(path)?;
    Ok(())
}

/// Last-modified time in ms, or 0 when the file can't be read.
pub fn mtime(path: &Path) -> i64 {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_millis() as i64)
}

/// Size, hash and mtime of a file, as the attachments table stores them.
#[derive(Clone)]
pub struct FileFacts {
    pub size: i64,
    pub sha256: String,
    pub mtime: i64,
}

pub fn facts(path: &Path) -> AppResult<FileFacts> {
    let size = fs::metadata(path)?.len() as i64;
    Ok(FileFacts {
        size,
        sha256: sha256(path)?,
        mtime: mtime(path),
    })
}
