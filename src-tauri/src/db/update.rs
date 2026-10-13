//! One `UPDATE` of exactly the columns a patch names, answering with the row as it now is.

use rusqlite::{params_from_iter, Connection, OptionalExtension, ToSql};

use super::columns::UPDATED_AT;
use crate::error::AppResult;
use crate::util::now_ms;

/// The columns one row of `table` gets. Column names are constants or literals at the call site,
/// never user input: they go into the SQL text.
pub struct Update<'a> {
    table: &'static str,
    /// `column = ?n`, one per value.
    sets: Vec<String>,
    values: Vec<Box<dyn ToSql + 'a>>,
}

impl<'a> Update<'a> {
    pub fn new(table: &'static str) -> Self {
        Update {
            table,
            sets: Vec::new(),
            values: Vec::new(),
        }
    }

    pub fn set(&mut self, column: &'static str, value: impl ToSql + 'a) {
        let n = self.values.len() + 1;
        self.sets.push(format!("{column} = ?{n}"));
        self.values.push(Box::new(value));
    }

    /// Sets the column only where it holds nothing yet.
    pub fn set_if_null(&mut self, column: &'static str, value: impl ToSql + 'a) {
        let n = self.values.len() + 1;
        self.sets
            .push(format!("{column} = coalesce({column}, ?{n})"));
        self.values.push(Box::new(value));
    }

    /// `set` when the patch names the column.
    pub fn set_some(&mut self, column: &'static str, value: Option<impl ToSql + 'a>) {
        if let Some(value) = value {
            self.set(column, value);
        }
    }

    /// Writes the columns and `updated_at` to the row `id`, and returns `returning` (a column list)
    /// read by `map`; `None` when there is no such row.
    pub fn run<T>(
        mut self,
        conn: &Connection,
        id: &'a str,
        returning: Returning<T>,
    ) -> AppResult<Option<T>> {
        self.set(UPDATED_AT, now_ms());
        let sql = format!(
            "UPDATE {} SET {} WHERE id = ?{} RETURNING {}",
            self.table,
            self.sets.join(", "),
            self.values.len() + 1,
            returning.columns
        );
        self.values.push(Box::new(id));
        let row = conn
            .query_row(&sql, params_from_iter(self.values), returning.map)
            .optional()?;
        Ok(row)
    }
}

/// What a write answers with: the columns to return and how a row of them is read.
pub struct Returning<T> {
    pub columns: &'static str,
    pub map: fn(&rusqlite::Row) -> rusqlite::Result<T>,
}
