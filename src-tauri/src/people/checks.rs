//! The rules for person fields. Each check returns the cleaned value or a rule error.

use super::{Dob, Patch};
use crate::error::{errors, AppResult};

const KINDS: [&str; 3] = ["phone", "email", "address"];

/// Checks every field a patch sets, before anything is written.
pub fn patch(p: &Patch) -> AppResult<()> {
    if let Some(first) = &p.first_name {
        first_name(first)?;
    }
    if let Some(Some(d)) = &p.dob {
        dob(d)?;
    }
    if let Some(Some(t)) = &p.tob {
        tob(t)?;
    }
    Ok(())
}

pub fn first_name(name: &str) -> AppResult<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(errors::FIRST_NAME_EMPTY.err());
    }
    Ok(trimmed.to_string())
}

fn is_leap(year: i64) -> bool {
    let every_fourth = year % 4 == 0 && year % 100 != 0;
    every_fourth || year % 400 == 0
}

/// Without a year, February allows the 29th.
fn days_in_month(year: Option<i64>, month: i64) -> i64 {
    match month {
        2 => match year {
            Some(y) if !is_leap(y) => 28,
            _ => 29,
        },
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

pub fn dob(d: &Dob) -> AppResult<()> {
    let year_ok = d.year.is_none_or(|y| (1..=9999).contains(&y));
    let month_ok = (1..=12).contains(&d.month);
    let day_ok = month_ok && (1..=days_in_month(d.year, d.month)).contains(&d.day);
    if !(year_ok && day_ok) {
        return Err(errors::DOB_INVALID.err());
    }
    Ok(())
}

/// Time of birth as `HH:MM`, 24-hour.
pub fn tob(t: &str) -> AppResult<()> {
    let bad = || errors::TOB_INVALID.err();
    let shaped = t.len() == 5 && t.as_bytes()[2] == b':';
    if !shaped {
        return Err(bad());
    }
    let hour_ok = t[..2].parse::<u8>().is_ok_and(|h| h < 24);
    let minute_ok = t[3..].parse::<u8>().is_ok_and(|m| m < 60);
    if !(hour_ok && minute_ok) {
        return Err(bad());
    }
    Ok(())
}

pub fn contact(kind: &str, value: &str) -> AppResult<String> {
    if !KINDS.contains(&kind) {
        return Err(errors::CONTACT_KIND.with(format!("Unknown contact kind {kind}")));
    }
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(errors::CONTACT_EMPTY.err());
    }
    Ok(trimmed.to_string())
}
