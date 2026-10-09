//! Facts about a file on disk that the attachments index keeps: hash, size and mtime.

use std::fs;
use std::io::Read;
use std::path::Path;
use std::time::UNIX_EPOCH;

use sha2::{Digest, Sha256};

use crate::error::AppResult;

const READ_CHUNK: usize = 64 * 1024;

/// Files this size or larger are left alone: the scan doesn't index a note this big, a file this
/// big can't be attached, and sync never commits one.
pub const MAX_FILE_BYTES: u64 = 30 * 1024 * 1024;

/// True for a file Tend works with (under `MAX_FILE_BYTES`).
pub fn within_limit(size: u64) -> bool {
    size < MAX_FILE_BYTES
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
