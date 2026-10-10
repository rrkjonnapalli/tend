//! Key/value app settings stored as JSON in the vault's `settings` table.

#[cfg(test)]
mod tests;

use std::collections::HashMap;

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;

use crate::error::{errors, AppResult};
use crate::util::now_ms;

/// The settings every device shares through sync; the rest belong to one device (layout, last
/// views, paths). Keys are the ones in `src/lib/config/settings.config.ts`.
pub const SHARED_KEYS: &[&str] = &["tasks.root_name", "timers.sound", crate::files::LIMIT_KEY];

pub fn all(conn: &Connection) -> AppResult<HashMap<String, Value>> {
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    let mut out = HashMap::new();
    for row in rows {
        let (key, text) = row?;
        out.insert(key, serde_json::from_str(&text)?);
    }
    Ok(out)
}

/// The settings with rules: the file size limit is a whole number of MB in its range.
fn check(key: &str, value: &Value) -> AppResult<()> {
    let in_range = value
        .as_u64()
        .is_some_and(|mb| crate::files::LIMIT_MB_RANGE.contains(&mb));
    if key == crate::files::LIMIT_KEY && !in_range {
        return Err(errors::FILE_LIMIT.err());
    }
    Ok(())
}

pub fn get(conn: &Connection, key: &str) -> AppResult<Option<Value>> {
    let text: Option<String> = conn
        .query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| {
            r.get(0)
        })
        .optional()?;
    let Some(text) = text else {
        return Ok(None);
    };
    Ok(Some(serde_json::from_str(&text)?))
}

pub fn set(conn: &Connection, key: &str, value: &Value) -> AppResult<()> {
    check(key, value)?;
    conn.execute(
        "INSERT INTO settings (id, key, value, updated_at) VALUES (?1, ?1, ?2, ?3)
         ON CONFLICT(id) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        params![key, value.to_string(), now_ms()],
    )?;
    Ok(())
}
