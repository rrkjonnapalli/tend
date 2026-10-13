//! dictionaryapi.dev: its URL, its reply (only what is read) and the mapper to `Dictionary`.
//! It has recordings; a word it doesn't have is a 404.

use serde::Deserialize;

use super::{
    first_unique, non_empty, unreadable, Dictionary, Sense, Source, MAX_ANTONYMS, MAX_SENSES,
    MAX_SYNONYMS,
};
use crate::error::AppResult;

const API: &str = "https://api.dictionaryapi.dev/api/v2/entries/en/";

/// The lookup URL for a word already made one path segment.
pub(super) fn url(word: &str) -> String {
    format!("{API}{word}")
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Entry {
    phonetic: Option<String>,
    #[serde(default)]
    phonetics: Vec<Phonetic>,
    #[serde(default)]
    meanings: Vec<Meaning>,
    #[serde(default)]
    source_urls: Vec<String>,
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
    #[serde(default)]
    antonyms: Vec<String>,
}

#[derive(Deserialize)]
struct Definition {
    definition: String,
    example: Option<String>,
    #[serde(default)]
    synonyms: Vec<String>,
    #[serde(default)]
    antonyms: Vec<String>,
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

/// A meaning's own synonyms, then each of its definitions'.
fn synonyms_of(meanings: &[&Meaning]) -> Vec<String> {
    let all = meanings.iter().flat_map(|m| {
        let own = m.definitions.iter().flat_map(|d| &d.synonyms);
        m.synonyms.iter().chain(own)
    });
    first_unique(all, MAX_SYNONYMS)
}

fn antonyms_of(meanings: &[&Meaning]) -> Vec<String> {
    let all = meanings.iter().flat_map(|m| {
        let own = m.definitions.iter().flat_map(|d| &d.antonyms);
        m.antonyms.iter().chain(own)
    });
    first_unique(all, MAX_ANTONYMS)
}

fn senses_of(meanings: &[&Meaning]) -> Vec<Sense> {
    let definitions = meanings.iter().flat_map(|m| &m.definitions);
    definitions
        .take(MAX_SENSES)
        .map(|d| Sense {
            definition: d.definition.trim().to_string(),
            example: non_empty(&d.example),
        })
        .collect()
}

pub(super) fn from_json(body: &str, fetched_at: i64) -> AppResult<Option<Dictionary>> {
    let entries: Vec<Entry> = serde_json::from_str(body).map_err(unreadable)?;
    let meanings: Vec<&Meaning> = entries.iter().flat_map(|e| &e.meanings).collect();
    let senses = senses_of(&meanings);
    if senses.is_empty() {
        return Ok(None);
    }
    Ok(Some(Dictionary {
        ipa: ipa_of(&entries),
        audio: audio_of(&entries),
        part_of_speech: meanings.iter().find_map(|m| non_empty(&m.part_of_speech)),
        senses,
        synonyms: synonyms_of(&meanings),
        antonyms: antonyms_of(&meanings),
        source: Source::DictionaryapiDev,
        source_url: entries.iter().flat_map(|e| &e.source_urls).next().cloned(),
        fetched_at,
    }))
}
