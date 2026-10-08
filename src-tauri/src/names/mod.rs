//! The name rule for folders and note files, and helpers for relative paths ('' is the root).
//! Names are free text, as long as they are safe as one file or folder name on every OS.

#[cfg(test)]
mod tests;

use crate::error::{errors, AppResult};

/// Longest name in characters; leaves room for `.md` under common 255-byte limits.
const MAX_CHARS: usize = 200;
const FORBIDDEN: [char; 3] = ['/', '\\', ':'];

pub fn check(name: &str) -> AppResult<()> {
    if name.is_empty() {
        return Err(errors::NAME_EMPTY.err());
    }
    if name.trim() != name {
        return Err(errors::NAME_SPACES.err());
    }
    let bad_char = name
        .chars()
        .any(|c| FORBIDDEN.contains(&c) || c.is_control());
    if bad_char || name.starts_with('.') {
        return Err(errors::NAME_CHARS.err());
    }
    if name.chars().count() > MAX_CHARS {
        return Err(errors::NAME_LONG.err());
    }
    Ok(())
}

pub fn is_valid(name: &str) -> bool {
    check(name).is_ok()
}

/// Every segment of a folder path must follow the rule. '' (the root) is fine.
pub fn check_path(path: &str) -> AppResult<()> {
    if path.is_empty() {
        return Ok(());
    }
    path.split('/').try_for_each(check)
}

pub fn join(folder: &str, name: &str) -> String {
    if folder.is_empty() {
        name.to_string()
    } else {
        format!("{folder}/{name}")
    }
}

pub fn parent(path: &str) -> &str {
    path.rfind('/').map_or("", |i| &path[..i])
}

pub fn base(path: &str) -> &str {
    path.rfind('/').map_or(path, |i| &path[i + 1..])
}

/// True when `path` is `root` or inside it.
pub fn is_under(path: &str, root: &str) -> bool {
    root.is_empty() || path == root || path.starts_with(&format!("{root}/"))
}
