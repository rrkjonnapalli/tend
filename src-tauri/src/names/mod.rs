//! The name rule for folders and note files, and helpers for relative paths ('' is the root).
//! Names are lowercase letters and digits with single underscores between them
//! (`^[a-z0-9]+(_[a-z0-9]+)*$`). Anything typed or found on disk is turned into such a name by
//! `slug`; a name taken in its folder gets a suffix (`plan_1`).

#[cfg(test)]
mod tests;

use unicode_normalization::char::is_combining_mark;
use unicode_normalization::UnicodeNormalization;

use crate::error::{errors, AppResult};

/// Longest name in characters; leaves room for `.md` and a suffix under 255-byte limits.
const MAX_CHARS: usize = 200;
/// How many `_1`, `_2`, … suffixes to try before giving up.
const MAX_SUFFIX: u32 = 9999;

/// `^[a-z0-9]+(_[a-z0-9]+)*$`.
pub fn is_valid(name: &str) -> bool {
    let allowed = name
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    let edges_ok = !name.starts_with('_') && !name.ends_with('_');
    !name.is_empty() && allowed && edges_ok && !name.contains("__") && name.len() <= MAX_CHARS
}

/// The name that text becomes on disk: accents become plain letters, everything lowercase, each
/// run of other characters one `_`, trimmed, cut to the limit ("Café notes (Q4)!" →
/// `cafe_notes_q4`). `None` when no letter or digit is left.
pub fn slug(text: &str) -> Option<String> {
    let mut out = String::new();
    let mut gap = false;
    let letters = text
        .nfd()
        .filter(|&c| !is_combining_mark(c))
        .flat_map(char::to_lowercase);
    for c in letters {
        if !(c.is_ascii_lowercase() || c.is_ascii_digit()) {
            gap = true;
            continue;
        }
        if out.len() >= MAX_CHARS {
            break;
        }
        if gap && !out.is_empty() {
            out.push('_');
        }
        gap = false;
        out.push(c);
    }
    (!out.is_empty()).then_some(out)
}

/// A folder name typed by the user, as it will be saved.
pub fn accept(name: &str) -> AppResult<String> {
    if name.trim().is_empty() {
        return Err(errors::NAME_EMPTY.err());
    }
    slug(name).ok_or_else(|| errors::NAME_CHARS.err())
}

/// A folder path typed by the user, each segment as it will be saved. '' (the root) is fine.
pub fn accept_path(path: &str) -> AppResult<String> {
    let trimmed = path.trim().trim_matches('/');
    if trimmed.is_empty() {
        return Ok(String::new());
    }
    let segments = trimmed
        .split('/')
        .map(accept)
        .collect::<AppResult<Vec<_>>>()?;
    Ok(segments.join("/"))
}

/// `base`, or `base_1`, `base_2`, … — the first one `taken` reports as free.
pub fn free(base: &str, taken: impl Fn(&str) -> AppResult<bool>) -> AppResult<String> {
    if !taken(base)? {
        return Ok(base.to_string());
    }
    for n in 1..=MAX_SUFFIX {
        let candidate = format!("{base}_{n}");
        if !taken(&candidate)? {
            return Ok(candidate);
        }
    }
    Err(errors::NAME_TAKEN.err())
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
