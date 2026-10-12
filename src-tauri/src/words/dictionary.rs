//! The dictionary lookup (dictionaryapi.dev): what a word means, how it sounds, its synonyms.
//! Mapped to Tend's own shape (`Dictionary`), which is what a word stores.

use serde::{Deserialize, Serialize};

use crate::error::{errors, AppResult};
use crate::http;
use crate::util::now_ms;

const API: &str = "https://api.dictionaryapi.dev/api/v2/entries/en/";
/// An entry is a few KB; a very common word can run past 100 KB.
const READ_LIMIT: u64 = 512 * 1024;
const MAX_SENSES: usize = 6;
const MAX_SYNONYMS: usize = 8;

/// What a word keeps from the lookup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dictionary {
    pub ipa: Option<String>,
    /// A recording of the word, when there is one.
    pub audio: Option<String>,
    pub part_of_speech: Option<String>,
    pub senses: Vec<Sense>,
    pub synonyms: Vec<String>,
    pub fetched_at: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sense {
    pub definition: String,
    pub example: Option<String>,
}

// ---- the API's shape (only what is read)

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Entry {
    phonetic: Option<String>,
    #[serde(default)]
    phonetics: Vec<Phonetic>,
    #[serde(default)]
    meanings: Vec<Meaning>,
}

#[derive(Deserialize)]
struct Phonetic {
    text: Option<String>,
    audio: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Meaning {
    part_of_speech: Option<String>,
    #[serde(default)]
    definitions: Vec<Definition>,
    #[serde(default)]
    synonyms: Vec<String>,
}

#[derive(Deserialize)]
struct Definition {
    definition: String,
    example: Option<String>,
    #[serde(default)]
    synonyms: Vec<String>,
}

/// A path segment with everything but unreserved characters percent-encoded (`bite the bullet` →
/// `bite%20the%20bullet`).
pub(super) fn segment(text: &str) -> String {
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

/// Looks the title up (in lowercase); `None` when the dictionary doesn't have it.
pub fn lookup(title: &str) -> AppResult<Option<Dictionary>> {
    let word = title.trim().to_lowercase();
    let url = format!("{API}{}", segment(&word));
    let Some(body) = http::get(&url, READ_LIMIT, errors::WORD_LOOKUP)? else {
        return Ok(None);
    };
    from_json(&body, now_ms())
}

/// Tend's shape of an API reply; `None` when it holds no definition.
pub(super) fn from_json(body: &str, fetched_at: i64) -> AppResult<Option<Dictionary>> {
    let entries: Vec<Entry> =
        serde_json::from_str(body).map_err(|e| errors::WORD_LOOKUP.with(e.to_string()))?;
    Ok(from_entries(&entries, fetched_at))
}

fn non_empty(text: &Option<String>) -> Option<String> {
    let text = text.as_deref().map(str::trim).filter(|t| !t.is_empty());
    text.map(String::from)
}

/// A recording's URL as https (the API sometimes leaves out the scheme: `//ssl.gstatic.com/…`).
fn https(url: String) -> String {
    match url.strip_prefix("//") {
        Some(rest) => format!("https://{rest}"),
        None => url,
    }
}

fn ipa_of(entries: &[Entry]) -> Option<String> {
    let phonetics = entries.iter().flat_map(|e| &e.phonetics);
    let from_list = phonetics.filter_map(|p| non_empty(&p.text)).next();
    entries
        .iter()
        .find_map(|e| non_empty(&e.phonetic))
        .or(from_list)
}

fn audio_of(entries: &[Entry]) -> Option<String> {
    let phonetics = entries.iter().flat_map(|e| &e.phonetics);
    phonetics
        .filter_map(|p| non_empty(&p.audio))
        .map(https)
        .next()
}

fn synonyms_of(meanings: &[&Meaning]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for meaning in meanings {
        let own = meaning.definitions.iter().flat_map(|d| &d.synonyms);
        for synonym in meaning.synonyms.iter().chain(own) {
            if out.len() < MAX_SYNONYMS && !out.contains(synonym) {
                out.push(synonym.clone());
            }
        }
    }
    out
}

fn from_entries(entries: &[Entry], fetched_at: i64) -> Option<Dictionary> {
    let meanings: Vec<&Meaning> = entries.iter().flat_map(|e| &e.meanings).collect();
    let senses: Vec<Sense> = meanings
        .iter()
        .flat_map(|m| &m.definitions)
        .take(MAX_SENSES)
        .map(|d| Sense {
            definition: d.definition.trim().to_string(),
            example: non_empty(&d.example),
        })
        .collect();
    if senses.is_empty() {
        return None;
    }
    Some(Dictionary {
        ipa: ipa_of(entries),
        audio: audio_of(entries),
        part_of_speech: meanings.iter().find_map(|m| non_empty(&m.part_of_speech)),
        senses,
        synonyms: synonyms_of(&meanings),
        fetched_at,
    })
}
