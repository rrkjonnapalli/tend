//! Facts about a file on disk that the attachments index keeps: hash, size and mtime.

use std::fs;
use std::io::Read;
use std::path::Path;
use std::time::UNIX_EPOCH;

use sha2::{Digest, Sha256};

use crate::error::AppResult;

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

/// A vault-relative path that stays inside the vault: relative, no `..`, and under `.tend/` only in
/// the attachment store. The empty path is the vault root. Rows that came by sync are checked
/// with this before Tend writes or opens what they point at.
pub fn stays_in_vault(path: &str) -> bool {
    use std::path::Component;
    let p = Path::new(path);
    let plain = p.components().all(|c| matches!(c, Component::Normal(_)));
    let in_tend = p.starts_with(crate::vault::TEND_DIR);
    let in_store = p.starts_with(crate::vault::ATTACHMENTS_DIR);
    plain && (!in_tend || in_store)
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

/// Last-modified time in ms, or 0 when the file can't be read.
pub fn mtime(path: &Path) -> i64 {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_millis() as i64)
}

/// Size, hash and mtime of a file, as the attachments table stores them.
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
