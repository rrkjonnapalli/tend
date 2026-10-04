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
    fn set_and_overwrite() {
        let conn = open_memory();
        set(&conn, "ui.nav_open", &json!(false)).unwrap();
        set(&conn, "ui.nav_open", &json!(true)).unwrap();
        let all = all(&conn).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all["ui.nav_open"], json!(true));
    }
}
