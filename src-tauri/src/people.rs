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

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    pub id: String,
    pub kind: String,
    pub value: String,
    pub label: String,
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
#[derive(Debug, Default, serde::Deserialize)]
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

#[derive(Debug, Clone, Copy, serde::Deserialize)]
pub struct Dob {
    pub year: Option<i64>,
    pub month: i64,
    pub day: i64,
}


const KINDS: [&str; 3] = ["phone", "email", "address"];

fn days_in_month(year: Option<i64>, month: i64) -> i64 {
    match month {
        2 => match year {
            Some(y) if !((y % 4 == 0 && y % 100 != 0) || y % 400 == 0) => 28,
            _ => 29,
        },
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn check_dob(d: &Dob) -> AppResult<()> {
    let valid_year = d.year.is_none_or(|y| (1..=9999).contains(&y));
    if !valid_year || !(1..=12).contains(&d.month) || d.day < 1 || d.day > days_in_month(d.year, d.month) {
        return Err(crate::error::AppError::rule("bad_dob", "Use yyyy-mm-dd or mm-dd"));
    }
    Ok(())
}

fn check_tob(t: &str) -> AppResult<()> {
    let ok = t.len() == 5
        && t.as_bytes()[2] == b':'
        && t[..2].parse::<u8>().is_ok_and(|h| h < 24)
        && t[3..].parse::<u8>().is_ok_and(|m| m < 60);
    if ok {
        Ok(())
    } else {
        Err(crate::error::AppError::rule("bad_tob", "Use a time like 6:30 AM"))
    }
}

fn check_first(name: &str) -> AppResult<String> {
    let n = name.trim();
    if n.is_empty() {
        return Err(crate::error::AppError::rule("empty_name", "A first name is needed"));
    }
    Ok(n.to_string())
}

pub fn create(conn: &Connection, first: &str, last: &str) -> AppResult<String> {
    let id = crate::util::new_id();
    let now = crate::util::now_ms();
    conn.execute(
        "INSERT INTO people (id, first_name, last_name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
        rusqlite::params![id, check_first(first)?, last.trim(), now],
    )?;
    Ok(id)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Person> {
    let row = list(conn)?
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| crate::error::AppError::NotFound("Person".into()))?;
    let (tob, notes) = conn.query_row("SELECT tob, notes FROM people WHERE id = ?1", [id], |r| Ok((r.get(0)?, r.get(1)?)))?;
    let contacts = conn
        .prepare("SELECT id, kind, value, label FROM person_contacts WHERE person_id = ?1 ORDER BY kind, position")?
        .query_map([id], |r| Ok(Contact { id: r.get(0)?, kind: r.get(1)?, value: r.get(2)?, label: r.get(3)? }))?
        .collect::<Result<_, _>>()?;
    Ok(Person { row, tob, notes, contacts })
}

pub fn update(conn: &mut Connection, id: &str, p: &Patch) -> AppResult<Person> {
    if let Some(Some(d)) = &p.dob {
        check_dob(d)?;
    }
    if let Some(Some(t)) = &p.tob {
        check_tob(t)?;
    }
    let tx = conn.transaction()?;
    if let Some(first) = &p.first_name {
        tx.execute("UPDATE people SET first_name = ?1 WHERE id = ?2", rusqlite::params![check_first(first)?, id])?;
    }
    if let Some(last) = &p.last_name {
        tx.execute("UPDATE people SET last_name = ?1 WHERE id = ?2", rusqlite::params![last.trim(), id])?;
    }
    if let Some(nick) = &p.nickname {
        let nick = nick.as_deref().map(str::trim).filter(|n| !n.is_empty());
        tx.execute("UPDATE people SET nickname = ?1 WHERE id = ?2", rusqlite::params![nick, id])?;
    }
    if let Some(dob) = &p.dob {
        let (y, m, d) = dob.map_or((None, None, None), |d| (d.year, Some(d.month), Some(d.day)));
        tx.execute(
            "UPDATE people SET dob_year = ?1, dob_month = ?2, dob_day = ?3 WHERE id = ?4",
            rusqlite::params![y, m, d, id],
        )?;
    }
    if let Some(tob) = &p.tob {
        tx.execute("UPDATE people SET tob = ?1 WHERE id = ?2", rusqlite::params![tob, id])?;
    }
    if let Some(notes) = &p.notes {
        tx.execute("UPDATE people SET notes = ?1 WHERE id = ?2", rusqlite::params![notes, id])?;
    }
    tx.execute("UPDATE people SET updated_at = ?1 WHERE id = ?2", rusqlite::params![crate::util::now_ms(), id])?;
    tx.commit()?;
    get(conn, id)
}

/// Deleting a person unassigns their tasks (ON DELETE SET NULL) and drops their contacts.
pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM people WHERE id = ?1", [id])?;
    Ok(())
}

fn check_contact(kind: &str, value: &str) -> AppResult<String> {
    if !KINDS.contains(&kind) {
        return Err(crate::error::AppError::rule("bad_kind", format!("Unknown contact kind {kind}")));
    }
    let v = value.trim();
    if v.is_empty() {
        return Err(crate::error::AppError::rule("empty_value", "Enter a value"));
    }
    Ok(v.to_string())
}

pub fn contact_add(conn: &Connection, person: &str, kind: &str, value: &str, label: &str) -> AppResult<()> {
    let value = check_contact(kind, value)?;
    let next: i64 = conn.query_row(
        "SELECT coalesce(max(position) + 1, 0) FROM person_contacts WHERE person_id = ?1 AND kind = ?2",
        rusqlite::params![person, kind],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO person_contacts (id, person_id, kind, value, label, position, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![crate::util::new_id(), person, kind, value, label.trim(), next, crate::util::now_ms()],
    )?;
    Ok(())
}

pub fn contact_update(conn: &Connection, id: &str, value: &str, label: &str) -> AppResult<()> {
    let kind: String = conn.query_row("SELECT kind FROM person_contacts WHERE id = ?1", [id], |r| r.get(0))?;
    let value = check_contact(&kind, value)?;
    conn.execute(
        "UPDATE person_contacts SET value = ?1, label = ?2 WHERE id = ?3",
        rusqlite::params![value, label.trim(), id],
    )?;
    Ok(())
}

pub fn contact_delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM person_contacts WHERE id = ?1", [id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_memory;

    fn patch(json: &str) -> Patch {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn create_update_and_list() {
        let mut conn = open_memory();
        let id = create(&conn, "Ravi", "Kumar").unwrap();
        assert!(create(&conn, "  ", "x").is_err());
        let p = update(&mut conn, &id, &patch(r#"{"nickname":"Ravs","dob":{"year":1995,"month":10,"day":9},"tob":"06:30"}"#)).unwrap();
        assert_eq!(p.row.nickname.as_deref(), Some("Ravs"));
        assert_eq!((p.row.dob_year, p.row.dob_month, p.row.dob_day), (Some(1995), Some(10), Some(9)));
        assert_eq!(p.tob.as_deref(), Some("06:30"));
        let p = update(&mut conn, &id, &patch(r#"{"dob":{"year":null,"month":2,"day":29},"nickname":"","tob":null}"#)).unwrap();
        assert_eq!((p.row.dob_year, p.row.dob_month, p.row.dob_day), (None, Some(2), Some(29)));
        assert_eq!((p.row.nickname, p.tob), (None, None));
    }

    #[test]
    fn invalid_dob_and_time_are_rejected_and_not_saved() {
        let mut conn = open_memory();
        let id = create(&conn, "A", "").unwrap();
        for bad in [
            r#"{"dob":{"year":2023,"month":2,"day":29}}"#,
            r#"{"dob":{"year":null,"month":13,"day":1}}"#,
            r#"{"dob":{"year":null,"month":4,"day":31}}"#,
            r#"{"tob":"25:00"}"#,
            r#"{"tob":"6:30"}"#,
        ] {
            assert!(update(&mut conn, &id, &patch(bad)).is_err(), "{bad} should fail");
        }
        let p = get(&conn, &id).unwrap();
        assert_eq!((p.row.dob_month, p.tob), (None, None));
    }

    #[test]
    fn contacts_keep_order_and_feed_the_list() {
        let mut conn = open_memory();
        let id = create(&conn, "A", "B").unwrap();
        contact_add(&conn, &id, "phone", "+91 1", "personal").unwrap();
        contact_add(&conn, &id, "phone", "+91 2", "work").unwrap();
        contact_add(&conn, &id, "email", "a@b.c", "").unwrap();
        assert!(contact_add(&conn, &id, "fax", "1", "").is_err());
        assert!(contact_add(&conn, &id, "email", " ", "").is_err());
        let row = &list(&conn).unwrap()[0];
        assert_eq!((row.phone.as_deref(), row.email.as_deref()), (Some("+91 1"), Some("a@b.c")));
        let p = get(&conn, &id).unwrap();
        let first = p.contacts.iter().find(|c| c.value == "+91 1").unwrap().id.clone();
        contact_update(&conn, &first, "+91 9", "home").unwrap();
        contact_delete(&conn, &p.contacts.iter().find(|c| c.value == "+91 2").unwrap().id).unwrap();
        let p = get(&conn, &id).unwrap();
        let phones: Vec<_> = p.contacts.iter().filter(|c| c.kind == "phone").map(|c| (c.value.as_str(), c.label.as_str())).collect();
        assert_eq!(phones, vec![("+91 9", "home")]);
        let t = crate::tasks::create(&mut conn, "", "T", None, None, None).unwrap();
        crate::tasks::update(&conn, &t.id, &serde_json::from_str(&format!(r#"{{"assigneeId":"{id}"}}"#)).unwrap()).unwrap();
        delete(&conn, &id).unwrap();
        assert!(crate::tasks::get(&conn, &t.id).unwrap().assignee_id.is_none());
    }
}
