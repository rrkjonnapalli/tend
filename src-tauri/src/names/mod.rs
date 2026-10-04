//! The name rule for folders and note files, and helpers for relative paths ('' is the root).

#[cfg(test)]
mod tests;

use crate::error::{errors, AppResult};

pub fn is_valid(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

pub fn check(name: &str) -> AppResult<()> {
    if name.is_empty() {
        return Err(errors::NAME_EMPTY.err());
    }
    if !is_valid(name) {
        return Err(errors::NAME_CHARS.err());
    }
    Ok(())
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
