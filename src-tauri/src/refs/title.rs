//! A link's page title: fetched over HTTP(S) and read from the page's `<title>`. Only the start of
//! the page is read, and a slow site gives up after a few seconds.

use std::io::Read;
use std::time::Duration;

use crate::error::{errors, AppResult};

const TIMEOUT: Duration = Duration::from_secs(5);
/// The title sits in `<head>`, near the top; nothing past this is read.
const READ_LIMIT: u64 = 64 * 1024;
const USER_AGENT: &str = "Tend (page title)";

/// The URL to fetch: as typed when it has a scheme, else https. Only http and https are fetched.
fn fetchable(url: &str) -> AppResult<String> {
    let url = url.trim();
    let lower = url.to_ascii_lowercase();
    if lower.starts_with("https://") || lower.starts_with("http://") {
        return Ok(url.to_string());
    }
    if url.contains("://") {
        return Err(errors::REF_TITLE_FETCH.with("Only http and https links have a page title."));
    }
    Ok(format!("https://{url}"))
}

/// The page's title, or `None` when it has none.
pub fn fetch(url: &str) -> AppResult<Option<String>> {
    let url = fetchable(url)?;
    let fail = |e: ureq::Error| errors::REF_TITLE_FETCH.with(e.to_string());
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(TIMEOUT))
        .build()
        .into();
    let mut response = agent
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .call()
        .map_err(fail)?;
    let mut head = Vec::new();
    response
        .body_mut()
        .as_reader()
        .take(READ_LIMIT)
        .read_to_end(&mut head)
        .map_err(|e| errors::REF_TITLE_FETCH.with(e.to_string()))?;
    Ok(title_of(&String::from_utf8_lossy(&head)))
}

/// The text of the first `<title>` element, entities decoded and spaces collapsed.
pub fn title_of(html: &str) -> Option<String> {
    // ASCII lowercasing keeps every byte where it was, so indexes carry over to `html`.
    let lower = html.to_ascii_lowercase();
    let tag = lower.find("<title")?;
    let open_end = tag + lower[tag..].find('>')? + 1;
    let close = open_end + lower[open_end..].find("</title")?;
    let text = decode(&html[open_end..close]);
    let words: Vec<&str> = text.split_whitespace().collect();
    let title = words.join(" ");
    if title.is_empty() {
        return None;
    }
    Some(title)
}

/// One `&…;` entity as its character; `None` for one this doesn't know.
fn entity(name: &str) -> Option<char> {
    let numeric = name.strip_prefix('#');
    if let Some(number) = numeric {
        let hex = number
            .strip_prefix('x')
            .or_else(|| number.strip_prefix('X'));
        let code = match hex {
            Some(hex) => u32::from_str_radix(hex, 16).ok()?,
            None => number.parse().ok()?,
        };
        return char::from_u32(code);
    }
    match name {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "nbsp" => Some(' '),
        _ => None,
    }
}

/// Replaces the entities a title holds; unknown ones stay as written.
fn decode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let after = &rest[amp + 1..];
        let known = after
            .find(';')
            .and_then(|semi| entity(&after[..semi]).map(|c| (c, semi)));
        match known {
            Some((c, semi)) => {
                out.push(c);
                rest = &after[semi + 1..];
            }
            None => {
                out.push('&');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}
