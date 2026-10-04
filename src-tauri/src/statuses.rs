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
