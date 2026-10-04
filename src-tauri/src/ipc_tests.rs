//! Commands called the way the UI calls them: JSON arguments through the IPC layer.

use serde_json::{json, Value};
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, MockRuntime, INVOKE_KEY};
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
        let app = crate::with_commands(mock_builder()).build(mock_context(noop_assets())).unwrap();
        app.manage(ConfigDir(config.path().to_path_buf()));
        let webview = WebviewWindowBuilder::new(&app, "main", Default::default()).build().unwrap();
        let h = Harness { vault, _config: config, _app: app, webview };
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
        self.call(cmd, args).unwrap_or_else(|e| panic!("{cmd} failed: {e}"))
    }
}

#[test]
fn notes_round_trip_through_ipc() {
    let h = Harness::new();
    h.ok("folder_create", json!({ "path": "work" }));
    let note = h.ok("note_create", json!({ "folder": "work", "name": "plan" }));
    let id = note["id"].as_str().unwrap().to_string();
    assert_eq!(note["title"], "Plan");

    h.ok("note_write", json!({ "id": id, "text": "# Q4 plan\n\nship" }));
    let doc = h.ok("note_read", json!({ "id": id }));
    assert_eq!(doc["text"], "# Q4 plan\n\nship");
    assert_eq!(doc["modifiedAt"].is_number(), true);

    let tags = h.ok("note_tags_set", json!({ "id": id, "tags": ["ops", "Q4"] }));
    assert_eq!(tags, json!(["ops", "Q4"]));

    h.ok("item_rename", json!({ "item": { "kind": "note", "id": id }, "name": "q4_plan" }));
    h.ok("item_move", json!({ "item": { "kind": "folder", "path": "work" }, "target": "archive" }));
    assert!(h.vault.path().join("notes/archive/work/q4_plan.md").is_file());

    let tree = h.ok("notes_tree", json!({}));
    assert_eq!(tree["folders"], json!(["archive", "archive/work"]));
    assert_eq!(tree["notes"][0]["folder"], "archive/work");

    let err = h.call("note_create", json!({ "folder": "", "name": "Bad Name" })).unwrap_err();
    assert_eq!(err["code"], "invalid_name");

    h.ok("item_delete", json!({ "item": { "kind": "folder", "path": "archive" } }));
    assert!(!h.vault.path().join("notes/archive").exists());
    assert_eq!(h.ok("notes_tree", json!({}))["notes"], json!([]));
}

#[test]
fn settings_and_restore() {
    let h = Harness::new();
    h.ok("settings_set", json!({ "key": "notes.wide", "value": true }));
    assert_eq!(h.ok("settings_all", json!({}))["notes.wide"], json!(true));
    let restored = h.ok("vault_restore", json!({}));
    assert_eq!(restored, json!(h.vault.path().to_string_lossy()));
}

#[test]
fn tasks_round_trip_through_ipc() {
    let h = Harness::new();
    let parent = h.ok("task_create", json!({ "task": { "path": "work", "title": "Ship" } }));
    let pid = parent["id"].as_str().unwrap();
    let sub = h.ok("task_create", json!({ "task": { "path": "", "title": "Sign", "parentId": pid, "orderKey": "z" } }));
    assert_eq!(sub["path"], "work");

    let t = h.ok("task_update", json!({ "id": pid, "patch": { "dueDate": "2026-10-10", "dueTime": "17:30", "status": "DONE" } }));
    assert_eq!((t["dueDate"].as_str(), t["status"].as_str()), (Some("2026-10-10"), Some("DONE")));
    assert!(t["completedAt"].is_number());
    let t = h.ok("task_update", json!({ "id": pid, "patch": { "dueTime": null } }));
    assert!(t["dueTime"].is_null() && t["dueDate"] == "2026-10-10");

    h.ok("my_day_set", json!({ "id": pid, "day": "2026-10-04", "included": true }));
    let data = h.ok("tasks_data", json!({ "day": "2026-10-04" }));
    assert_eq!(data["myDay"], json!([[pid, true]]));
    assert_eq!(data["folders"], json!(["work"]));

    h.ok("task_tags_set", json!({ "id": pid, "tags": ["v0.1"] }));
    h.ok("comment_add", json!({ "task": pid, "body": "looks good" }));
    let detail = h.ok("task_get", json!({ "id": pid }));
    assert_eq!(detail["tags"], json!(["v0.1"]));
    assert_eq!(h.ok("comments_list", json!({ "task": pid }))[0]["body"], "looks good");

    let file = h.vault.path().join("report.txt");
    std::fs::write(&file, "x").unwrap();
    h.ok("attachments_add", json!({ "owner": "task", "id": pid, "paths": [file] }));
    let links = h.ok("attachments_list", json!({ "owner": "task", "id": pid }));
    assert_eq!(links[0]["name"], "report.txt");
    assert_eq!(std::fs::read_dir(h.vault.path().join(".attachments")).unwrap().count(), 1);

    h.ok("task_move", json!({ "id": pid, "target": "home" }));
    assert_eq!(h.ok("task_get", json!({ "id": sub["id"] }))["path"], "home");
    h.ok("task_folder_rename", json!({ "path": "home", "name": "house" }));
    assert_eq!(h.ok("task_get", json!({ "id": pid }))["path"], "house");
    h.ok("task_delete", json!({ "id": pid }));
    assert_eq!(h.ok("tasks_data", json!({ "day": "x" }))["tasks"], json!([]));
}
