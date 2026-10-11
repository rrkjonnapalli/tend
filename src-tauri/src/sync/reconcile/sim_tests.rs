//! Simulation (spec §19): three devices share one remote and make random changes, with clocks
//! set wrong on purpose, syncing in random order and sometimes crashing mid-run. After every sync
//! the merged state must hold every rule; at the end every device must hold the same rows, no
//! deleted row may have come back, and nothing created and kept may be lost.

use std::collections::BTreeSet;

use serde_json::{json, Value};

use super::*;

const DEVICES: usize = 3;
const SEEDS: u64 = 150;
const STEPS: usize = 120;

/// A small deterministic random source (xorshift), so a failing seed can be replayed.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn pick<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        if items.is_empty() {
            return None;
        }
        Some(&items[self.below(items.len())])
    }

    fn chance(&mut self, percent: u64) -> bool {
        self.next() % 100 < percent
    }
}

struct Device {
    snap: Snapshot,
    /// Index of the remote commit this device last synced with.
    base: usize,
    /// The commit this device pushed but didn't get to record as its base (a crash after push).
    pending: Option<usize>,
    /// This device's clock error, in the same units as `now`.
    skew: i64,
    next_id: u32,
}

struct World {
    rng: Rng,
    remote: Vec<Snapshot>,
    devices: Vec<Device>,
    now: i64,
    /// Ids (ULID-like, made by a device) that some device deleted.
    deleted: BTreeSet<String>,
    /// People made by a device; the ones never deleted must survive.
    people: BTreeSet<String>,
}

fn row(v: Value) -> Row {
    v.as_object().unwrap().clone()
}

/// What every vault starts with.
fn start() -> Snapshot {
    let mut s = Snapshot::new();
    for table in super::super::shards::table_names() {
        s.insert(table, Rows::new());
    }
    let enums = s.get_mut("tend_enums").unwrap();
    for (value, label, position) in [("NOT_STARTED", "Not started", 0), ("DONE", "Done", 1000)] {
        let id = format!("TASK_STATUS:{value}");
        let meta = json!({ "position": position }).to_string();
        enums.insert(
            id.clone(),
            row(json!({ "id": id, "kind": EnumKind::TaskStatus.as_str(), "value": value, "label": label, "meta": meta, "updated_at": 0 })),
        );
    }
    for table in ["task_folders", "note_folders"] {
        let root = row(json!({ "id": "root", "path": "", "updated_at": 0 }));
        s.get_mut(table).unwrap().insert("root".into(), root);
    }
    s
}

impl World {
    fn new(seed: u64) -> World {
        let mut rng = Rng(seed * 2_654_435_761 + 1);
        let devices = (0..DEVICES)
            .map(|_| Device {
                snap: start(),
                base: 0,
                pending: None,
                skew: rng.below(4000) as i64 - 2000,
                next_id: 0,
            })
            .collect();
        World {
            rng,
            remote: vec![start()],
            devices,
            now: 0,
            deleted: BTreeSet::new(),
            people: BTreeSet::new(),
        }
    }

    /// One sync run. Applying the merge also moves the base to the fetched head (in the same
    /// transaction), so a crash before the push can't make a later delete look like a remote add.
    /// The commit about to be pushed is recorded as pending first; a later run that finds it on
    /// the remote takes it as the base, so a crash after the push can't either.
    fn sync(&mut self, d: usize, crash: Crash) {
        if let Some(pushed) = self.devices[d].pending.take() {
            if pushed < self.remote.len() {
                self.devices[d].base = pushed;
            }
        }
        let head_index = self.remote.len() - 1;
        let base = &self.remote[self.devices[d].base];
        let head = &self.remote[head_index];
        let merged = reconcile(&Sides {
            base,
            local: &self.devices[d].snap,
            remote: head,
        });
        check_rules(&merged);
        let changed = merged != *head;
        let device = &mut self.devices[d];
        device.snap = merged.clone();
        device.base = head_index;
        if matches!(crash, Crash::BeforePush) || !changed {
            return;
        }
        device.pending = Some(head_index + 1);
        self.remote.push(merged);
        if matches!(crash, Crash::AfterPush) {
            return;
        }
        let device = &mut self.devices[d];
        device.pending = None;
        device.base = self.remote.len() - 1;
    }
}

enum Crash {
    None,
    /// After the merge is applied, before the push.
    BeforePush,
    /// After the push, before the base moves to the pushed commit.
    AfterPush,
}

// ---- reading a device's rows

fn keys(s: &Snapshot, table: &str) -> Vec<String> {
    s[table].keys().cloned().collect()
}

fn texts(s: &Snapshot, table: &str, column: &str) -> Vec<String> {
    s[table]
        .values()
        .filter_map(|r| text(r, column))
        .map(String::from)
        .collect()
}

// ---- random changes, each keeping the device's own rules (as the app does)

impl World {
    fn stamp(&mut self, d: usize) -> i64 {
        self.now += 10;
        self.now + self.devices[d].skew
    }

    fn new_id(&mut self, d: usize, prefix: &str) -> String {
        let device = &mut self.devices[d];
        device.next_id += 1;
        format!("{prefix}{:05}-{d}", device.next_id)
    }

    fn step(&mut self, d: usize) {
        let ops: [fn(&mut World, usize); 18] = [
            add_person,
            edit_person,
            delete_person,
            add_task_folder,
            rename_task_folder,
            delete_task_folder,
            add_task,
            edit_task,
            delete_task,
            add_comment,
            tag_task,
            untag_task,
            add_label,
            rename_label,
            label_task,
            add_note,
            rename_note,
            delete_note,
        ];
        let op = ops[self.rng.below(ops.len())];
        op(self, d);
    }
}

fn add_person(w: &mut World, d: usize) {
    let id = w.new_id(d, "p");
    let at = w.stamp(d);
    w.people.insert(id.clone());
    let person = row(json!({ "id": id, "first_name": "Ann", "updated_at": at }));
    w.devices[d]
        .snap
        .get_mut("people")
        .unwrap()
        .insert(id, person);
}

fn edit_person(w: &mut World, d: usize) {
    let ids = keys(&w.devices[d].snap, "people");
    let Some(id) = w.rng.pick(&ids).cloned() else {
        return;
    };
    let at = w.stamp(d);
    let name = ["Bo", "Cy", "Di"][w.rng.below(3)];
    let p = w.devices[d]
        .snap
        .get_mut("people")
        .unwrap()
        .get_mut(&id)
        .unwrap();
    p.insert("first_name".into(), json!(name));
    p.insert("updated_at".into(), json!(at));
}

fn delete_person(w: &mut World, d: usize) {
    let ids = keys(&w.devices[d].snap, "people");
    let Some(id) = w.rng.pick(&ids).cloned() else {
        return;
    };
    w.deleted.insert(id.clone());
    let snap = &mut w.devices[d].snap;
    snap.get_mut("people").unwrap().remove(&id);
    // ON DELETE SET NULL, without touching updated_at.
    for task in snap.get_mut("tasks").unwrap().values_mut() {
        if text(task, "assignee_id") == Some(id.as_str()) {
            task.insert("assignee_id".into(), Value::Null);
        }
    }
}

const FOLDERS: [&str; 6] = ["a", "b", "c", "a/x", "b/y", "a/x/z"];

fn add_task_folder(w: &mut World, d: usize) {
    let path = *w.rng.pick(&FOLDERS).unwrap();
    let paths = texts(&w.devices[d].snap, "task_folders", "path");
    let parent = names::parent(path).to_string();
    if paths.contains(&path.to_string()) || !paths.contains(&parent) {
        return;
    }
    let id = w.new_id(d, "f");
    let at = w.stamp(d);
    let folder = row(json!({ "id": id, "path": path, "updated_at": at }));
    w.devices[d]
        .snap
        .get_mut("task_folders")
        .unwrap()
        .insert(id, folder);
}

fn rename_task_folder(w: &mut World, d: usize) {
    let folders: Vec<(String, String)> = w.devices[d].snap["task_folders"]
        .iter()
        .map(|(id, r)| (id.clone(), text(r, "path").unwrap().to_string()))
        .filter(|(_, p)| !p.is_empty())
        .collect();
    let Some((_, old)) = w.rng.pick(&folders).cloned() else {
        return;
    };
    let name = ["d", "e", "a"][w.rng.below(3)];
    let new = names::join(names::parent(&old), name);
    let paths = texts(&w.devices[d].snap, "task_folders", "path");
    if paths.contains(&new) {
        return;
    }
    let at = w.stamp(d);
    let moved = |p: &str| -> Option<String> {
        if p == old {
            return Some(new.clone());
        }
        p.strip_prefix(&format!("{old}/"))
            .map(|rest| format!("{new}/{rest}"))
    };
    let snap = &mut w.devices[d].snap;
    for table in ["task_folders", "tasks"] {
        for r in snap.get_mut(table).unwrap().values_mut() {
            let Some(next) = moved(text(r, "path").unwrap()) else {
                continue;
            };
            r.insert("path".into(), json!(next));
            r.insert("updated_at".into(), json!(at));
        }
    }
}

fn delete_task_folder(w: &mut World, d: usize) {
    let paths: Vec<String> = texts(&w.devices[d].snap, "task_folders", "path")
        .into_iter()
        .filter(|p| !p.is_empty())
        .collect();
    let Some(path) = w.rng.pick(&paths).cloned() else {
        return;
    };
    let inside = |p: &str| names::is_under(p, &path);
    let snap = &mut w.devices[d].snap;
    let gone_folders: Vec<String> = snap["task_folders"]
        .iter()
        .filter(|(_, r)| inside(text(r, "path").unwrap()))
        .map(|(id, _)| id.clone())
        .collect();
    let gone_tasks: Vec<String> = snap["tasks"]
        .iter()
        .filter(|(_, r)| inside(text(r, "path").unwrap()))
        .map(|(id, _)| id.clone())
        .collect();
    for id in gone_folders {
        snap.get_mut("task_folders").unwrap().remove(&id);
        w.deleted.insert(id);
    }
    for id in gone_tasks {
        remove_task(&mut w.deleted, snap, &id);
    }
}

fn add_task(w: &mut World, d: usize) {
    let snap = &w.devices[d].snap;
    let paths = texts(snap, "task_folders", "path");
    let statuses = kind_values(snap, EnumKind::TaskStatus);
    let people = keys(snap, "people");
    let tops: Vec<(String, String)> = snap["tasks"]
        .iter()
        .filter(|(_, r)| r["parent_id"].is_null())
        .map(|(id, r)| (id.clone(), text(r, "path").unwrap().to_string()))
        .collect();
    let mut path = w.rng.pick(&paths).cloned().unwrap();
    let mut parent = Value::Null;
    if w.rng.chance(30) {
        if let Some((id, p)) = w.rng.pick(&tops).cloned() {
            parent = json!(id);
            path = p;
        }
    }
    let status = w.rng.pick(&statuses).cloned().unwrap();
    let assignee = w
        .rng
        .pick(&people)
        .cloned()
        .map_or(Value::Null, Value::from);
    let id = w.new_id(d, "t");
    let at = w.stamp(d);
    let task = row(json!({
        "id": id, "path": path, "parent_id": parent, "title": "T", "status": status,
        "assignee_id": assignee, "order_key": "V", "updated_at": at,
    }));
    w.devices[d].snap.get_mut("tasks").unwrap().insert(id, task);
}

fn edit_task(w: &mut World, d: usize) {
    let ids = keys(&w.devices[d].snap, "tasks");
    let statuses = kind_values(&w.devices[d].snap, EnumKind::TaskStatus);
    let Some(id) = w.rng.pick(&ids).cloned() else {
        return;
    };
    let status = w.rng.pick(&statuses).cloned().unwrap();
    let title = ["A", "B", "C"][w.rng.below(3)];
    let at = w.stamp(d);
    let t = w.devices[d]
        .snap
        .get_mut("tasks")
        .unwrap()
        .get_mut(&id)
        .unwrap();
    t.insert("title".into(), json!(title));
    t.insert("status".into(), json!(status));
    t.insert("updated_at".into(), json!(at));
}

/// Deletes a task, its subtasks and everything hanging off them (ON DELETE CASCADE).
fn remove_task(deleted: &mut BTreeSet<String>, snap: &mut Snapshot, id: &str) {
    let subs: Vec<String> = snap["tasks"]
        .iter()
        .filter(|(_, r)| text(r, "parent_id") == Some(id))
        .map(|(id, _)| id.clone())
        .collect();
    for sub in subs {
        remove_task(deleted, snap, &sub);
    }
    snap.get_mut("tasks").unwrap().remove(id);
    deleted.insert(id.to_string());
    for child in ["task_comments", "my_day"] {
        let rows = snap.get_mut(child).unwrap();
        let gone: Vec<String> = rows
            .iter()
            .filter(|(_, r)| text(r, "task_id") == Some(id))
            .map(|(id, _)| id.clone())
            .collect();
        for g in gone {
            rows.remove(&g);
            deleted.insert(g);
        }
    }
    // The trigger tasks_drop_links.
    let links = snap.get_mut("tend_links").unwrap();
    let gone: Vec<String> = links
        .iter()
        .filter(|(_, r)| {
            text(r, "source_type") == Some(SourceType::Task.as_str())
                && text(r, "source_id") == Some(id)
        })
        .map(|(id, _)| id.clone())
        .collect();
    for g in gone {
        links.remove(&g);
        deleted.insert(g);
    }
}

fn delete_task(w: &mut World, d: usize) {
    let ids = keys(&w.devices[d].snap, "tasks");
    let Some(id) = w.rng.pick(&ids).cloned() else {
        return;
    };
    remove_task(&mut w.deleted, &mut w.devices[d].snap, &id);
}

fn add_comment(w: &mut World, d: usize) {
    let ids = keys(&w.devices[d].snap, "tasks");
    let Some(task) = w.rng.pick(&ids).cloned() else {
        return;
    };
    let id = w.new_id(d, "c");
    let at = w.stamp(d);
    let comment = row(json!({ "id": id, "task_id": task, "body": "hi", "updated_at": at }));
    w.devices[d]
        .snap
        .get_mut("task_comments")
        .unwrap()
        .insert(id, comment);
}

/// The values of one enum kind.
fn kind_values(snap: &Snapshot, kind: EnumKind) -> Vec<String> {
    kind_rows(snap, kind)
        .map(|(_, r)| text(r, "value").unwrap().to_string())
        .collect()
}

fn kind_rows(snap: &Snapshot, kind: EnumKind) -> impl Iterator<Item = (&String, &Row)> {
    snap["tend_enums"]
        .iter()
        .filter(move |(_, r)| text(r, "kind") == Some(kind.as_str()))
}

/// Adds an enum row unless the kind already has the value.
fn ensure_enum(w: &mut World, d: usize, kind: EnumKind, value: &str, label: &str) {
    if kind_values(&w.devices[d].snap, kind).contains(&value.to_string()) {
        return;
    }
    let id = w.new_id(d, "e");
    let at = w.stamp(d);
    let r = row(json!({
        "id": id, "kind": kind.as_str(), "value": value, "label": label,
        "meta": "{}", "updated_at": at,
    }));
    w.devices[d]
        .snap
        .get_mut("tend_enums")
        .unwrap()
        .insert(id, r);
}

/// Links a task to an enum value unless it already is.
fn link(w: &mut World, d: usize, task: String, kind: EnumKind, value: String) {
    let linked = w.devices[d].snap["tend_links"].values().any(|r| {
        text(r, "source_id") == Some(task.as_str())
            && text(r, "target_kind") == Some(kind.as_str())
            && text(r, "target_value") == Some(value.as_str())
    });
    if linked {
        return;
    }
    let id = w.new_id(d, "l");
    let at = w.stamp(d);
    let r = row(json!({
        "id": id,
        "source_type": SourceType::Task.as_str(),
        "source_id": task,
        "target_type": TargetType::Enum.as_str(),
        "target_kind": kind.as_str(),
        "target_value": value,
        "updated_at": at,
    }));
    w.devices[d]
        .snap
        .get_mut("tend_links")
        .unwrap()
        .insert(id, r);
}

fn tag_task(w: &mut World, d: usize) {
    let ids = keys(&w.devices[d].snap, "tasks");
    let Some(task) = w.rng.pick(&ids).cloned() else {
        return;
    };
    let tag = ["ops", "ui", "db"][w.rng.below(3)];
    let value = tag.to_uppercase();
    ensure_enum(w, d, EnumKind::Tag, &value, tag);
    link(w, d, task, EnumKind::Tag, value);
}

fn untag_task(w: &mut World, d: usize) {
    let ids = keys(&w.devices[d].snap, "tend_links");
    let Some(id) = w.rng.pick(&ids).cloned() else {
        return;
    };
    w.devices[d].snap.get_mut("tend_links").unwrap().remove(&id);
    w.deleted.insert(id);
}

const LABELS: [&str; 4] = ["Review", "Script", "Blocked", "Ship"];

fn label_used(snap: &Snapshot, name: &str) -> bool {
    kind_rows(snap, EnumKind::TaskLabel)
        .any(|(_, r)| text(r, "label").unwrap().eq_ignore_ascii_case(name))
}

fn add_label(w: &mut World, d: usize) {
    let name = *w.rng.pick(&LABELS).unwrap();
    let value = names::slug(name).unwrap().to_uppercase();
    if label_used(&w.devices[d].snap, name) {
        return;
    }
    ensure_enum(w, d, EnumKind::TaskLabel, &value, name);
}

fn rename_label(w: &mut World, d: usize) {
    let ids: Vec<String> = kind_rows(&w.devices[d].snap, EnumKind::TaskLabel)
        .map(|(id, _)| id.clone())
        .collect();
    let Some(id) = w.rng.pick(&ids).cloned() else {
        return;
    };
    let name = *w.rng.pick(&LABELS).unwrap();
    if label_used(&w.devices[d].snap, name) {
        return;
    }
    let at = w.stamp(d);
    let l = w.devices[d]
        .snap
        .get_mut("tend_enums")
        .unwrap()
        .get_mut(&id)
        .unwrap();
    l.insert("label".into(), json!(name));
    l.insert("updated_at".into(), json!(at));
}

fn label_task(w: &mut World, d: usize) {
    let tasks = keys(&w.devices[d].snap, "tasks");
    let labels = kind_values(&w.devices[d].snap, EnumKind::TaskLabel);
    let (Some(task), Some(label)) = (w.rng.pick(&tasks).cloned(), w.rng.pick(&labels).cloned())
    else {
        return;
    };
    link(w, d, task, EnumKind::TaskLabel, label);
}

fn add_note(w: &mut World, d: usize) {
    let name = ["plan", "idea", "log"][w.rng.below(3)];
    let path = format!("{name}.md");
    if texts(&w.devices[d].snap, "attachments", "path").contains(&path) {
        return;
    }
    let attachment = w.new_id(d, "a");
    let note = w.new_id(d, "n");
    let at = w.stamp(d);
    let snap = &mut w.devices[d].snap;
    snap.get_mut("attachments").unwrap().insert(
        attachment.clone(),
        row(json!({ "id": attachment, "path": path, "name": path, "updated_at": at })),
    );
    snap.get_mut("notes").unwrap().insert(
        note.clone(),
        row(json!({ "id": note, "attachment_id": attachment, "updated_at": at })),
    );
}

fn rename_note(w: &mut World, d: usize) {
    let bodies = texts(&w.devices[d].snap, "notes", "attachment_id");
    let Some(id) = w.rng.pick(&bodies).cloned() else {
        return;
    };
    let name = ["plan", "idea", "log", "todo"][w.rng.below(4)];
    let path = format!("{name}.md");
    if texts(&w.devices[d].snap, "attachments", "path").contains(&path) {
        return;
    }
    let at = w.stamp(d);
    let a = w.devices[d]
        .snap
        .get_mut("attachments")
        .unwrap()
        .get_mut(&id)
        .unwrap();
    a.insert("path".into(), json!(path));
    a.insert("name".into(), json!(path));
    a.insert("updated_at".into(), json!(at));
}

fn delete_note(w: &mut World, d: usize) {
    let ids = keys(&w.devices[d].snap, "notes");
    let Some(id) = w.rng.pick(&ids).cloned() else {
        return;
    };
    let snap = &mut w.devices[d].snap;
    let note = snap.get_mut("notes").unwrap().remove(&id).unwrap();
    let attachment = text(&note, "attachment_id").unwrap().to_string();
    snap.get_mut("attachments").unwrap().remove(&attachment);
    w.deleted.insert(id);
    w.deleted.insert(attachment);
}

// ---- the rules every merged state must hold

fn check_rules(s: &Snapshot) {
    let has = |table: &str, id: &str| s[table].contains_key(id);
    let folder_paths = texts(s, "task_folders", "path");
    let statuses = kind_values(s, EnumKind::TaskStatus);
    for t in s["tasks"].values() {
        assert!(
            folder_paths.contains(&text(t, "path").unwrap().to_string()),
            "task folder: {t:?}"
        );
        assert!(
            statuses.contains(&text(t, "status").unwrap().to_string()),
            "task status: {t:?}"
        );
        if let Some(p) = text(t, "parent_id") {
            assert!(has("tasks", p), "task parent: {t:?}");
        }
        if let Some(p) = text(t, "assignee_id") {
            assert!(has("people", p), "task assignee: {t:?}");
        }
    }
    for (child, column, parent) in [
        ("task_comments", "task_id", "tasks"),
        ("tend_links", "source_id", "tasks"),
        ("notes", "attachment_id", "attachments"),
    ] {
        for r in s[child].values() {
            assert!(
                has(parent, text(r, column).unwrap()),
                "{child}.{column}: {r:?}"
            );
        }
    }
    for f in texts(s, "task_folders", "path") {
        let parent = names::parent(&f).to_string();
        assert!(
            f.is_empty() || folder_paths.contains(&parent),
            "folder parent: {f}"
        );
    }
    assert_unique(&texts(s, "task_folders", "path"), "task folder paths");
    assert_unique(&texts(s, "attachments", "path"), "attachment paths");
    let lower: Vec<String> = kind_rows(s, EnumKind::TaskLabel)
        .map(|(_, r)| text(r, "label").unwrap().to_lowercase())
        .collect();
    assert_unique(&lower, "label names");
    let enum_keys: Vec<String> = s["tend_enums"]
        .values()
        .map(|r| format!("{}|{}", text(r, "kind").unwrap(), text(r, "value").unwrap()))
        .collect();
    assert_unique(&enum_keys, "enum kind + value");
    let mut links = Vec::new();
    for r in s["tend_links"].values() {
        let kind = r["target_kind"].as_str().unwrap();
        let value = text(r, "target_value").unwrap().to_string();
        assert!(
            enum_keys.contains(&format!("{kind}|{value}")),
            "link target: {r:?}"
        );
        links.push(format!("{}|{kind}|{value}", text(r, "source_id").unwrap()));
    }
    assert_unique(&links, "links");
}

fn assert_unique(items: &[String], what: &str) {
    let set: BTreeSet<&String> = items.iter().collect();
    assert_eq!(set.len(), items.len(), "{what} not unique: {items:?}");
}

// ---- the run

fn run(seed: u64) {
    let mut w = World::new(seed);
    for _ in 0..STEPS {
        let d = w.rng.below(DEVICES);
        if w.rng.chance(25) {
            let crash = match w.rng.below(10) {
                0 => Crash::BeforePush,
                1 => Crash::AfterPush,
                _ => Crash::None,
            };
            w.sync(d, crash);
        } else {
            w.step(d);
        }
    }
    // Everyone syncs twice, with no more changes: all devices must agree.
    for _ in 0..2 {
        for d in 0..DEVICES {
            w.sync(d, Crash::None);
        }
    }
    let commits = w.remote.len();
    for d in 0..DEVICES {
        w.sync(d, Crash::None);
    }
    assert_eq!(
        w.remote.len(),
        commits,
        "seed {seed}: a sync with nothing new made a commit"
    );
    let head = w.remote.last().unwrap().clone();
    for (d, device) in w.devices.iter().enumerate() {
        assert_eq!(device.snap, head, "seed {seed}: device {d} differs");
    }
    assert_eq!(
        reconcile(&Sides {
            base: &head,
            local: &head,
            remote: &head
        }),
        head,
        "seed {seed}: merging the result changed it"
    );
    for (table, rows) in &head {
        for id in rows.keys() {
            assert!(
                !w.deleted.contains(id),
                "seed {seed}: deleted {table} {id} came back"
            );
        }
    }
    for id in w.people.difference(&w.deleted) {
        assert!(
            head["people"].contains_key(id),
            "seed {seed}: person {id} was lost"
        );
    }
}

#[test]
fn devices_converge_and_keep_every_rule() {
    for seed in 0..SEEDS {
        run(seed);
    }
}
