//! Commands called the way the UI calls them: JSON arguments through the IPC layer.

use serde_json::{json, Value};
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{
    get_ipc_response, mock_builder, mock_context, noop_assets, MockRuntime, INVOKE_KEY,
};
use tauri::webview::InvokeRequest;
use tauri::{Manager, WebviewWindow, WebviewWindowBuilder};

use crate::config::ConfigDir;

pub struct Harness {
    pub vault: tempfile::TempDir,
    _config: tempfile::TempDir,
    _app: tauri::App<MockRuntime>,
    webview: WebviewWindow<MockRuntime>,
}

impl Harness {
    /// A mock app with a fresh vault already open.
    pub fn new() -> Self {
        let config = tempfile::tempdir().unwrap();
        let vault = tempfile::tempdir().unwrap();
        let app = crate::with_commands(mock_builder())
            .build(mock_context(noop_assets()))
            .unwrap();
        app.manage(ConfigDir(config.path().to_path_buf()));
        let webview = WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        let h = Harness {
            vault,
            _config: config,
            _app: app,
            webview,
        };
        let check = h.ok("vault_open", json!({ "path": h.vault.path() }));
        assert_eq!(check["ok"], json!(true));
        h
    }

    pub fn call(&self, cmd: &str, args: Value) -> Result<Value, Value> {
        let request = InvokeRequest {
            cmd: cmd.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: InvokeBody::Json(args),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        };
        get_ipc_response(&self.webview, request).map(|b| b.deserialize::<Value>().unwrap())
    }

    pub fn ok(&self, cmd: &str, args: Value) -> Value {
        self.call(cmd, args)
            .unwrap_or_else(|e| panic!("{cmd} failed: {e}"))
    }
}

#[test]
fn notes_round_trip_through_ipc() {
    let h = Harness::new();
    h.ok("folder_create", json!({ "path": "work" }));
    let note = h.ok("note_create", json!({ "folder": "work", "title": "Plan" }));
    let id = note["id"].as_str().unwrap().to_string();
    assert_eq!(note["name"], "plan");

    h.ok(
        "note_write",
        json!({ "id": id, "text": "# Q4 plan\n\nship" }),
    );
    let doc = h.ok("note_read", json!({ "id": id }));
    assert_eq!(doc["text"], "# Q4 plan\n\nship");
    assert!(doc["modifiedAt"].is_number());

    let tags = h.ok("note_tags_set", json!({ "id": id, "tags": ["ops", "Q4"] }));
    assert_eq!(tags, json!(["ops", "q4"]));

    h.ok(
        "item_rename",
        json!({ "item": { "kind": "note", "id": id }, "name": "Q4 plan" }),
    );
    h.ok(
        "item_move",
        json!({ "item": { "kind": "folder", "path": "work" }, "target": "archive" }),
    );
    assert!(h.vault.path().join("archive/work/q4_plan.md").is_file());

    let tree = h.ok("notes_tree", json!({}));
    assert_eq!(tree["folders"], json!(["archive", "archive/work"]));
    assert_eq!(tree["notes"][0]["folder"], "archive/work");
    assert_eq!(tree["notes"][0]["name"], "q4_plan");

    let err = h
        .call("note_create", json!({ "folder": "", "title": " " }))
        .unwrap_err();
    assert_eq!(err["key"], "NOTE_TITLE_EMPTY");

    h.ok(
        "item_delete",
        json!({ "item": { "kind": "folder", "path": "archive" } }),
    );
    assert!(!h.vault.path().join("archive").exists());
    assert_eq!(h.ok("notes_tree", json!({}))["notes"], json!([]));
}

#[test]
fn settings_and_restore() {
    let h = Harness::new();
    h.ok(
        "settings_set",
        json!({ "key": "notes.wide", "value": true }),
    );
    assert_eq!(h.ok("settings_all", json!({}))["notes.wide"], json!(true));
    let restored = h.ok("vault_restore", json!({}));
    assert_eq!(
        restored,
        json!({ "path": h.vault.path().to_string_lossy(), "error": null })
    );
}

#[test]
fn tasks_round_trip_through_ipc() {
    let h = Harness::new();
    let parent = h.ok(
        "task_create",
        json!({ "task": { "path": "work", "title": "Ship", "orderKey": "a0" } }),
    );
    let pid = parent["id"].as_str().unwrap();
    let sub = h.ok(
        "task_create",
        json!({ "task": { "path": "", "title": "Sign", "parentId": pid, "orderKey": "a1" } }),
    );
    assert_eq!(sub["path"], "work");

    let t = h.ok("task_update", json!({ "id": pid, "patch": { "dueDate": "2026-10-10", "dueTime": "17:30", "status": "DONE" } }));
    assert_eq!(
        (t["dueDate"].as_str(), t["status"].as_str()),
        (Some("2026-10-10"), Some("DONE"))
    );
    assert!(t["completedAt"].is_number());
    let t = h.ok(
        "task_update",
        json!({ "id": pid, "patch": { "dueTime": null } }),
    );
    assert!(t["dueTime"].is_null() && t["dueDate"] == "2026-10-10");

    h.ok(
        "my_day_set",
        json!({ "id": pid, "day": "2026-10-04", "included": true }),
    );
    let data = h.ok("tasks_data", json!({ "day": "2026-10-04" }));
    assert_eq!(data["myDay"], json!([[pid, true]]));
    assert_eq!(data["folders"], json!(["work"]));

    h.ok("task_tags_set", json!({ "id": pid, "tags": ["v0.1"] }));
    h.ok("comment_add", json!({ "task": pid, "body": "looks good" }));
    let detail = h.ok("task_get", json!({ "id": pid }));
    assert_eq!(detail["tags"], json!(["v0_1"]));
    assert_eq!(
        h.ok("comments_list", json!({ "task": pid }))[0]["body"],
        "looks good"
    );

    let file = h.vault.path().join("report.txt");
    std::fs::write(&file, "x").unwrap();
    h.ok(
        "attachments_add",
        json!({ "owner": "TASK", "id": pid, "paths": [file] }),
    );
    let links = h.ok("attachments_list", json!({ "owner": "TASK", "id": pid }));
    assert_eq!(links[0]["name"], "report.txt");
    assert_eq!(
        std::fs::read_dir(h.vault.path().join(".tend/attachments"))
            .unwrap()
            .count(),
        1
    );

    h.ok("task_move", json!({ "id": pid, "target": "home" }));
    assert_eq!(h.ok("task_get", json!({ "id": sub["id"] }))["path"], "home");
    h.ok(
        "task_folder_rename",
        json!({ "path": "home", "name": "house" }),
    );
    assert_eq!(h.ok("task_get", json!({ "id": pid }))["path"], "house");
    h.ok("task_delete", json!({ "id": pid }));
    assert_eq!(
        h.ok("tasks_data", json!({ "day": "x" }))["tasks"],
        json!([])
    );
}

#[test]
fn the_library_round_trips_through_ipc() {
    let h = Harness::new();
    let word = h.ok("word_create", json!({ "word": { "title": "Petrichor" } }));
    assert_eq!(
        (word["title"].clone(), word["kind"].clone()),
        (json!("petrichor"), json!("WORD"))
    );
    let id = word["id"].as_str().unwrap().to_string();
    h.ok("word_tags_set", json!({ "id": id, "tags": ["Sound"] }));
    let edited = h.ok(
        "word_update",
        json!({ "id": id, "patch": { "meaning": "rain on dry ground", "dictionary": null } }),
    );
    assert_eq!(edited["tags"], json!(["sound"]));
    assert_eq!(
        h.ok("words_list", json!({}))[0]["meaning"],
        "rain on dry ground"
    );
    assert_eq!(
        h.ok("word_kinds_list", json!({})).as_array().unwrap().len(),
        6
    );
    let taken = h
        .call("word_create", json!({ "word": { "title": "PETRICHOR" } }))
        .unwrap_err();
    assert_eq!(taken["key"], "WORD_TAKEN");

    let stack = h.ok("stack_create", json!({ "parent": null, "name": "Haiku" }));
    let snap = h.ok(
        "snap_create",
        json!({ "snap": { "text": "first rain", "stackId": stack["id"], "paper": "STICKY", "color": "#d4b23b" } }),
    );
    assert_eq!(snap["paper"], "STICKY");
    h.ok("stack_remove", json!({ "id": stack["id"] }));
    let board = h.ok("snaps_board", json!({}));
    assert_eq!(board["snaps"][0]["stackId"], Value::Null);
    assert_eq!(board["stacks"], json!([]));
    let bad = h.call(
        "snap_create",
        json!({ "snap": { "text": "x", "paper": "FOLDED" } }),
    );
    assert!(bad.is_err(), "an unknown paper is refused");

    let link = json!({ "reference": { "kind": "LINK", "value": "https://svelte.dev/" } });
    let r = h.ok("ref_create", link.clone());
    assert_eq!(r["urlKey"], "svelte.dev");
    assert_eq!(
        h.call("ref_create", link).unwrap_err()["key"],
        "REF_LINK_TAKEN"
    );
    h.ok("ref_delete", json!({ "id": r["id"] }));
    assert_eq!(h.ok("refs_list", json!({})), json!([]));
}

#[test]
fn people_round_trip_through_ipc() {
    let h = Harness::new();
    let id = h.ok("person_create", json!({ "first": "Ravi", "last": "Kumar" }));
    let p = h.ok("person_update", json!({ "id": id, "patch": { "dob": { "year": null, "month": 10, "day": 9 }, "tob": "06:30" } }));
    assert_eq!(
        (
            p["dobMonth"].as_i64(),
            p["dobYear"].is_null(),
            p["tob"].as_str()
        ),
        (Some(10), true, Some("06:30"))
    );
    let err = h
        .call(
            "person_update",
            json!({ "id": id, "patch": { "dob": { "year": 2023, "month": 2, "day": 29 } } }),
        )
        .unwrap_err();
    assert_eq!(err["key"], "DOB_INVALID");
    h.ok(
        "contact_add",
        json!({ "person": id, "kind": "email", "value": "r@k.in", "label": "work" }),
    );
    assert_eq!(h.ok("people_list", json!({}))[0]["email"], "r@k.in");
    h.ok("person_delete", json!({ "id": id }));
    assert_eq!(h.ok("people_list", json!({})), json!([]));
}

#[test]
fn timers_and_feedback_through_ipc() {
    let h = Harness::new();
    let t = h.ok("timer_start", json!({ "title": "Tea", "ms": 10_000 }));
    assert_eq!(h.ok("timers_list", json!({}))[0]["title"], "Tea");
    h.ok("timer_add_minute", json!({ "id": t["id"] }));
    let ends = h.ok("timers_list", json!({}))[0]["endsAt"]
        .as_i64()
        .unwrap();
    assert_eq!(ends, t["endsAt"].as_i64().unwrap() + 60_000);
    h.ok("timer_ring", json!({ "id": t["id"] }));
    assert_eq!(h.ok("timers_list", json!({})), json!([]));
    assert_eq!(h.ok("timers_finished_while_closed", json!({})), json!([]));

    h.ok(
        "feedback_send",
        json!({ "kind": "bug", "body": "it crashed" }),
    );
    let err = h
        .call("feedback_send", json!({ "kind": "bug", "body": " " }))
        .unwrap_err();
    assert_eq!(err["key"], "FEEDBACK_EMPTY");

    let s = h.ok(
        "status_add",
        json!({ "label": "Waiting", "value": "WAITING", "color": "#d57149" }),
    );
    assert_eq!(s["position"], 2);
    let err = h
        .call("status_delete", json!({ "value": "DONE" }))
        .unwrap_err();
    assert_eq!(err["key"], "STATUS_DEFAULT");
}

#[test]
fn sync_connects_restores_and_disconnects_through_ipc() {
    let server = tempfile::tempdir().unwrap();
    git2::Repository::init_bare(server.path()).unwrap();
    let url = server.path().to_string_lossy().into_owned();
    let setup = json!({ "url": url, "token": "secret" });

    let h = Harness::new();
    h.ok("note_create", json!({ "folder": "", "title": "Plan" }));
    let off = h.ok("sync_status", json!({}));
    assert_eq!(off["config"], Value::Null);
    h.ok("sync_connect", json!({ "setup": setup }));
    let on = h.ok("sync_status", json!({}));
    assert_eq!(on["config"]["branch"], "main");
    assert_eq!(on["hasToken"], true);
    assert!(on.get("token").is_none(), "the token never reaches the UI");
    assert_eq!(on["config"]["intervalMinutes"], 15);

    let too_long = h
        .call("sync_set_interval", json!({ "minutes": 1441 }))
        .unwrap_err();
    assert_eq!(too_long["key"], "SYNC_INTERVAL");
    h.ok("sync_set_interval", json!({ "minutes": 30 }));
    assert_eq!(
        h.ok("sync_status", json!({}))["config"]["intervalMinutes"],
        30
    );

    // Another vault can't connect to a repo that now holds one.
    let other = Harness::new();
    let refused = other
        .call("sync_connect", json!({ "setup": setup }))
        .unwrap_err();
    assert_eq!(refused["key"], "SYNC_REMOTE_NOT_EMPTY");

    let copy = tempfile::tempdir().unwrap();
    h.ok(
        "sync_restore",
        json!({ "path": copy.path(), "setup": setup }),
    );
    assert!(copy.path().join("plan.md").is_file());
    let restored = h.ok("sync_status", json!({}));
    assert_eq!(restored["config"]["url"], url);

    h.ok("sync_disconnect", json!({}));
    let off = h.ok("sync_status", json!({}));
    assert_eq!(off["config"], Value::Null);
    assert_eq!(off["hasToken"], false);
}

#[test]
fn restore_needs_an_empty_folder() {
    let h = Harness::new();
    let full = tempfile::tempdir().unwrap();
    std::fs::write(full.path().join("a.md"), "# A").unwrap();
    let setup = json!({ "url": "https://example.com/x.git", "token": "t" });
    let refused = h
        .call(
            "sync_restore",
            json!({ "path": full.path(), "setup": setup }),
        )
        .unwrap_err();
    assert_eq!(refused["key"], "RESTORE_NOT_EMPTY");
}
