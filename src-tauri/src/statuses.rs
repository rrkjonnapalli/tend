//! Task statuses (app-wide). NOT_STARTED is always first and DONE always last.

use rusqlite::Connection;
use serde::Serialize;

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub value: String,
    pub label: String,
    pub color: String,
    pub position: i64,
    pub is_default: bool,
}

pub fn list(conn: &Connection) -> AppResult<Vec<Status>> {
    let rows = conn
        .prepare("SELECT value, label, color, position, is_default FROM statuses ORDER BY position")?
        .query_map([], |r| {
            Ok(Status {
                value: r.get(0)?,
                label: r.get(1)?,
                color: r.get(2)?,
                position: r.get(3)?,
                is_default: r.get::<_, i64>(4)? == 1,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(rows)
}

use rusqlite::{params, OptionalExtension};

use crate::error::AppError;

const RESERVED: [&str; 3] = ["NOT_STARTED", "IN_PROGRESS", "DONE"];
const FIRST: &str = "NOT_STARTED";
const LAST: &str = "DONE";

fn check_label(conn: &Connection, label: &str, except: Option<&str>) -> AppResult<String> {
    let label = label.trim();
    if label.is_empty() {
        return Err(AppError::rule("empty_label", "Label can’t be empty."));
    }
    let taken = conn
        .query_row(
            "SELECT 1 FROM statuses WHERE label = ?1 COLLATE NOCASE AND value IS NOT ?2",
            params![label, except],
            |_| Ok(()),
        )
        .optional()?
        .is_some();
    if taken {
        return Err(AppError::rule("label_taken", "Another status already uses this label."));
    }
    Ok(label.to_string())
}

fn check_color(color: &str) -> AppResult<()> {
    let ok = color.len() == 7 && color.starts_with('#') && color[1..].chars().all(|c| c.is_ascii_hexdigit());
    if ok {
        Ok(())
    } else {
        Err(AppError::rule("bad_color", "Colors are #rrggbb."))
    }
}

/// Adds a custom status just before Done.
pub fn add(conn: &Connection, label: &str, value: &str, color: &str) -> AppResult<Status> {
    let value = value.trim();
    let valid = value.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        && value.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
    if !valid {
        return Err(AppError::rule("bad_value", "Give the status a value (A–Z, 0–9 and _)."));
    }
    if RESERVED.iter().any(|r| r.eq_ignore_ascii_case(value)) {
        return Err(AppError::rule("reserved", format!("{value} is reserved for a default status.")));
    }
    let exists = conn
        .query_row("SELECT 1 FROM statuses WHERE value = ?1", [value], |_| Ok(()))
        .optional()?
        .is_some();
    if exists {
        return Err(AppError::rule("value_taken", format!("{value} is already used by another status.")));
    }
    let label = check_label(conn, label, None)?;
    check_color(color)?;
    let position: i64 = conn.query_row("SELECT max(position) + 1 FROM statuses WHERE value <> ?1", [LAST], |r| r.get(0))?;
    conn.execute(
        "INSERT INTO statuses (value, label, color, position, is_default) VALUES (?1, ?2, ?3, ?4, 0)",
        params![value, label, color, position],
    )?;
    get(conn, value)
}

fn get(conn: &Connection, value: &str) -> AppResult<Status> {
    list(conn)?
        .into_iter()
        .find(|s| s.value == value)
        .ok_or_else(|| AppError::NotFound("Status".into()))
}

pub fn rename(conn: &Connection, value: &str, label: &str) -> AppResult<Status> {
    let label = check_label(conn, label, Some(value))?;
    conn.execute("UPDATE statuses SET label = ?1 WHERE value = ?2", params![label, value])?;
    get(conn, value)
}

pub fn recolor(conn: &Connection, value: &str, color: &str) -> AppResult<Status> {
    check_color(color)?;
    conn.execute("UPDATE statuses SET color = ?1 WHERE value = ?2", params![color, value])?;
    get(conn, value)
}

/// Sets the order. Not started stays first and Done stays last.
pub fn reorder(conn: &mut Connection, values: &[String]) -> AppResult<()> {
    let current = list(conn)?;
    let mut same: Vec<_> = values.iter().map(String::as_str).collect();
    same.sort_unstable();
    let mut have: Vec<_> = current.iter().map(|s| s.value.as_str()).collect();
    have.sort_unstable();
    if same != have || values.first().map(String::as_str) != Some(FIRST) || values.last().map(String::as_str) != Some(LAST) {
        return Err(AppError::rule("bad_order", "Not started stays first and Done stays last."));
    }
    let tx = conn.transaction()?;
    for (i, v) in values.iter().enumerate().take(values.len() - 1) {
        tx.execute("UPDATE statuses SET position = ?1 WHERE value = ?2", params![i as i64, v])?;
    }
    tx.commit()?;
    Ok(())
}

/// Deletes a custom status that no task uses.
pub fn delete(conn: &Connection, value: &str) -> AppResult<()> {
    let s = get(conn, value)?;
    if s.is_default {
        return Err(AppError::rule("default", "Default statuses can be renamed, not deleted."));
    }
    let used: i64 = conn.query_row("SELECT count(*) FROM tasks WHERE status = ?1", [value], |r| r.get(0))?;
    if used > 0 {
        return Err(AppError::rule(
            "in_use",
            format!("Can’t delete “{}”: some tasks still have this status. Move them to another status first.", s.label),
        ));
    }
    conn.execute("DELETE FROM statuses WHERE value = ?1", [value])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_memory;

    fn values(conn: &Connection) -> Vec<String> {
        list(conn).unwrap().into_iter().map(|s| s.value).collect()
    }

    fn code(r: AppResult<impl std::fmt::Debug>) -> &'static str {
        match r {
            Err(e) => e.code(),
            Ok(v) => panic!("expected an error, got {v:?}"),
        }
    }

    #[test]
    fn defaults_are_first_middle_last() {
        let conn = open_memory();
        assert_eq!(values(&conn), vec!["NOT_STARTED", "IN_PROGRESS", "DONE"]);
    }

    #[test]
    fn custom_statuses_go_before_done() {
        let conn = open_memory();
        add(&conn, "Waiting", "WAITING", "#d57149").unwrap();
        add(&conn, "In review", "IN_REVIEW", "#6f5ee4").unwrap();
        assert_eq!(values(&conn), vec!["NOT_STARTED", "IN_PROGRESS", "WAITING", "IN_REVIEW", "DONE"]);
    }

    #[test]
    fn add_rules() {
        let conn = open_memory();
        assert_eq!(code(add(&conn, "Ready", "done", "#000000")), "bad_value");
        assert_eq!(code(add(&conn, "Ready", "DONE", "#000000")), "reserved");
        assert_eq!(code(add(&conn, "Ready", "In_Progress", "#000000")), "bad_value");
        assert_eq!(code(add(&conn, "not started", "READY", "#000000")), "label_taken");
        assert_eq!(code(add(&conn, "Ready", "READY", "blue")), "bad_color");
        add(&conn, "Ready", "READY", "#000000").unwrap();
        assert_eq!(code(add(&conn, "Other", "READY", "#000000")), "value_taken");
        assert_eq!(code(add(&conn, "  ", "X", "#000000")), "empty_label");
    }

    #[test]
    fn rename_and_recolor() {
        let conn = open_memory();
        assert_eq!(rename(&conn, "NOT_STARTED", "Ready").unwrap().label, "Ready");
        assert_eq!(code(rename(&conn, "IN_PROGRESS", "ready")), "label_taken");
        assert_eq!(rename(&conn, "IN_PROGRESS", "In Progress").unwrap().label, "In Progress");
        assert_eq!(recolor(&conn, "DONE", "#62d2a2").unwrap().color, "#62d2a2");
    }

    #[test]
    fn reorder_keeps_first_and_last() {
        let mut conn = open_memory();
        add(&conn, "Waiting", "WAITING", "#d57149").unwrap();
        let order = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        reorder(&mut conn, &order(&["NOT_STARTED", "WAITING", "IN_PROGRESS", "DONE"])).unwrap();
        assert_eq!(values(&conn), vec!["NOT_STARTED", "WAITING", "IN_PROGRESS", "DONE"]);
        assert_eq!(code(reorder(&mut conn, &order(&["WAITING", "NOT_STARTED", "IN_PROGRESS", "DONE"]))), "bad_order");
        assert_eq!(code(reorder(&mut conn, &order(&["NOT_STARTED", "IN_PROGRESS", "DONE"]))), "bad_order");
    }

    #[test]
    fn delete_rules() {
        let mut conn = open_memory();
        add(&conn, "Waiting", "WAITING", "#d57149").unwrap();
        assert_eq!(code(delete(&conn, "DONE")), "default");
        let t = crate::tasks::create(&mut conn, "", "T", None, Some("WAITING"), None).unwrap();
        assert_eq!(code(delete(&conn, "WAITING")), "in_use");
        crate::tasks::delete(&conn, &t.id).unwrap();
        delete(&conn, "WAITING").unwrap();
        assert_eq!(values(&conn), vec!["NOT_STARTED", "IN_PROGRESS", "DONE"]);
    }
}
