//! Phone numbers, emails and addresses of a person, kept in order within each kind.

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

use super::checks;
use crate::error::{errors, AppError, AppResult};
use crate::util::{new_id, now_ms};

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    pub id: String,
    pub kind: String,
    pub value: String,
    pub label: String,
}

/// The kinds whose first contact the people list shows.
pub const PHONE: &str = "phone";
pub const EMAIL: &str = "email";

fn not_found() -> AppError {
    errors::NOT_FOUND.with("Contact was not found")
}

pub fn of(conn: &Connection, person: &str) -> AppResult<Vec<Contact>> {
    let contacts = conn
        .prepare(
            "SELECT id, kind, value, label FROM person_contacts
             WHERE person_id = ?1 ORDER BY kind, position",
        )?
        .query_map([person], |r| {
            Ok(Contact {
                id: r.get(0)?,
                kind: r.get(1)?,
                value: r.get(2)?,
                label: r.get(3)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(contacts)
}

/// A contact to add to a person, after the others of its kind.
pub struct NewContact<'a> {
    pub person: &'a str,
    pub kind: &'a str,
    pub value: &'a str,
    pub label: &'a str,
}

pub fn add(conn: &Connection, new: &NewContact) -> AppResult<()> {
    let NewContact {
        person,
        kind,
        value,
        label,
    } = *new;
    let value = checks::contact(kind, value)?;
    let position: i64 = conn.query_row(
        "SELECT coalesce(max(position) + 1, 0) FROM person_contacts
         WHERE person_id = ?1 AND kind = ?2",
        params![person, kind],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO person_contacts (id, person_id, kind, value, label, position, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        params![
            new_id(),
            person,
            kind,
            value,
            label.trim(),
            position,
            now_ms()
        ],
    )?;
    Ok(())
}

/// A contact and the value and label it gets.
pub struct ContactEdit<'a> {
    pub id: &'a str,
    pub value: &'a str,
    pub label: &'a str,
}

pub fn update(conn: &Connection, edit: &ContactEdit) -> AppResult<()> {
    // The kind as stored says how the value is checked.
    let kind: Option<String> = conn
        .query_row(
            "SELECT kind FROM person_contacts WHERE id = ?1",
            [edit.id],
            |r| r.get(0),
        )
        .optional()?;
    let kind = kind.ok_or_else(not_found)?;
    let value = checks::contact(&kind, edit.value)?;
    conn.execute(
        "UPDATE person_contacts SET value = ?1, label = ?2, updated_at = ?3 WHERE id = ?4",
        params![value, edit.label.trim(), now_ms(), edit.id],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    let deleted = conn.execute("DELETE FROM person_contacts WHERE id = ?1", [id])?;
    if deleted == 0 {
        return Err(not_found());
    }
    Ok(())
}
