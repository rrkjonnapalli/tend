//! The name rule for folders and notes, and helpers for relative paths ('' is the root).
//! Names use A–Z a–z 0–9, space and `. , _ -`; leading and trailing spaces and dots are trimmed.
//! A name that is taken in its folder gets a suffix (`Plan_1`) instead of an error.

#[cfg(test)]
mod tests;

use unicode_normalization::char::is_combining_mark;
use unicode_normalization::UnicodeNormalization;

use crate::error::{errors, AppResult};

/// Longest name in characters; leaves room for `.md` and a suffix under 255-byte limits.
const MAX_CHARS: usize = 200;
/// How many `_1`, `_2`, … suffixes to try before giving up.
const MAX_SUFFIX: u32 = 9999;

fn allowed(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, ' ' | '.' | ',' | '_' | '-')
}

/// `name` without leading or trailing spaces and dots.
pub fn clean(name: &str) -> &str {
    name.trim_matches(|c| c == ' ' || c == '.')
}

/// Checks a cleaned name.
pub fn check(name: &str) -> AppResult<()> {
    if name.is_empty() {
        return Err(errors::NAME_EMPTY.err());
    }
    if !name.chars().all(allowed) {
        return Err(errors::NAME_CHARS.err());
    }
    if name.chars().count() > MAX_CHARS {
        return Err(errors::NAME_LONG.err());
    }
    Ok(())
}

/// Cleans and checks one name typed by the user.
pub fn accept(name: &str) -> AppResult<String> {
    let cleaned = clean(name);
    check(cleaned)?;
    Ok(cleaned.to_string())
}

/// The valid name closest to `raw`, for a name found on disk that breaks the rule: accents become
/// plain letters ("Café" → "Cafe"), other characters are dropped, runs of spaces collapse, and the
/// result is cleaned and cut to the limit. `None` when nothing usable is left.
pub fn normalize(raw: &str) -> Option<String> {
    let kept: String = raw
        .nfd()
        .filter(|&c| !is_combining_mark(c) && allowed(c))
        .collect();
    let spaced = kept
        .split(' ')
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let cut: String = spaced.chars().take(MAX_CHARS).collect();
    let name = clean(&cut);
    (!name.is_empty()).then(|| name.to_string())
}

/// True for a name found on disk that Tend can index as it is (a scan never renames).
pub fn is_valid(name: &str) -> bool {
    clean(name) == name && check(name).is_ok()
}

/// Cleans and checks every segment of a folder path. '' (the root) is fine.
pub fn accept_path(path: &str) -> AppResult<String> {
    let trimmed = path.trim_matches('/');
    if trimmed.is_empty() {
        return Ok(String::new());
    }
    let segments = trimmed
        .split('/')
        .map(accept)
        .collect::<AppResult<Vec<_>>>()?;
    Ok(segments.join("/"))
}

/// `path` with each segment spelled as an existing folder spells it ("work/new" → "Work/new" when
/// "Work" exists), so differently-cased paths never split one folder in two. `stored` looks a path
/// up ignoring case.
pub fn canonical(
    path: &str,
    stored: impl Fn(&str) -> AppResult<Option<String>>,
) -> AppResult<String> {
    let mut current = String::new();
    for segment in path.split('/').filter(|s| !s.is_empty()) {
        let candidate = join(&current, segment);
        current = stored(&candidate)?.unwrap_or(candidate);
    }
    Ok(current)
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

/// Two names are the same item when they differ only in case (as on macOS).
pub fn same(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
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
