//! People: the list used for assignees and the People section.

mod checks;
mod contacts;

#[cfg(test)]
mod tests;

use rusqlite::{params, Connection, OptionalExtension, ToSql, Transaction};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::util::{new_id, now_ms};

pub use contacts::{
    add as contact_add, delete as contact_delete, update as contact_update, Contact,
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

fn row(conn: &Connection, id: &str) -> AppResult<PersonRow> {
    conn.query_row(&format!("{ROW_SQL} WHERE p.id = ?1"), [id], to_row)
        .optional()?
        .ok_or_else(|| AppError::NotFound("Person".into()))
}

/// Returns the new person's id.
pub fn create(conn: &Connection, first: &str, last: &str) -> AppResult<String> {
    let first = checks::first_name(first)?;
    let id = new_id();
    conn.execute(
        "INSERT INTO people (id, first_name, last_name, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?4)",
        params![id, first, last.trim(), now_ms()],
    )?;
    Ok(id)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Person> {
    let row = row(conn, id)?;
    let (tob, notes) =
        conn.query_row("SELECT tob, notes FROM people WHERE id = ?1", [id], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })?;
    let contacts = contacts::of(conn, id)?;
    Ok(Person {
        row,
        tob,
        notes,
        contacts,
    })
}

pub fn update(conn: &mut Connection, id: &str, p: &Patch) -> AppResult<Person> {
    checks::patch(p)?;
    let tx = conn.transaction()?;
    if let Some(first) = &p.first_name {
        set(&tx, id, "first_name", &first.trim())?;
    }
    if let Some(last) = &p.last_name {
        set(&tx, id, "last_name", &last.trim())?;
    }
    if let Some(nickname) = &p.nickname {
        set(&tx, id, "nickname", &clean_nickname(nickname.as_deref()))?;
    }
    if let Some(dob) = &p.dob {
        set_dob(&tx, id, dob.as_ref())?;
    }
    if let Some(tob) = &p.tob {
        set(&tx, id, "tob", tob)?;
    }
    if let Some(notes) = &p.notes {
        set(&tx, id, "notes", notes)?;
    }
    set(&tx, id, "updated_at", &now_ms())?;
    tx.commit()?;
    get(conn, id)
}

/// Writes one column. `column` is always one of the literals in `update`, never user input.
fn set(tx: &Transaction, id: &str, column: &'static str, value: &dyn ToSql) -> AppResult<()> {
    tx.execute(
        &format!("UPDATE people SET {column} = ?1 WHERE id = ?2"),
        params![value, id],
    )?;
    Ok(())
}

/// A blank nickname is stored as none.
fn clean_nickname(nickname: Option<&str>) -> Option<&str> {
    nickname.map(str::trim).filter(|n| !n.is_empty())
}

fn set_dob(tx: &Transaction, id: &str, dob: Option<&Dob>) -> AppResult<()> {
    let year = dob.and_then(|d| d.year);
    let month = dob.map(|d| d.month);
    let day = dob.map(|d| d.day);
    tx.execute(
        "UPDATE people SET dob_year = ?1, dob_month = ?2, dob_day = ?3 WHERE id = ?4",
        params![year, month, day, id],
    )?;
    Ok(())
}

/// Deleting a person unassigns their tasks (ON DELETE SET NULL) and drops their contacts.
pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM people WHERE id = ?1", [id])?;
    Ok(())
}
