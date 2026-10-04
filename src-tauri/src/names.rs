//! The name rule for folders and note files, and helpers for relative paths ('' is the root).

use crate::error::{AppError, AppResult};

pub const EMPTY: &str = "Name can’t be empty";
pub const CHARS: &str = "Only a–z, 0–9 and _ (no spaces or capitals)";
pub const CLASH: &str = "Something with this name already exists here";

pub fn is_valid(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

pub fn check(name: &str) -> AppResult<()> {
    if name.is_empty() {
        return Err(AppError::InvalidName(EMPTY.into()));
    }
    if !is_valid(name) {
        return Err(AppError::InvalidName(CHARS.into()));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_rule() {
        assert!(is_valid("work_2024"));
        for bad in ["", "Work", "my notes", "a-b", "a.b", "é"] {
            assert!(!is_valid(bad), "{bad:?} should be invalid");
        }
        assert!(matches!(check(""), Err(AppError::InvalidName(m)) if m == EMPTY));
        assert!(matches!(check("A"), Err(AppError::InvalidName(m)) if m == CHARS));
    }

    #[test]
    fn paths() {
        assert!(check_path("").is_ok());
        assert!(check_path("work/tauri").is_ok());
        assert!(check_path("work/Tauri").is_err());
        assert!(check_path("work//x").is_err());
        assert_eq!(join("", "a"), "a");
        assert_eq!(join("a", "b"), "a/b");
        assert_eq!(parent("a/b/c"), "a/b");
        assert_eq!(parent("a"), "");
        assert_eq!(base("a/b/c"), "c");
        assert!(is_under("a/b", "a"));
        assert!(!is_under("ab", "a"));
        assert!(is_under("anything", ""));
    }
}
