//! freedictionaryapi.com (English Wiktionary, CC BY-SA 4.0): its URL, its reply (only what is
//! read) and the mapper to `Dictionary`. It has idioms and antonyms but no recordings; a word it
//! doesn't have is a reply with no entries. Its licence asks for a link to the entry's
//! Wiktionary page, which the reply gives (`source.url`).

use serde::Deserialize;

use super::{
    first_unique, non_empty, unreadable, Dictionary, Sense, Source, MAX_ANTONYMS, MAX_SENSES,
    MAX_SYNONYMS,
};
use crate::error::AppResult;

const API: &str = "https://freedictionaryapi.com/api/v1/entries/en/";
/// The `type` of a pronunciation written in the phonetic alphabet.
const IPA: &str = "ipa";
/// How an "example" that is really a thesaurus line starts ("Near-synonym: brief").
const NYM_LABELS: [&str; 8] = [
    "synonym",
    "near-synonym",
    "antonym",
    "hypernym",
    "hyponym",
    "coordinate term",
    "holonym",
    "meronym",
];

/// The lookup URL for a word already made one path segment.
pub(super) fn url(word: &str) -> String {
    format!("{API}{word}")
}

#[derive(Deserialize)]
struct Reply {
    #[serde(default)]
    entries: Vec<Entry>,
    source: Option<Origin>,
}

#[derive(Deserialize)]
struct Origin {
    url: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Entry {
    part_of_speech: Option<String>,
    #[serde(default)]
    pronunciations: Vec<Pronunciation>,
    #[serde(default)]
    senses: Vec<Meaning>,
    #[serde(default)]
    synonyms: Vec<String>,
    #[serde(default)]
    antonyms: Vec<String>,
}

#[derive(Deserialize)]
struct Pronunciation {
    #[serde(rename = "type")]
    kind: Option<String>,
    text: Option<String>,
}

/// One sense of an entry.
#[derive(Deserialize)]
struct Meaning {
    definition: String,
    #[serde(default)]
    examples: Vec<String>,
    #[serde(default)]
    quotes: Vec<Quote>,
    #[serde(default)]
    synonyms: Vec<String>,
    #[serde(default)]
    antonyms: Vec<String>,
}

#[derive(Deserialize)]
struct Quote {
    text: Option<String>,
}

fn ipa_of(entries: &[Entry]) -> Option<String> {
    let pronunciations = entries.iter().flat_map(|e| &e.pronunciations);
    pronunciations
        .filter(|p| p.kind.as_deref() == Some(IPA))
        .find_map(|p| non_empty(&p.text))
}

/// Whether an example reads as a sentence: not a thesaurus line, not a citation that starts with
/// its year ("1821, Vicesimus Knox, …").
fn is_usage(example: &str) -> bool {
    let lower = example.trim().to_lowercase();
    let is_nym_line = NYM_LABELS.iter().any(|label| lower.starts_with(label));
    let starts_with_year = lower.starts_with(|c: char| c.is_ascii_digit());
    !lower.is_empty() && !is_nym_line && !starts_with_year
}

/// A sense's example: its first usage example, else the text of its first quote.
fn example_of(meaning: &Meaning) -> Option<String> {
    let usage = meaning.examples.iter().find(|e| is_usage(e));
    let quoted = meaning.quotes.iter().find_map(|q| non_empty(&q.text));
    usage.map(|e| e.trim().to_string()).or(quoted)
}

fn senses_of(entries: &[Entry]) -> Vec<Sense> {
    let meanings = entries.iter().flat_map(|e| &e.senses);
    meanings
        .filter(|m| !m.definition.trim().is_empty())
        .take(MAX_SENSES)
        .map(|m| Sense {
            definition: m.definition.trim().to_string(),
            example: example_of(m),
        })
        .collect()
}

/// An entry's own synonyms, then each of its senses'.
fn synonyms_of(entries: &[Entry]) -> Vec<String> {
    let all = entries.iter().flat_map(|e| {
        let own = e.senses.iter().flat_map(|s| &s.synonyms);
        e.synonyms.iter().chain(own)
    });
    first_unique(all, MAX_SYNONYMS)
}

fn antonyms_of(entries: &[Entry]) -> Vec<String> {
    let all = entries.iter().flat_map(|e| {
        let own = e.senses.iter().flat_map(|s| &s.antonyms);
        e.antonyms.iter().chain(own)
    });
    first_unique(all, MAX_ANTONYMS)
}

/// The entry's page as a link that opens: the reply writes a phrase's page with its spaces
/// (`…/wiki/bite the bullet`).
fn link(url: String) -> String {
    url.replace(' ', "%20")
}

pub(super) fn from_json(body: &str, fetched_at: i64) -> AppResult<Option<Dictionary>> {
    let reply: Reply = serde_json::from_str(body).map_err(unreadable)?;
    let senses = senses_of(&reply.entries);
    if senses.is_empty() {
        return Ok(None);
    }
    let first_kind = reply
        .entries
        .iter()
        .find_map(|e| non_empty(&e.part_of_speech));
    Ok(Some(Dictionary {
        ipa: ipa_of(&reply.entries),
        audio: None,
        part_of_speech: first_kind,
        senses,
        synonyms: synonyms_of(&reply.entries),
        antonyms: antonyms_of(&reply.entries),
        source: Source::FreedictionaryapiCom,
        source_url: reply.source.and_then(|s| non_empty(&s.url)).map(link),
        fetched_at,
    }))
}
