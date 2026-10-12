//! Words: anything come across and worth understanding (a word, idiom, phrase, term …). A title is
//! unique by exact match; a `WORD` title is saved in lowercase, other kinds keep the case typed, so
//! `it` (word) and `IT` (abbr) are different entries.

mod dictionary;
mod kinds;

#[cfg(test)]
mod tests;

use rusqlite::{params, Connection, OptionalExtension, ToSql, Transaction};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::enums::SourceType;
use crate::error::{errors, AppResult};
use crate::tags;
use crate::util::{new_id, now_ms};

pub use dictionary::{lookup, Dictionary};
pub use kinds::{
    add as kind_add, delete as kind_delete, list as kinds, rename as kind_rename,
    reorder as kind_reorder, Kind,
};

/// The kind whose titles are saved in lowercase; new words get it unless told otherwise.
pub const WORD: &str = "WORD";
/// Can't be deleted; a word whose kind is gone (after a sync) gets it.
pub const OTHER: &str = "OTHER";

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Word {
    pub id: String,
    pub title: String,
    pub kind: String,
    /// What the dictionary lookup returned; `None` when not looked up or not found.
    pub dictionary: Option<Value>,
    pub meaning: String,
    pub notes: String,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// A word to add. Only the title is needed.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewWord {
    pub title: String,
    pub kind: Option<String>,
    pub dictionary: Option<Value>,
    #[serde(default)]
    pub meaning: String,
    #[serde(default)]
    pub notes: String,
}

/// Fields an edit can change; absent fields stay. `dictionary: null` clears the lookup.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Patch {
    pub title: Option<String>,
    pub kind: Option<String>,
    #[serde(default, with = "crate::util::nullable")]
    pub dictionary: Option<Option<Value>>,
    pub meaning: Option<String>,
    pub notes: Option<String>,
}

const COLS: &str = "id, title, kind, dictionary, meaning, notes, created_at, updated_at";

/// A row as stored: `dictionary` still JSON text, tags not read yet.
type Stored = (
    String,
    String,
    String,
    Option<String>,
    String,
    String,
    i64,
    i64,
);

fn to_stored(r: &rusqlite::Row) -> rusqlite::Result<Stored> {
    Ok((
        r.get(0)?,
        r.get(1)?,
        r.get(2)?,
        r.get(3)?,
        r.get(4)?,
        r.get(5)?,
        r.get(6)?,
        r.get(7)?,
    ))
}

fn to_word(stored: Stored, tags: Vec<String>) -> AppResult<Word> {
    let (id, title, kind, dictionary, meaning, notes, created_at, updated_at) = stored;
    let dictionary = dictionary.map(|d| serde_json::from_str(&d)).transpose()?;
    Ok(Word {
        id,
        title,
        kind,
        dictionary,
        meaning,
        notes,
        tags,
        created_at,
        updated_at,
    })
}

/// Every word, alphabetical (ignoring case), with its tags.
pub fn list(conn: &Connection) -> AppResult<Vec<Word>> {
    let mut tags = tags::by_record(conn, SourceType::Word)?;
    let stored: Vec<Stored> = conn
        .prepare(&format!(
            "SELECT {COLS} FROM words ORDER BY title COLLATE NOCASE, title"
        ))?
        .query_map([], to_stored)?
        .collect::<Result<_, _>>()?;
    stored
        .into_iter()
        .map(|s| {
            let own = tags.remove(&s.0).unwrap_or_default();
            to_word(s, own)
        })
        .collect()
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Word> {
    let stored = conn
        .query_row(
            &format!("SELECT {COLS} FROM words WHERE id = ?1"),
            [id],
            to_stored,
        )
        .optional()?
        .ok_or_else(|| errors::NOT_FOUND.with("Word was not found"))?;
    to_word(stored, tags::of(conn, SourceType::Word, id)?)
}

fn check_kind(conn: &Connection, kind: &str) -> AppResult<()> {
    if !kinds::exists(conn, kind)? {
        return Err(errors::WORD_KIND_UNKNOWN.err());
    }
    Ok(())
}

/// The title as saved: trimmed, and lowercase for a `WORD`.
fn saved_title(title: &str, kind: &str) -> AppResult<String> {
    let title = title.trim();
    if title.is_empty() {
        return Err(errors::WORD_TITLE_EMPTY.err());
    }
    if kind == WORD {
        return Ok(title.to_lowercase());
    }
    Ok(title.to_string())
}

/// Refuses a title another word (not `except`) already has.
fn check_free(conn: &Connection, title: &str, except: Option<&str>) -> AppResult<()> {
    let taken = conn
        .query_row(
            "SELECT 1 FROM words WHERE title = ?1 AND id IS NOT ?2",
            params![title, except],
            |_| Ok(()),
        )
        .optional()?;
    if taken.is_some() {
        return Err(errors::WORD_TAKEN.with(format!("Already saved: “{title}”.")));
    }
    Ok(())
}

fn json_text(value: &Option<Value>) -> AppResult<Option<String>> {
    Ok(value.as_ref().map(serde_json::to_string).transpose()?)
}

pub fn create(conn: &Connection, new: &NewWord) -> AppResult<Word> {
    let kind = new.kind.as_deref().unwrap_or(WORD);
    check_kind(conn, kind)?;
    let title = saved_title(&new.title, kind)?;
    check_free(conn, &title, None)?;
    let id = new_id();
    conn.execute(
        "INSERT INTO words (id, title, kind, dictionary, meaning, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        params![
            id,
            title,
            kind,
            json_text(&new.dictionary)?,
            new.meaning,
            new.notes,
            now_ms()
        ],
    )?;
    get(conn, &id)
}

/// The title and kind after `patch`: a new kind or title is checked, and a title that changes
/// (lowercased by a move to `WORD`, say) must be free.
fn checked_naming(conn: &Connection, current: &Word, patch: &Patch) -> AppResult<(String, String)> {
    let kind = patch.kind.clone().unwrap_or_else(|| current.kind.clone());
    check_kind(conn, &kind)?;
    let typed = patch.title.as_deref().unwrap_or(&current.title);
    let title = saved_title(typed, &kind)?;
    if title != current.title {
        check_free(conn, &title, Some(&current.id))?;
    }
    Ok((title, kind))
}

pub fn update(conn: &mut Connection, id: &str, patch: &Patch) -> AppResult<Word> {
    let current = get(conn, id)?;
    let (title, kind) = checked_naming(conn, &current, patch)?;
    let tx = conn.transaction()?;
    set(&tx, id, "title", &title)?;
    set(&tx, id, "kind", &kind)?;
    if let Some(dictionary) = &patch.dictionary {
        set(&tx, id, "dictionary", &json_text(dictionary)?)?;
    }
    if let Some(meaning) = &patch.meaning {
        set(&tx, id, "meaning", meaning)?;
    }
    if let Some(notes) = &patch.notes {
        set(&tx, id, "notes", notes)?;
    }
    set(&tx, id, "updated_at", &now_ms())?;
    tx.commit()?;
    get(conn, id)
}

/// Writes one column. `column` is always one of the literals in `update`, never user input.
fn set(tx: &Transaction, id: &str, column: &'static str, value: &dyn ToSql) -> AppResult<()> {
    tx.execute(
        &format!("UPDATE words SET {column} = ?1 WHERE id = ?2"),
        params![value, id],
    )?;
    Ok(())
}

/// Deletes the word; its tags come off with it (trigger `words_drop_links`).
pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM words WHERE id = ?1", [id])?;
    Ok(())
}
