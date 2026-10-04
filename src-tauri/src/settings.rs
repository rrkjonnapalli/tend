//! Key/value app settings stored as JSON in the vault's `settings` table.

use std::collections::HashMap;

use rusqlite::{params, Connection};
use serde_json::Value;

use crate::error::AppResult;
use crate::util::now_ms;

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

pub fn get(conn: &Connection, key: &str) -> AppResult<Option<Value>> {
    let text: Option<String> = conn
        .query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| r.get(0))
        .ok();
    Ok(match text {
        Some(t) => Some(serde_json::from_str(&t)?),
        None => None,
    })
}

pub fn set(conn: &Connection, key: &str, value: &Value) -> AppResult<()> {
    conn.execute(
        "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        params![key, value.to_string(), now_ms()],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_memory;
    use serde_json::json;

    #[test]
    fn set_get_and_overwrite() {
        let conn = open_memory();
        assert_eq!(get(&conn, "ui.nav_open").unwrap(), None);
        set(&conn, "ui.nav_open", &json!(false)).unwrap();
        set(&conn, "ui.nav_open", &json!(true)).unwrap();
        assert_eq!(get(&conn, "ui.nav_open").unwrap(), Some(json!(true)));
        assert_eq!(all(&conn).unwrap().len(), 1);
    }
}
