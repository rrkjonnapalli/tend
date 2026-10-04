//! Colors stored in the database (status and label colors): `#rrggbb`.

use crate::error::{errors, AppResult};

pub fn check(color: &str) -> AppResult<()> {
    let shaped = color.len() == 7 && color.starts_with('#');
    if !shaped || !color[1..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(errors::COLOR_INVALID.err());
    }
    Ok(())
}
