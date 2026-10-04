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
