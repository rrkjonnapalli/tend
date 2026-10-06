//! Phone numbers, emails and addresses of a person, kept in order within each kind.

use rusqlite::{params, Connection};
use serde::Serialize;

use super::checks;
use crate::error::AppResult;
use crate::util::{new_id, now_ms};

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    pub id: String,
    pub kind: String,
    pub value: String,
    pub label: String,
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

pub fn add(conn: &Connection, person: &str, kind: &str, value: &str, label: &str) -> AppResult<()> {
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

pub fn update(conn: &Connection, id: &str, value: &str, label: &str) -> AppResult<()> {
    let kind: String = conn.query_row(
        "SELECT kind FROM person_contacts WHERE id = ?1",
        [id],
        |r| r.get(0),
    )?;
    let value = checks::contact(&kind, value)?;
    conn.execute(
        "UPDATE person_contacts SET value = ?1, label = ?2, updated_at = ?3 WHERE id = ?4",
        params![value, label.trim(), now_ms(), id],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM person_contacts WHERE id = ?1", [id])?;
    Ok(())
}
