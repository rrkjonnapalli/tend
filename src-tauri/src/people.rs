//! People: the list used for assignees and the People section.

use rusqlite::Connection;
use serde::Serialize;

use crate::error::AppResult;

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

pub fn list(conn: &Connection) -> AppResult<Vec<PersonRow>> {
    let first = |kind: &str| {
        format!("(SELECT value FROM person_contacts c WHERE c.person_id = p.id AND c.kind = '{kind}' ORDER BY position LIMIT 1)")
    };
    let sql = format!(
        "SELECT id, first_name, last_name, nickname, dob_year, dob_month, dob_day, {}, {}
         FROM people p ORDER BY first_name COLLATE NOCASE, last_name COLLATE NOCASE",
        first("phone"),
        first("email")
    );
    let rows = conn
        .prepare(&sql)?
        .query_map([], |r| {
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
        })?
        .collect::<Result<_, _>>()?;
    Ok(rows)
}
