//! Snapshot files (spec §8): every synced table written as JSONL under `.tend/sync/`. Rows whose id
//! is a ULID go into year shards of 10,000 rows in id order (`tasks/2026-001.jsonl`); rows with
//! natural ids (settings keys, default enum rows, `root`) go into `fixed.jsonl`. Every run
//! rebuilds all of them from SQLite and writes only the files whose bytes changed, so Git sees
//! exactly what changed.

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::types::ValueRef;
use rusqlite::Connection;
use serde_json::{json, Map, Number, Value};
use ulid::Ulid;

use crate::db::{self, columns, tables};
use crate::error::{errors, AppResult};
use crate::settings;
use crate::vault::SYNC_DIR;

const SHARD_ROWS: usize = 10_000;
const FIXED_FILE: &str = "fixed.jsonl";
const MANIFEST_FILE: &str = "manifest.json";
const SHARD_EXT: &str = "jsonl";
const FORMAT_VERSION: u32 = 1;
const MS_PER_DAY: i64 = 86_400_000;

/// A row as written: column → value. Keys sort, so the same row always gives the same bytes.
pub type Row = Map<String, Value>;

/// Every synced table's rows by id.
pub type Snapshot = BTreeMap<&'static str, BTreeMap<String, Row>>;

/// A synced table: the columns that belong to this device only, and which rows are shared.
struct Table {
    name: &'static str,
    local_columns: &'static [&'static str],
    /// Only rows whose column holds one of these values sync (settings: only the shared keys).
    only: Option<(&'static str, &'static [&'static str])>,
}

const fn table(name: &'static str) -> Table {
    Table {
        name,
        local_columns: &[],
        only: None,
    }
}

/// Every synced table, parents before children (the order rows are applied in).
const TABLES: &[Table] = &[
    Table {
        only: Some(("key", settings::SHARED_KEYS)),
        ..table(tables::SETTINGS)
    },
    table(tables::ENUMS),
    Table {
        local_columns: &["mtime", "missing_at"],
        ..table(tables::ATTACHMENTS)
    },
    Table {
        local_columns: &["missing_at"],
        ..table(tables::NOTE_FOLDERS)
    },
    table(tables::NOTES),
    table(tables::PEOPLE),
    table(tables::PERSON_CONTACTS),
    table(tables::TASK_FOLDERS),
    table(tables::TASKS),
    table(tables::MY_DAY),
    table(tables::TASK_COMMENTS),
    table(tables::WORDS),
    table(tables::SNAP_STACKS),
    table(tables::SNAPS),
    table(tables::REFS),
    // Last: links point at rows of every table above.
    table(tables::LINKS),
];

/// The names of the synced tables, in apply order.
pub fn table_names() -> Vec<&'static str> {
    TABLES.iter().map(|t| t.name).collect()
}

/// Every synced row in the database.
pub fn snapshot(conn: &Connection) -> AppResult<Snapshot> {
    snapshot_of(conn, &table_names())
}

/// The synced rows of the named tables only (an older schema lacks the newer ones).
pub fn snapshot_of(conn: &Connection, names: &[&str]) -> AppResult<Snapshot> {
    let mut out = Snapshot::new();
    for t in TABLES.iter().filter(|t| names.contains(&t.name)) {
        out.insert(t.name, read_table(conn, t)?);
    }
    Ok(out)
}

fn read_table(conn: &Connection, t: &Table) -> AppResult<BTreeMap<String, Row>> {
    let mut stmt = conn.prepare(&format!("SELECT * FROM {}", t.name))?;
    let columns: Vec<String> = stmt.column_names().iter().map(|c| c.to_string()).collect();
    let mut rows = stmt.query([])?;
    let mut out = BTreeMap::new();
    while let Some(r) = rows.next()? {
        let mut row = Row::new();
        for (i, column) in columns.iter().enumerate() {
            if t.local_columns.contains(&column.as_str()) {
                continue;
            }
            row.insert(column.clone(), to_json(t.name, r.get_ref(i)?)?);
        }
        if !is_shared(t, &row) {
            continue;
        }
        let id = row_id(t.name, &row)?;
        out.insert(id, row);
    }
    Ok(out)
}

fn is_shared(t: &Table, row: &Row) -> bool {
    let Some((column, values)) = t.only else {
        return true;
    };
    let value = row.get(column).and_then(Value::as_str);
    value.is_some_and(|v| values.contains(&v))
}

fn row_id(table: &str, row: &Row) -> AppResult<String> {
    match row.get(columns::ID).and_then(Value::as_str) {
        Some(id) => Ok(id.to_string()),
        None => Err(errors::DB.with(format!("A {table} row has no text id"))),
    }
}

fn to_json(table: &str, value: ValueRef) -> AppResult<Value> {
    match value {
        ValueRef::Null => Ok(Value::Null),
        ValueRef::Integer(n) => Ok(Value::from(n)),
        ValueRef::Real(x) => Ok(Number::from_f64(x).map_or(Value::Null, Value::Number)),
        ValueRef::Text(t) => Ok(Value::String(String::from_utf8_lossy(t).into_owned())),
        ValueRef::Blob(_) => {
            Err(errors::DB.with(format!("{table} holds a blob, which sync can't write")))
        }
    }
}

/// The UTC year of a moment in ms since 1970 (civil-from-days, H. Hinnant). The standard library
/// has no calendar, and this is the only date sync needs.
fn utc_year(ms: i64) -> i64 {
    let days = ms.div_euclid(MS_PER_DAY);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    // The algorithm's years start in March, so January and February belong to the next one.
    let march_month = (5 * day_of_year + 2) / 153;
    let year = year_of_era + era * 400;
    if march_month >= 10 {
        year + 1
    } else {
        year
    }
}

/// The year a ULID was made in; `None` for a natural id.
fn ulid_year(id: &str) -> Option<i64> {
    let ulid = Ulid::from_string(id).ok()?;
    let ms = i64::try_from(ulid.timestamp_ms()).ok()?;
    Some(utc_year(ms))
}

/// The files a snapshot becomes: path under `.tend/sync/` → contents.
pub fn render(snapshot: &Snapshot) -> AppResult<BTreeMap<PathBuf, String>> {
    let mut files = BTreeMap::new();
    for (table, rows) in snapshot {
        shard_table(table, rows, &mut files)?;
    }
    files.insert(PathBuf::from(MANIFEST_FILE), manifest()?);
    Ok(files)
}

fn shard_table(
    table: &str,
    rows: &BTreeMap<String, Row>,
    files: &mut BTreeMap<PathBuf, String>,
) -> AppResult<()> {
    let mut fixed = Vec::new();
    let mut years: BTreeMap<i64, Vec<&Row>> = BTreeMap::new();
    for (id, row) in rows {
        match ulid_year(id) {
            Some(year) => years.entry(year).or_default().push(row),
            None => fixed.push(row),
        }
    }
    let dir = PathBuf::from(table);
    if !fixed.is_empty() {
        files.insert(dir.join(FIXED_FILE), lines(&fixed)?);
    }
    for (year, rows) in years {
        for (i, chunk) in rows.chunks(SHARD_ROWS).enumerate() {
            let name = format!("{year}-{:03}.{SHARD_EXT}", i + 1);
            files.insert(dir.join(name), lines(chunk)?);
        }
    }
    Ok(())
}

/// One JSON object per line, each line ending in a newline.
fn lines(rows: &[&Row]) -> AppResult<String> {
    let mut out = String::new();
    for row in rows {
        out.push_str(&serde_json::to_string(row)?);
        out.push('\n');
    }
    Ok(out)
}

fn manifest() -> AppResult<String> {
    let manifest = json!({
        "app_id": db::APP_ID,
        "format_version": FORMAT_VERSION,
        "schema_version": db::schema_version(),
        "tables": table_names(),
    });
    let mut text = serde_json::to_string_pretty(&manifest)?;
    text.push('\n');
    Ok(text)
}

/// Refuses a set of shard files that isn't a Tend vault, or comes from a newer schema. Returns
/// the schema version the files were written at.
pub fn check_manifest(files: &BTreeMap<PathBuf, String>) -> AppResult<i64> {
    let manifest: Option<Value> = files
        .get(Path::new(MANIFEST_FILE))
        .and_then(|text| serde_json::from_str(text).ok());
    let Some(manifest) = manifest else {
        return Err(errors::SYNC_NOT_A_VAULT.err());
    };
    if manifest["app_id"] != db::APP_ID {
        return Err(errors::SYNC_NOT_A_VAULT.err());
    }
    let version = manifest["schema_version"].as_i64().unwrap_or(i64::MAX);
    if version < 1 {
        return Err(errors::SYNC_NOT_A_VAULT.err());
    }
    if version > db::schema_version() {
        return Err(errors::SNAPSHOT_TOO_NEW.err());
    }
    Ok(version)
}

/// The rows in a set of shard files (as `render` makes them, e.g. read from a Git tree). Files of a
/// table this app doesn't know are refused: they come from a newer schema.
pub fn parse(files: &BTreeMap<PathBuf, String>) -> AppResult<Snapshot> {
    let mut out: Snapshot = TABLES.iter().map(|t| (t.name, BTreeMap::new())).collect();
    for (rel, text) in files {
        if rel.as_os_str() == MANIFEST_FILE {
            continue;
        }
        let rows = table_of(rel).and_then(|name| out.get_mut(name));
        let Some(rows) = rows else {
            return Err(
                errors::SNAPSHOT_TOO_NEW.with(format!("Unknown snapshot file {}", rel.display()))
            );
        };
        for line in text.lines().filter(|l| !l.is_empty()) {
            let row: Row = serde_json::from_str(line)?;
            let id = row_id(&rel.display().to_string(), &row)?;
            rows.insert(id, row);
        }
    }
    Ok(out)
}

/// The synced table a shard path belongs to (`tasks/2026-001.jsonl` → `tasks`).
fn table_of(rel: &Path) -> Option<&'static str> {
    let dir = rel.parent()?.to_str()?;
    TABLES.iter().map(|t| t.name).find(|name| *name == dir)
}

/// Writes the database's synced rows as shards under `<root>/.tend/sync/`: files whose bytes
/// changed are written, shards no longer produced are removed. Returns how many files changed.
pub fn export(conn: &Connection, root: &Path) -> AppResult<usize> {
    let files = render(&snapshot(conn)?)?;
    let dir = root.join(SYNC_DIR);
    let written = write_changed(&dir, &files)?;
    let removed = remove_stale(&dir, &files)?;
    Ok(written + removed)
}

fn write_changed(dir: &Path, files: &BTreeMap<PathBuf, String>) -> AppResult<usize> {
    let mut written = 0;
    for (rel, text) in files {
        let path = dir.join(rel);
        let same = fs::read_to_string(&path).is_ok_and(|old| old == *text);
        if same {
            continue;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, text)?;
        written += 1;
    }
    Ok(written)
}

/// Removes shard files the snapshot no longer has, and table folders left empty.
fn remove_stale(dir: &Path, files: &BTreeMap<PathBuf, String>) -> AppResult<usize> {
    let mut removed = 0;
    let Ok(tables) = fs::read_dir(dir) else {
        return Ok(0);
    };
    for table in tables.flatten() {
        let table_dir = table.path();
        if !table_dir.is_dir() {
            continue;
        }
        for shard in fs::read_dir(&table_dir)?.flatten() {
            let rel = shard.path().strip_prefix(dir).map(Path::to_path_buf);
            let known = rel.is_ok_and(|r| files.contains_key(&r));
            if !known {
                fs::remove_file(shard.path())?;
                removed += 1;
            }
        }
        let empty = fs::read_dir(&table_dir)?.next().is_none();
        if empty {
            fs::remove_dir(&table_dir)?;
        }
    }
    Ok(removed)
}
