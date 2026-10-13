//! The rules for status values, labels, colours and order.

use rusqlite::Connection;

use super::{Status, DONE, FIRST, KIND, RESERVED};
use crate::enums::rows;
use crate::error::{errors, AppResult};

/// A new value: UPPER_SNAKE, not a default status's value, not used yet. Returns it trimmed.
pub fn new_value<'a>(conn: &Connection, value: &'a str) -> AppResult<&'a str> {
    let value = value.trim();
    let starts_with_letter = value.chars().next().is_some_and(|c| c.is_ascii_uppercase());
    let upper_snake = value
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
    if !(starts_with_letter && upper_snake) {
        return Err(errors::STATUS_VALUE.err());
    }
    if RESERVED.iter().any(|r| r.eq_ignore_ascii_case(value)) {
        return Err(
            errors::STATUS_RESERVED.with(format!("{value} is reserved for a default status."))
        );
    }
    if rows::get(conn, KIND, value)?.is_some() {
        return Err(
            errors::STATUS_VALUE_TAKEN.with(format!("{value} is already used by another status."))
        );
    }
    Ok(value)
}

/// A non-empty label no other status uses (`except` is the status being renamed).
pub fn label(conn: &Connection, label: &str, except: Option<&str>) -> AppResult<String> {
    let label = label.trim();
    if label.is_empty() {
        return Err(errors::STATUS_LABEL_EMPTY.err());
    }
    let wanted = rows::LabelUse {
        kind: KIND,
        label,
        except,
    };
    if rows::label_taken(conn, &wanted)? {
        return Err(errors::STATUS_LABEL_TAKEN.err());
    }
    Ok(label.to_string())
}

/// The same statuses as now, with Not started first and Done last.
pub fn order(current: &[Status], values: &[String]) -> AppResult<()> {
    let mut proposed: Vec<&str> = values.iter().map(String::as_str).collect();
    proposed.sort_unstable();
    let mut existing: Vec<&str> = current.iter().map(|s| s.value.as_str()).collect();
    existing.sort_unstable();
    let same_set = proposed == existing;
    let first_ok = values.first().map(String::as_str) == Some(FIRST);
    let last_ok = values.last().map(String::as_str) == Some(DONE);
    if !(same_set && first_ok && last_ok) {
        return Err(errors::STATUS_ORDER.err());
    }
    Ok(())
}
