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

pub fn dob(d: &Dob) -> AppResult<()> {
    let year_ok = d.year.is_none_or(|y| (1..=9999).contains(&y));
    let month_ok = (1..=12).contains(&d.month);
    let day_ok = month_ok && (1..=crate::dates::days_in_month(d.year, d.month)).contains(&d.day);
    if !(year_ok && day_ok) {
        return Err(errors::DOB_INVALID.err());
    }
    Ok(())
}

/// Time of birth as `HH:MM`, 24-hour.
pub fn tob(t: &str) -> AppResult<()> {
    if !crate::dates::is_time(t) {
        return Err(errors::TOB_INVALID.err());
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
