//! Reconciling (spec §9): the merged state of base, local and remote snapshots. Pure: the same
//! three inputs give the same result on every device, so every device ends with the same rows.
//!
//! 1. Three-way merge per row, column by column: a change on one side wins; a column both sides
//!    changed takes the newer row's value (`updated_at`; equal times: the line that sorts
//!    higher); a deletion on either side wins.
//! 2. Unique keys: rows that collide after merging are merged or renamed, never dropped with
//!    their content (§9.2). Words and links that share a key become one row (`MERGE_ON_KEY`).
//! 3. References: rows pointing at something that no longer exists are fixed or dropped, parents
//!    first, until nothing changes (§9.3).

#[cfg(test)]
mod sim_tests;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use super::shards::{Row, Snapshot};
use crate::enums::rows::POSITION;
use crate::enums::{EnumKind, SourceType, TargetType};
use crate::names;
use crate::words;

/// The three versions a merge works from: the last agreed state and each side's.
#[derive(Clone, Copy)]
pub struct Sides<T> {
    pub base: T,
    pub local: T,
    pub remote: T,
}

/// The merged state of the three snapshots.
pub fn reconcile(sides: &Sides<&Snapshot>) -> Snapshot {
    let mut merged = merge_tables(sides);
    fix_unique_keys(&mut merged);
    fix_references(&mut merged);
    merged
}

// ---- 1. three-way merge

fn merge_tables(sides: &Sides<&Snapshot>) -> Snapshot {
    let empty = BTreeMap::new();
    let tables: BTreeSet<&'static str> = sides
        .base
        .keys()
        .chain(sides.local.keys())
        .chain(sides.remote.keys())
        .copied()
        .collect();
    let mut merged = Snapshot::new();
    for table in tables {
        let rows = Sides {
            base: sides.base.get(table).unwrap_or(&empty),
            local: sides.local.get(table).unwrap_or(&empty),
            remote: sides.remote.get(table).unwrap_or(&empty),
        };
        merged.insert(table, merge_table(rows));
    }
    merged
}

type Rows = BTreeMap<String, Row>;

fn merge_table(rows: Sides<&Rows>) -> Rows {
    let ids: BTreeSet<&String> = rows
        .base
        .keys()
        .chain(rows.local.keys())
        .chain(rows.remote.keys())
        .collect();
    let mut out = Rows::new();
    for id in ids {
        let row = merge_row(Sides {
            base: rows.base.get(id),
            local: rows.local.get(id),
            remote: rows.remote.get(id),
        });
        if let Some(row) = row {
            out.insert(id.clone(), row);
        }
    }
    out
}

fn merge_row(row: Sides<Option<&Row>>) -> Option<Row> {
    match (row.base, row.local, row.remote) {
        // Present at the last sync and gone on one side: deleted, whatever the other side did.
        (Some(_), None, _) | (Some(_), _, None) => None,
        (Some(base), Some(local), Some(remote)) => Some(merge_columns(Sides {
            base,
            local,
            remote,
        })),
        (None, Some(l), None) => Some(l.clone()),
        (None, None, Some(r)) => Some(r.clone()),
        (None, Some(l), Some(r)) => Some(newer(l, r).clone()),
        (None, None, None) => None,
    }
}

/// Each column from the side that changed it since the base; a column both changed comes from
/// the newer row. So a rename on one device and an edit on another both survive.
fn merge_columns(row: Sides<&Row>) -> Row {
    // Most rows changed on one side only (or neither): no column needs looking at.
    if row.local == row.base {
        return row.remote.clone();
    }
    if row.remote == row.base {
        return row.local.clone();
    }
    let winner = newer(row.local, row.remote);
    let columns: BTreeSet<&String> = row.local.keys().chain(row.remote.keys()).collect();
    let mut out = Row::new();
    for column in columns {
        let was = row.base.get(column);
        let mine = row.local.get(column);
        let theirs = row.remote.get(column);
        let value = if mine == was {
            theirs
        } else if theirs == was {
            mine
        } else {
            winner.get(column)
        };
        if let Some(value) = value {
            out.insert(column.clone(), value.clone());
        }
    }
    out
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
/// Link, path and enum tables: rows with the same key collapse into the one with the smallest id.
/// Nothing points at an enum or link row by id, so dropping the others leaves nothing dangling.
const KEEP_SMALLEST: &[(&str, &[&str])] = &[
    ("tend_enums", &["kind", "value"]),
    (
        "tend_links",
        &[
            "source_type",
            "source_id",
            "target_type",
            "target_kind",
            "target_value",
        ],
    ),
    ("my_day", &["task_id", "day"]),
    ("note_folders", &["path"]),
    ("task_folders", &["path"]),
];

/// Display names unique ignoring case within a group: a later row gets " 2", " 3", … so both
/// stay. (table, name column, group column)
const UNIQUE_NAMES: &[(&str, &str, &str)] = &[("tend_enums", "label", "kind")];

/// A table whose rows sharing a key become one row: the smallest id is kept, `merge` makes its
/// fields from the two rows, and the links (`tend_links`) of the others move to it. `merge` must be
/// pure, so every device makes the same row.
struct MergeRule {
    table: &'static str,
    key: &'static [&'static str],
    /// The link source type of this table's rows.
    source: SourceType,
    merge: fn(&Row, &Row) -> Row,
}

const MERGE_ON_KEY: &[MergeRule] = &[
    MergeRule {
        table: "words",
        key: &["title"],
        source: SourceType::Word,
        merge: newest_fill_empty,
    },
    MergeRule {
        table: "refs",
        key: &["url_key"],
        source: SourceType::Ref,
        merge: newest_fill_empty,
    },
];

fn fix_unique_keys(merged: &mut Snapshot) {
    // Before KEEP_SMALLEST, which then drops links doubled by the move.
    for rule in MERGE_ON_KEY {
        merge_on_key(merged, rule);
    }
    for (table, columns) in KEEP_SMALLEST {
        if let Some(rows) = merged.get_mut(table) {
            keep_smallest(rows, columns);
        }
    }
    for (table, column, group) in UNIQUE_NAMES {
        if let Some(rows) = merged.get_mut(table) {
            number_duplicates(rows, column, group);
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

fn is_empty(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) => true,
        Some(Value::String(s)) => s.is_empty(),
        _ => false,
    }
}

/// Every field from the newer row (`updated_at`, ties as in `newer`); a field empty there comes
/// from the other row.
fn newest_fill_empty(a: &Row, b: &Row) -> Row {
    let newest = newer(a, b);
    let other = if std::ptr::eq(newest, a) { b } else { a };
    let mut out = newest.clone();
    for (column, value) in other {
        if is_empty(out.get(column)) && !is_empty(Some(value)) {
            out.insert(column.clone(), value.clone());
        }
    }
    out
}

/// Ids of rows sharing a key (a key with an empty part is no key), smallest first; only groups of
/// two or more.
fn shared_keys(rows: &Rows, columns: &[&str]) -> Vec<Vec<String>> {
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (id, row) in rows {
        let key = key_of(row, columns);
        if key.iter().any(|v| is_empty(Some(v))) {
            continue;
        }
        let key = serde_json::to_string(&key).unwrap_or_default();
        groups.entry(key).or_default().push(id.clone());
    }
    groups.into_values().filter(|ids| ids.len() > 1).collect()
}

fn merge_on_key(merged: &mut Snapshot, rule: &MergeRule) {
    let Some(rows) = merged.get_mut(rule.table) else {
        return;
    };
    // Dropped id → the id kept.
    let mut moved: BTreeMap<String, String> = BTreeMap::new();
    for ids in shared_keys(rows, rule.key) {
        let keep = ids[0].clone();
        let mut row = rows[&keep].clone();
        for other in &ids[1..] {
            row = (rule.merge)(&row, &rows[other]);
            moved.insert(other.clone(), keep.clone());
        }
        row.insert("id".into(), Value::String(keep.clone()));
        for other in &ids[1..] {
            rows.remove(other);
        }
        rows.insert(keep, row);
    }
    move_links(merged, rule.source, &moved);
}

/// Points the links of `source` rows at the row each was merged into.
fn move_links(merged: &mut Snapshot, source: SourceType, moved: &BTreeMap<String, String>) {
    let Some(links) = merged.get_mut("tend_links") else {
        return;
    };
    for row in links.values_mut() {
        if text(row, "source_type") != Some(source.as_str()) {
            continue;
        }
        let kept = text(row, "source_id").and_then(|id| moved.get(id)).cloned();
        if let Some(kept) = kept {
            row.insert("source_id".into(), Value::String(kept));
        }
    }
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

fn number_duplicates(rows: &mut Rows, column: &str, group: &str) {
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let ids: Vec<String> = rows.keys().cloned().collect();
    for id in ids {
        let Some(row) = rows.get_mut(&id) else {
            continue;
        };
        let name = text(row, column).unwrap_or_default().to_string();
        let set = text(row, group).unwrap_or_default().to_string();
        let mut candidate = name.clone();
        let mut n = 2;
        while seen.contains(&(set.clone(), candidate.to_lowercase())) {
            candidate = format!("{name} {n}");
            n += 1;
        }
        seen.insert((set, candidate.to_lowercase()));
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
        // Every suffix taken (thousands of clashes): the row's own id can't clash.
        let free = free_path(&path, &taken).unwrap_or_else(|| with_id(&path, &id));
        taken.insert(free.clone());
        set_path(row, &free);
    }
}

/// A path's folder, file stem and extension (with its dot, or empty).
fn split_path(path: &str) -> (&str, &str, String) {
    let folder = names::parent(path);
    let file = names::base(path);
    match file.rsplit_once('.') {
        Some((stem, ext)) => (folder, stem, format!(".{ext}")),
        None => (folder, file, String::new()),
    }
}

/// `folder/name.ext` with the name given the next free suffix among `taken`; `None` when every
/// suffix is taken.
fn free_path(path: &str, taken: &BTreeSet<String>) -> Option<String> {
    let (folder, stem, ext) = split_path(path);
    let is_taken = |n: &str| Ok(taken.contains(&names::join(folder, &format!("{n}{ext}"))));
    let name = names::free(stem, is_taken).ok()?;
    Some(names::join(folder, &format!("{name}{ext}")))
}

/// `folder/name_<id>.ext`: unique because the id is, and within the name rule (lowercase).
fn with_id(path: &str, id: &str) -> String {
    let (folder, stem, ext) = split_path(path);
    let name = names::with_tag(stem, &id.to_lowercase());
    names::join(folder, &format!("{name}{ext}"))
}

fn set_path(row: &mut Row, path: &str) {
    row.insert("path".into(), Value::String(path.to_string()));
    row.insert("name".into(), Value::String(names::base(path).to_string()));
}

// ---- 3. references

/// Applies the reference rules until a pass changes nothing (a dropped task drops its subtasks,
/// which drops their comments, and so on). A pass that changes something changes at least one
/// row, so one pass per row (plus the last, quiet one) is always enough; the cap only guards
/// against a rule that would undo another.
fn fix_references(merged: &mut Snapshot) {
    let rows: usize = merged.values().map(BTreeMap::len).sum();
    for _ in 0..=rows {
        let changed = fix_pass(merged);
        if !changed {
            return;
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
    changed |= drop_missing(merged, "person_contacts", "person_id", "people");
    changed |= fix_tasks(merged);
    for child in ["my_day", "task_comments"] {
        changed |= drop_missing(merged, child, "task_id", "tasks");
    }
    changed |= fix_words(merged);
    changed |= fix_stacks(merged);
    changed |= drop_dangling_links(merged);
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
        let base = names::base(&path);
        let free = free_path(base, &taken).unwrap_or_else(|| with_id(base, id));
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
    let statuses = enum_values(merged, EnumKind::TaskStatus);
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

/// Sets `column` to `fallback` when it holds a value `valid` refuses. Returns true when that changed
/// the row: a fallback `valid` refuses too (no root folder row) must not count as a change every
/// pass.
fn fix_column(row: &mut Row, column: &str, valid: impl Fn(&str) -> bool, fallback: Value) -> bool {
    let bad = text(row, column).is_some_and(|v| !valid(v));
    let changes = bad && row.get(column) != Some(&fallback);
    if changes {
        row.insert(column.to_string(), fallback);
    }
    changes
}

/// A word whose kind is gone becomes `OTHER`: never `WORD`, whose lowercasing could make a title
/// clash after the unique keys were settled.
fn fix_words(merged: &mut Snapshot) -> bool {
    let kinds = enum_values(merged, EnumKind::WordKind);
    let Some(rows) = merged.get_mut("words") else {
        return false;
    };
    let mut changed = false;
    for row in rows.values_mut() {
        let fallback = Value::String(words::OTHER.into());
        changed |= fix_column(row, "kind", |k| kinds.contains(k), fallback);
    }
    changed
}

/// Snaps and stacks whose stack is gone go to the board; stacks nested in each other (each moved
/// into the other on a different device) are untangled by moving the one with the larger id out.
fn fix_stacks(merged: &mut Snapshot) -> bool {
    let stacks = ids(merged, "snap_stacks");
    let mut changed = false;
    for (table, column) in [("snaps", "stack_id"), ("snap_stacks", "parent_id")] {
        let Some(rows) = merged.get_mut(table) else {
            continue;
        };
        for row in rows.values_mut() {
            changed |= fix_column(row, column, |s| stacks.contains(s), Value::Null);
        }
    }
    changed | break_stack_cycle(merged)
}

/// The parent of each stack that has one.
fn stack_parents(merged: &Snapshot) -> BTreeMap<String, String> {
    let rows = merged
        .get("snap_stacks")
        .into_iter()
        .flat_map(BTreeMap::iter);
    rows.filter_map(|(id, r)| Some((id.clone(), text(r, "parent_id")?.to_string())))
        .collect()
}

/// The stacks of one cycle, if there is any. A walk up from a stack in a cycle comes back to it
/// within as many steps as there are stacks.
fn stack_cycle(parents: &BTreeMap<String, String>) -> Option<Vec<String>> {
    for start in parents.keys() {
        let mut path = vec![start.clone()];
        let mut current = start;
        for _ in 0..parents.len() {
            let Some(parent) = parents.get(current) else {
                break;
            };
            if parent == start {
                return Some(path);
            }
            path.push(parent.clone());
            current = parent;
        }
    }
    None
}

/// Moves the stack with the largest id of one cycle to the board. One per pass: the passes repeat
/// until none is left.
fn break_stack_cycle(merged: &mut Snapshot) -> bool {
    let Some(cycle) = stack_cycle(&stack_parents(merged)) else {
        return false;
    };
    let Some(last) = cycle.iter().max() else {
        return false;
    };
    let row = merged
        .get_mut("snap_stacks")
        .and_then(|rows| rows.get_mut(last));
    let Some(row) = row else {
        return false;
    };
    row.insert("parent_id".into(), Value::Null);
    true
}

/// The values of one enum kind.
fn enum_values(merged: &Snapshot, kind: EnumKind) -> BTreeSet<String> {
    enum_rows(merged, kind)
        .filter_map(|r| text(r, "value"))
        .map(String::from)
        .collect()
}

fn enum_rows(merged: &Snapshot, kind: EnumKind) -> impl Iterator<Item = &Row> {
    let rows = merged
        .get("tend_enums")
        .into_iter()
        .flat_map(BTreeMap::values);
    rows.filter(move |r| text(r, "kind") == Some(kind.as_str()))
}

/// An enum row's `meta.position` (its JSON text parsed); missing sorts last.
fn position(row: &Row) -> i64 {
    let meta: Option<Value> = text(row, "meta").and_then(|m| serde_json::from_str(m).ok());
    let position = meta
        .as_ref()
        .and_then(|m| m.get(POSITION))
        .and_then(Value::as_i64);
    position.unwrap_or(i64::MAX)
}

/// The status with the lowest position (ties: the smaller value).
fn first_status(merged: &Snapshot) -> Option<String> {
    enum_rows(merged, EnumKind::TaskStatus)
        .min_by_key(|r| {
            (
                position(r),
                text(r, "value").unwrap_or_default().to_string(),
            )
        })
        .and_then(|r| text(r, "value"))
        .map(String::from)
}

/// Links whose source row, attachment or enum value is gone.
fn drop_dangling_links(merged: &mut Snapshot) -> bool {
    let sources: BTreeMap<&str, BTreeSet<String>> = SourceType::ALL
        .into_iter()
        .map(|s| (s.as_str(), ids(merged, s.table())))
        .collect();
    let attachments = ids(merged, "attachments");
    let enums: BTreeSet<(String, String)> = merged
        .get("tend_enums")
        .into_iter()
        .flat_map(BTreeMap::values)
        .map(|r| {
            let kind = text(r, "kind").unwrap_or_default().to_string();
            (kind, text(r, "value").unwrap_or_default().to_string())
        })
        .collect();
    let target_alive = |row: &Row| {
        let kind = text(row, "target_kind").unwrap_or_default().to_string();
        let value = text(row, "target_value").unwrap_or_default().to_string();
        match text(row, "target_type") {
            Some(t) if t == TargetType::Enum.as_str() => enums.contains(&(kind, value)),
            Some(t) if t == TargetType::Attachment.as_str() => attachments.contains(&value),
            _ => false,
        }
    };
    let source_alive = |row: &Row| {
        let alive = text(row, "source_type").and_then(|t| sources.get(t));
        let id = text(row, "source_id").unwrap_or_default();
        alive.is_some_and(|ids| ids.contains(id))
    };
    let Some(rows) = merged.get_mut("tend_links") else {
        return false;
    };
    let before = rows.len();
    rows.retain(|_, row| source_alive(row) && target_alive(row));
    rows.len() != before
}
