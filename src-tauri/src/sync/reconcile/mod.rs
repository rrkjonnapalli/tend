//! Reconciling (spec §9): the merged state of base, local and remote snapshots. Pure: the same
//! three inputs give the same result on every device, so every device ends with the same rows.
//!
//! 1. Three-way merge per row: a change on one side wins; both changed → newer `updated_at`
//!    (equal times: the line that sorts higher); a deletion on either side wins.
//! 2. Unique keys: rows that collide after merging are merged or renamed, never dropped with
//!    their content (§9.2).
//! 3. References: rows pointing at something that no longer exists are fixed or dropped, parents
//!    first, until nothing changes (§9.3).

#[cfg(test)]
mod sim_tests;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use super::shards::{Row, Snapshot};
use crate::names;

/// The merged state of the three snapshots.
pub fn reconcile(base: &Snapshot, local: &Snapshot, remote: &Snapshot) -> Snapshot {
    let mut merged = merge_tables(base, local, remote);
    fix_unique_keys(&mut merged);
    fix_references(&mut merged);
    merged
}

// ---- 1. three-way merge

fn merge_tables(base: &Snapshot, local: &Snapshot, remote: &Snapshot) -> Snapshot {
    let empty = BTreeMap::new();
    let tables: BTreeSet<&'static str> = base
        .keys()
        .chain(local.keys())
        .chain(remote.keys())
        .copied()
        .collect();
    let mut merged = Snapshot::new();
    for table in tables {
        let b = base.get(table).unwrap_or(&empty);
        let l = local.get(table).unwrap_or(&empty);
        let r = remote.get(table).unwrap_or(&empty);
        merged.insert(table, merge_table(b, l, r));
    }
    merged
}

type Rows = BTreeMap<String, Row>;

fn merge_table(base: &Rows, local: &Rows, remote: &Rows) -> Rows {
    let ids: BTreeSet<&String> = base
        .keys()
        .chain(local.keys())
        .chain(remote.keys())
        .collect();
    let mut out = Rows::new();
    for id in ids {
        let row = merge_row(base.get(id), local.get(id), remote.get(id));
        if let Some(row) = row {
            out.insert(id.clone(), row);
        }
    }
    out
}

fn merge_row(base: Option<&Row>, local: Option<&Row>, remote: Option<&Row>) -> Option<Row> {
    match (base, local, remote) {
        // Present at the last sync and gone on one side: deleted, whatever the other side did.
        (Some(_), None, _) | (Some(_), _, None) => None,
        (Some(b), Some(l), Some(r)) => Some(changed_side(b, l, r).clone()),
        (None, Some(l), None) => Some(l.clone()),
        (None, None, Some(r)) => Some(r.clone()),
        (None, Some(l), Some(r)) => Some(newer(l, r).clone()),
        (None, None, None) => None,
    }
}

/// The version that holds a change since the base; when both changed, the newer one.
fn changed_side<'a>(base: &Row, local: &'a Row, remote: &'a Row) -> &'a Row {
    if local == base {
        return remote;
    }
    if remote == base {
        return local;
    }
    newer(local, remote)
}

/// The row with the later `updated_at`; equal times pick the line that sorts higher, so every
/// device picks the same one.
fn newer<'a>(a: &'a Row, b: &'a Row) -> &'a Row {
    let at = |row: &Row| row.get("updated_at").and_then(Value::as_i64).unwrap_or(0);
    if at(a) != at(b) {
        return if at(a) > at(b) { a } else { b };
    }
    let line = |row: &Row| serde_json::to_string(row).unwrap_or_default();
    if line(a) >= line(b) {
        a
    } else {
        b
    }
}

// ---- 2. unique keys

/// Link and path tables: rows with the same key collapse into the one with the smallest id.
const KEEP_SMALLEST: &[(&str, &[&str])] = &[
    ("note_tags", &["note_id", "tag_id"]),
    ("task_tags", &["task_id", "tag_id"]),
    ("task_label_links", &["task_id", "label_id"]),
    ("my_day", &["task_id", "day"]),
    (
        "attachment_links",
        &["owner_type", "owner_id", "attachment_id"],
    ),
    ("note_folders", &["path"]),
    ("task_folders", &["path"]),
];

/// Display names unique ignoring case: a later row gets " 2", " 3", … so both stay.
const UNIQUE_NAMES: &[(&str, &str)] = &[("task_labels", "name"), ("statuses", "label")];

fn fix_unique_keys(merged: &mut Snapshot) {
    for (table, columns) in KEEP_SMALLEST {
        if let Some(rows) = merged.get_mut(table) {
            keep_smallest(rows, columns);
        }
    }
    for (table, column) in UNIQUE_NAMES {
        if let Some(rows) = merged.get_mut(table) {
            number_duplicates(rows, column);
        }
    }
    if let Some(rows) = merged.get_mut("attachments") {
        suffix_clashing_paths(rows);
    }
}

fn text<'a>(row: &'a Row, column: &str) -> Option<&'a str> {
    row.get(column).and_then(Value::as_str)
}

fn key_of(row: &Row, columns: &[&str]) -> Vec<Value> {
    columns
        .iter()
        .map(|c| row.get(*c).cloned().unwrap_or(Value::Null))
        .collect()
}

fn keep_smallest(rows: &mut Rows, columns: &[&str]) {
    let mut seen = BTreeSet::new();
    let mut extra = Vec::new();
    // Rows iterate in id order, so the first of each key has the smallest id.
    for (id, row) in rows.iter() {
        let key = serde_json::to_string(&key_of(row, columns)).unwrap_or_default();
        if !seen.insert(key) {
            extra.push(id.clone());
        }
    }
    for id in extra {
        rows.remove(&id);
    }
}

fn number_duplicates(rows: &mut Rows, column: &str) {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let ids: Vec<String> = rows.keys().cloned().collect();
    for id in ids {
        let Some(row) = rows.get_mut(&id) else {
            continue;
        };
        let name = text(row, column).unwrap_or_default().to_string();
        let mut candidate = name.clone();
        let mut n = 2;
        while seen.contains(&candidate.to_lowercase()) {
            candidate = format!("{name} {n}");
            n += 1;
        }
        seen.insert(candidate.to_lowercase());
        if candidate != name {
            row.insert(column.to_string(), Value::String(candidate));
        }
    }
}

/// Two note bodies at the same path: the one with the smaller id keeps it, the other gets the
/// next free name in its folder (`plan_1.md`). Attached files live at their own id, so they never
/// clash.
fn suffix_clashing_paths(rows: &mut Rows) {
    let mut taken: BTreeSet<String> = BTreeSet::new();
    let ids: Vec<String> = rows.keys().cloned().collect();
    for id in ids {
        let Some(row) = rows.get_mut(&id) else {
            continue;
        };
        let path = text(row, "path").unwrap_or_default().to_string();
        if taken.insert(path.clone()) {
            continue;
        }
        let free = free_path(&path, &taken);
        taken.insert(free.clone());
        set_path(row, &free);
    }
}

/// `folder/name.ext` with the name given the next free suffix among `taken`.
fn free_path(path: &str, taken: &BTreeSet<String>) -> String {
    let folder = names::parent(path);
    let file = names::base(path);
    let (stem, ext) = match file.rsplit_once('.') {
        Some((stem, ext)) => (stem, format!(".{ext}")),
        None => (file, String::new()),
    };
    let is_taken = |n: &str| Ok(taken.contains(&names::join(folder, &format!("{n}{ext}"))));
    let name = names::free(stem, is_taken).unwrap_or_else(|_| stem.to_string());
    names::join(folder, &format!("{name}{ext}"))
}

fn set_path(row: &mut Row, path: &str) {
    row.insert("path".into(), Value::String(path.to_string()));
    row.insert("name".into(), Value::String(names::base(path).to_string()));
}

// ---- 3. references

/// Applies the reference rules until a pass changes nothing (a dropped task drops its subtasks,
/// which drops their comments, and so on).
fn fix_references(merged: &mut Snapshot) {
    loop {
        let changed = fix_pass(merged);
        if !changed {
            break;
        }
    }
}

fn ids(merged: &Snapshot, table: &str) -> BTreeSet<String> {
    merged
        .get(table)
        .map(|rows| rows.keys().cloned().collect())
        .unwrap_or_default()
}

fn values(merged: &Snapshot, table: &str, column: &str) -> BTreeSet<String> {
    merged
        .get(table)
        .map(|rows| {
            rows.values()
                .filter_map(|r| text(r, column))
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

/// One pass over every rule. Returns true when anything changed.
fn fix_pass(merged: &mut Snapshot) -> bool {
    let mut changed = false;
    changed |= drop_orphan_folders(merged, "note_folders");
    changed |= drop_orphan_folders(merged, "task_folders");
    changed |= drop_missing(merged, "notes", "attachment_id", "attachments");
    changed |= move_homeless_notes(merged);
    changed |= drop_missing(merged, "note_tags", "note_id", "notes");
    changed |= drop_missing(merged, "note_tags", "tag_id", "tags");
    changed |= drop_missing(merged, "person_contacts", "person_id", "people");
    changed |= fix_tasks(merged);
    for child in ["my_day", "task_tags", "task_label_links", "task_comments"] {
        changed |= drop_missing(merged, child, "task_id", "tasks");
    }
    changed |= drop_missing(merged, "task_tags", "tag_id", "tags");
    changed |= drop_missing(merged, "task_label_links", "label_id", "task_labels");
    changed |= drop_missing(merged, "attachment_links", "attachment_id", "attachments");
    changed |= drop_ownerless_links(merged);
    changed
}

/// Drops rows of `table` whose `column` names a row `parent` no longer has.
fn drop_missing(merged: &mut Snapshot, table: &str, column: &str, parent: &str) -> bool {
    let alive = ids(merged, parent);
    let Some(rows) = merged.get_mut(table) else {
        return false;
    };
    let before = rows.len();
    rows.retain(|_, row| text(row, column).is_some_and(|id| alive.contains(id)));
    rows.len() != before
}

/// A folder whose parent folder is gone goes too (its tasks move to the root, its notes too).
fn drop_orphan_folders(merged: &mut Snapshot, table: &str) -> bool {
    let paths = values(merged, table, "path");
    let Some(rows) = merged.get_mut(table) else {
        return false;
    };
    let before = rows.len();
    rows.retain(|_, row| {
        let path = text(row, "path").unwrap_or_default();
        let parent = names::parent(path);
        path.is_empty() || parent.is_empty() || paths.contains(parent)
    });
    rows.len() != before
}

/// A note body whose folder is gone moves to the notes root, with a suffix if the name is taken.
fn move_homeless_notes(merged: &mut Snapshot) -> bool {
    let folders = values(merged, "note_folders", "path");
    let bodies: BTreeSet<String> = values(merged, "notes", "attachment_id");
    let Some(rows) = merged.get_mut("attachments") else {
        return false;
    };
    let mut taken: BTreeSet<String> = rows
        .values()
        .filter_map(|r| text(r, "path"))
        .map(String::from)
        .collect();
    let mut changed = false;
    for (id, row) in rows.iter_mut() {
        if !bodies.contains(id) {
            continue;
        }
        let path = text(row, "path").unwrap_or_default().to_string();
        let folder = names::parent(&path);
        if folder.is_empty() || folders.contains(folder) {
            continue;
        }
        taken.remove(&path);
        let free = free_path(names::base(&path), &taken);
        taken.insert(free.clone());
        set_path(row, &free);
        changed = true;
    }
    changed
}

/// Tasks: a missing parent drops the task; a missing folder moves it to the root; a missing status
/// becomes the first status; a missing assignee is cleared.
fn fix_tasks(merged: &mut Snapshot) -> bool {
    let tasks = ids(merged, "tasks");
    let folders = values(merged, "task_folders", "path");
    let people = ids(merged, "people");
    let statuses = values(merged, "statuses", "value");
    let first = first_status(merged);
    let Some(rows) = merged.get_mut("tasks") else {
        return false;
    };
    let before = rows.len();
    rows.retain(|_, row| text(row, "parent_id").is_none_or(|p| tasks.contains(p)));
    let mut changed = rows.len() != before;
    for row in rows.values_mut() {
        changed |= fix_column(
            row,
            "path",
            |p| folders.contains(p),
            Value::String(String::new()),
        );
        changed |= fix_column(row, "assignee_id", |p| people.contains(p), Value::Null);
        if let Some(first) = &first {
            changed |= fix_column(
                row,
                "status",
                |s| statuses.contains(s),
                Value::String(first.clone()),
            );
        }
    }
    changed
}

/// Sets `column` to `fallback` when it holds a value `valid` refuses. Returns true when it did.
fn fix_column(row: &mut Row, column: &str, valid: impl Fn(&str) -> bool, fallback: Value) -> bool {
    let bad = text(row, column).is_some_and(|v| !valid(v));
    if bad {
        row.insert(column.to_string(), fallback);
    }
    bad
}

/// The status with the lowest position (ties: the smaller value).
fn first_status(merged: &Snapshot) -> Option<String> {
    let rows = merged.get("statuses")?;
    let position = |r: &Row| {
        r.get("position")
            .and_then(Value::as_i64)
            .unwrap_or(i64::MAX)
    };
    rows.values()
        .min_by_key(|r| {
            (
                position(r),
                text(r, "value").unwrap_or_default().to_string(),
            )
        })
        .and_then(|r| text(r, "value"))
        .map(String::from)
}

/// Attachment links whose owner (a task, note, comment or person) is gone.
fn drop_ownerless_links(merged: &mut Snapshot) -> bool {
    let owners: BTreeMap<&str, BTreeSet<String>> = [
        ("task", ids(merged, "tasks")),
        ("note", ids(merged, "notes")),
        ("comment", ids(merged, "task_comments")),
        ("person", ids(merged, "people")),
    ]
    .into_iter()
    .collect();
    let Some(rows) = merged.get_mut("attachment_links") else {
        return false;
    };
    let before = rows.len();
    rows.retain(|_, row| {
        let kind = text(row, "owner_type").unwrap_or_default();
        let owner = text(row, "owner_id").unwrap_or_default();
        owners.get(kind).is_some_and(|alive| alive.contains(owner))
    });
    rows.len() != before
}
