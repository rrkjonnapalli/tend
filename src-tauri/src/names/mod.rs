//! The name rule for folders and note files, and helpers for relative paths ('' is the root).

#[cfg(test)]
mod tests;

use crate::error::{errors, AppResult};

/// Lowercase letters and digits, with single underscores only between them:
/// `^[a-z0-9]+(_[a-z0-9]+)*$`.
pub fn is_valid(name: &str) -> bool {
    let allowed = name
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    let edges_ok = !name.starts_with('_') && !name.ends_with('_');
    !name.is_empty() && allowed && edges_ok && !name.contains("__")
}

/// The name a title gets on disk: lowercase, every run of other characters becomes one `_`,
/// trimmed at both ends ("Window config!" → `window_config`). `None` when nothing is left.
pub fn slug(title: &str) -> Option<String> {
    let mut out = String::new();
    let mut gap = false;
    for c in title.chars().flat_map(char::to_lowercase) {
        let keep = c.is_ascii_lowercase() || c.is_ascii_digit();
        if !keep {
            gap = true;
            continue;
        }
        if gap && !out.is_empty() {
            out.push('_');
        }
        gap = false;
        out.push(c);
    }
    if out.is_empty() {
        return None;
    }
    Some(out)
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
