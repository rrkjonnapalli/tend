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

/// Entries whose names start with this are hidden (`.tend/`, `.git/`, `.DS_Store`): Tend never
/// reads, renames or watches them.
const HIDDEN_PREFIX: char = '.';

pub fn is_hidden(name: &str) -> bool {
    name.starts_with(HIDDEN_PREFIX)
}

/// `^[a-z0-9]+(_[a-z0-9]+)*$`, at most `MAX_CHARS` long.
pub fn is_valid(name: &str) -> bool {
    if name.is_empty() || name.len() > MAX_CHARS {
        return false;
    }
    let allowed = name
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    let edges_ok = !name.starts_with('_') && !name.ends_with('_');
    let single_gaps = !name.contains("__");
    allowed && edges_ok && single_gaps
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
    if out.is_empty() {
        return None;
    }
    Some(out)
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

/// `base`, or `base_1`, `base_2`, … — the first one `taken` reports as free. A suffixed name is
/// cut so it still fits `MAX_CHARS` (an over-long name would fail the rule and be fixed again).
pub fn free(base: &str, taken: impl Fn(&str) -> AppResult<bool>) -> AppResult<String> {
    if !taken(base)? {
        return Ok(base.to_string());
    }
    for n in 1..=MAX_SUFFIX {
        let candidate = with_suffix(base, n);
        if !taken(&candidate)? {
            return Ok(candidate);
        }
    }
    Err(errors::NAME_TAKEN.err())
}

/// `base_n`, with `base` shortened when needed to stay within `MAX_CHARS`. Names are ASCII.
fn with_suffix(base: &str, n: u32) -> String {
    let suffix = format!("_{n}");
    let room = MAX_CHARS - suffix.len();
    let kept = base[..base.len().min(room)].trim_end_matches('_');
    format!("{kept}{suffix}")
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
