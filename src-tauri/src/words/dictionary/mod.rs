//! The dictionary lookup: what a word means, how it sounds, its synonyms and antonyms. Each
//! source has its own file with its URL, its reply's shape and its mapper to Tend's shape
//! (`Dictionary`), which is what a word stores. Which source is asked is the caller's choice
//! (Settings → Library).

mod dictionaryapi_dev;
mod freedictionaryapi_com;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

use crate::error::{errors, AppResult};
use crate::http;
use crate::util::now_ms;

/// An entry is a few KB; a very common word can run past 100 KB.
const READ_LIMIT: u64 = 512 * 1024;
const MAX_SENSES: usize = 6;
const MAX_SYNONYMS: usize = 8;
const MAX_ANTONYMS: usize = 8;

/// A dictionary Tend can ask. A word stored before sources were recorded came from the first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Source {
    #[default]
    DictionaryapiDev,
    FreedictionaryapiCom,
}

/// What a word keeps from the lookup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dictionary {
    pub ipa: Option<String>,
    /// A recording of the word, when the source has one.
    pub audio: Option<String>,
    pub part_of_speech: Option<String>,
    pub senses: Vec<Sense>,
    pub synonyms: Vec<String>,
    #[serde(default)]
    pub antonyms: Vec<String>,
    #[serde(default)]
    pub source: Source,
    /// The entry's own page at the source, to credit it.
    #[serde(default)]
    pub source_url: Option<String>,
    pub fetched_at: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sense {
    pub definition: String,
    pub example: Option<String>,
}

/// A path segment with everything but unreserved characters percent-encoded (`bite the bullet` →
/// `bite%20the%20bullet`).
fn segment(text: &str) -> String {
    let mut out = String::new();
    for byte in text.bytes() {
        let plain = byte.is_ascii_alphanumeric() || b"-._~".contains(&byte);
        if plain {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

fn non_empty(text: &Option<String>) -> Option<String> {
    let text = text.as_deref().map(str::trim).filter(|t| !t.is_empty());
    text.map(String::from)
}

/// The first `max` different words of `words`, in order.
fn first_unique<'a>(words: impl Iterator<Item = &'a String>, max: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for word in words {
        if out.len() < max && !out.contains(word) {
            out.push(word.clone());
        }
    }
    out
}

/// A reply that isn't the JSON its source promises: logged, and told as a failed lookup.
fn unreadable(e: serde_json::Error) -> crate::error::AppError {
    eprintln!("tend: could not read the dictionary's reply: {e}");
    errors::WORD_LOOKUP.err()
}

fn url_of(source: Source, word: &str) -> String {
    match source {
        Source::DictionaryapiDev => dictionaryapi_dev::url(word),
        Source::FreedictionaryapiCom => freedictionaryapi_com::url(word),
    }
}

/// Tend's shape of a source's reply; `None` when it holds no definition.
fn from_json(source: Source, body: &str, fetched_at: i64) -> AppResult<Option<Dictionary>> {
    match source {
        Source::DictionaryapiDev => dictionaryapi_dev::from_json(body, fetched_at),
        Source::FreedictionaryapiCom => freedictionaryapi_com::from_json(body, fetched_at),
    }
}

/// Looks the title up (in lowercase) at `source`; `None` when it doesn't have it.
pub fn lookup(title: &str, source: Source) -> AppResult<Option<Dictionary>> {
    let word = title.trim().to_lowercase();
    let url = url_of(source, &segment(&word));
    let Some(body) = http::get(&url, READ_LIMIT, errors::WORD_LOOKUP)? else {
        return Ok(None);
    };
    from_json(source, &body, now_ms())
}
