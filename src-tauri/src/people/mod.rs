//! People: the list used for assignees and the People section.

mod checks;
mod contacts;

#[cfg(test)]
mod tests;

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::db::{columns, tables, Returning, Update};
use crate::error::{errors, AppError, AppResult};
use crate::util::{new_id, now_ms};

pub use contacts::{
    add as contact_add, delete as contact_delete, update as contact_update, Contact, ContactEdit,
    NewContact,
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PersonRow {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub nickname: Option<String>,
    pub dob_year: Option<i64>,
    pub dob_month: Option<i64>,
    pub dob_day: Option<i64>,
    /// First phone and email, for the list columns and search.
    pub phone: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    #[serde(flatten)]
    pub row: PersonRow,
    pub tob: Option<String>,
    pub notes: String,
    pub contacts: Vec<Contact>,
}

/// Fields the person page edits. Absent fields stay as they are; `null` clears optional ones.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Patch {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    #[serde(default, with = "crate::util::nullable")]
    pub nickname: Option<Option<String>>,
    /// Date of birth as a whole; `null` clears it.
    #[serde(default, with = "crate::util::nullable")]
    pub dob: Option<Option<Dob>>,
    #[serde(default, with = "crate::util::nullable")]
    pub tob: Option<Option<String>>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Dob {
    pub year: Option<i64>,
    pub month: i64,
    pub day: i64,
}

/// The list columns. Phone and email are each kind's first contact.
const ROW_SQL: &str = "SELECT id, first_name, last_name, nickname, dob_year, dob_month, dob_day,
    (SELECT value FROM person_contacts c WHERE c.person_id = p.id AND c.kind = 'phone'
     ORDER BY position LIMIT 1),
    (SELECT value FROM person_contacts c WHERE c.person_id = p.id AND c.kind = 'email'
     ORDER BY position LIMIT 1)
    FROM people p";

fn to_row(r: &rusqlite::Row) -> rusqlite::Result<PersonRow> {
    Ok(PersonRow {
        id: r.get(0)?,
        first_name: r.get(1)?,
        last_name: r.get(2)?,
        nickname: r.get(3)?,
        dob_year: r.get(4)?,
        dob_month: r.get(5)?,
        dob_day: r.get(6)?,
        phone: r.get(7)?,
        email: r.get(8)?,
    })
}

pub fn list(conn: &Connection) -> AppResult<Vec<PersonRow>> {
    let rows = conn
        .prepare(&format!(
            "{ROW_SQL} ORDER BY first_name COLLATE NOCASE, last_name COLLATE NOCASE"
        ))?
        .query_map([], to_row)?
        .collect::<Result<_, _>>()?;
    Ok(rows)
}

fn not_found() -> AppError {
    errors::NOT_FOUND.with("Person was not found")
}

/// A person to add.
pub struct NewPerson<'a> {
    pub first: &'a str,
    pub last: &'a str,
}

/// Returns the new person's id.
pub fn create(conn: &Connection, new: &NewPerson) -> AppResult<String> {
    let first = checks::first_name(new.first)?;
    let id = new_id();
    conn.execute(
        "INSERT INTO people (id, first_name, last_name, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?4)",
        params![id, first, new.last.trim(), now_ms()],
    )?;
    Ok(id)
}

/// A person's own columns: everything of `Person` but the contacts.
struct Own {
    id: String,
    first_name: String,
    last_name: String,
    nickname: Option<String>,
    dob_year: Option<i64>,
    dob_month: Option<i64>,
    dob_day: Option<i64>,
    tob: Option<String>,
    notes: String,
}

const OWN_COLS: &str =
    "id, first_name, last_name, nickname, dob_year, dob_month, dob_day, tob, notes";

fn to_own(r: &rusqlite::Row) -> rusqlite::Result<Own> {
    Ok(Own {
        id: r.get(0)?,
        first_name: r.get(1)?,
        last_name: r.get(2)?,
        nickname: r.get(3)?,
        dob_year: r.get(4)?,
        dob_month: r.get(5)?,
        dob_day: r.get(6)?,
        tob: r.get(7)?,
        notes: r.get(8)?,
    })
}

/// What a write to `people` answers with.
const WRITTEN: Returning<Own> = Returning {
    columns: OWN_COLS,
    map: to_own,
};

/// The person with their contacts (one more query: they live in their own table).
fn with_contacts(conn: &Connection, own: Own) -> AppResult<Person> {
    let contacts = contacts::of(conn, &own.id)?;
    // Contacts come in order within each kind, so the first of a kind is the one the list shows.
    let first = |kind: &str| {
        let found = contacts.iter().find(|c| c.kind == kind);
        found.map(|c| c.value.clone())
    };
    let row = PersonRow {
        phone: first(contacts::PHONE),
        email: first(contacts::EMAIL),
        id: own.id,
        first_name: own.first_name,
        last_name: own.last_name,
        nickname: own.nickname,
        dob_year: own.dob_year,
        dob_month: own.dob_month,
        dob_day: own.dob_day,
    };
    Ok(Person {
        row,
        tob: own.tob,
        notes: own.notes,
        contacts,
    })
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Person> {
    let own = conn
        .query_row(
            &format!("SELECT {OWN_COLS} FROM people WHERE id = ?1"),
            [id],
            to_own,
        )
        .optional()?
        .ok_or_else(not_found)?;
    with_contacts(conn, own)
}

pub fn update(conn: &Connection, id: &str, p: &Patch) -> AppResult<Person> {
    checks::patch(p)?;
    let mut update = Update::new(tables::PEOPLE);
    update.set_some("first_name", p.first_name.as_deref().map(str::trim));
    update.set_some("last_name", p.last_name.as_deref().map(str::trim));
    if let Some(nickname) = &p.nickname {
        update.set("nickname", clean_nickname(nickname.as_deref()));
    }
    if let Some(dob) = &p.dob {
        update.set("dob_year", dob.and_then(|d| d.year));
        update.set("dob_month", dob.map(|d| d.month));
        update.set("dob_day", dob.map(|d| d.day));
    }
    update.set_some("tob", p.tob.as_ref());
    update.set_some(columns::NOTES, p.notes.as_ref());
    let own = update.run(conn, id, WRITTEN)?.ok_or_else(not_found)?;
    with_contacts(conn, own)
}

/// A blank nickname is stored as none.
fn clean_nickname(nickname: Option<&str>) -> Option<&str> {
    nickname.map(str::trim).filter(|n| !n.is_empty())
}

/// Deleting a person unassigns their tasks (ON DELETE SET NULL) and drops their contacts.
pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    let deleted = conn.execute("DELETE FROM people WHERE id = ?1", [id])?;
    if deleted == 0 {
        return Err(not_found());
    }
    Ok(())
}
