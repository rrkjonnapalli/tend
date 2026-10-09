//! Sync: connecting, restoring, settings, the token and "Sync now" (spec §14–16). Network work runs
//! on its own database connection, so the app's commands never wait on it.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime, State};

use crate::config::{self, ConfigDir, SyncConfig};
use crate::db;
use crate::error::{errors, AppResult};
use crate::state::AppState;
use crate::sync::{run, runner, state as sync_state, token};
use crate::vault;

const DEFAULT_BRANCH: &str = "main";
const DEFAULT_AUTHOR: &str = "Tend";
const DEFAULT_EMAIL: &str = "tend@localhost";
const DEFAULT_INTERVAL_MINUTES: u32 = 15;
/// The longest wait between automatic syncs: a day.
const MAX_INTERVAL_MINUTES: u32 = 24 * 60;

/// What Settings → Sync and the dock icon show. Never the token.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    config: Option<SyncConfig>,
    has_token: bool,
    #[serde(flatten)]
    now: runner::Status,
    last_run_at: Option<i64>,
    last_result: Option<String>,
    last_ok_at: Option<i64>,
}

/// What the user types to connect or restore.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncSetup {
    url: String,
    branch: Option<String>,
    username: Option<String>,
    token: String,
}

fn config_of(setup: &SyncSetup) -> SyncConfig {
    let branch = setup
        .branch
        .as_deref()
        .map(str::trim)
        .filter(|b| !b.is_empty());
    let username = setup
        .username
        .as_deref()
        .map(str::trim)
        .filter(|u| !u.is_empty());
    SyncConfig {
        url: setup.url.trim().to_string(),
        branch: branch.unwrap_or(DEFAULT_BRANCH).to_string(),
        username: username.map(String::from),
        author_name: DEFAULT_AUTHOR.into(),
        author_email: DEFAULT_EMAIL.into(),
        interval_minutes: DEFAULT_INTERVAL_MINUTES,
    }
}

#[tauri::command(async)]
pub fn sync_status(state: State<AppState>, cfg: State<ConfigDir>) -> AppResult<SyncStatus> {
    let vault = state.vault()?;
    let config = config::sync_config(&cfg.0, &vault.root);
    let has_token = match &config {
        Some(c) => token::load(&c.url)?.is_some(),
        None => false,
    };
    let (last_run_at, last_result, last_ok_at) = vault.db().query_row(
        "SELECT last_run_at, last_result, last_ok_at FROM sync_state WHERE id = 1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    Ok(SyncStatus {
        config,
        has_token,
        now: state.with_runner(|r| r.status()).unwrap_or_default(),
        last_run_at,
        last_result,
        last_ok_at,
    })
}

/// Starts syncing the open vault to an empty remote.
#[tauri::command]
pub async fn sync_connect<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    cfg: State<'_, ConfigDir>,
    setup: SyncSetup,
) -> AppResult<()> {
    let vault = state.vault()?;
    let config = config_of(&setup);
    token::save(&config.url, &setup.token)?;
    let remote = runner::remote_of(&config)?;
    let (mut conn, _) = db::open(&vault::db_path(&vault.root))?;
    if let Err(e) = run::connect(&mut conn, &vault.root, &remote) {
        token::clear(&config.url)?;
        return Err(e);
    }
    config::set_sync_config(&cfg.0, &vault.root, Some(config))?;
    crate::start_sync(&app, &state, &vault);
    Ok(())
}

/// Makes `path` (an empty folder) a vault restored from the remote, and opens it.
#[tauri::command]
pub async fn sync_restore<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    cfg: State<'_, ConfigDir>,
    path: String,
    setup: SyncSetup,
) -> AppResult<vault::Check> {
    let root = PathBuf::from(&path);
    if vault::visible_entries(&root) > 0 {
        return Err(errors::RESTORE_NOT_EMPTY.err());
    }
    let (check, opened) = vault::open(&root, false);
    let Some(opened) = opened else {
        return Ok(check);
    };
    let config = config_of(&setup);
    token::save(&config.url, &setup.token)?;
    let remote = runner::remote_of(&config)?;
    if let Err(e) = run::restore(&mut opened.db(), &root, &remote) {
        token::clear(&config.url)?;
        return Err(e);
    }
    config::set_sync_config(&cfg.0, &root, Some(config))?;
    super::vault::activate(&app, &state, opened)?;
    config::remember_vault(&cfg.0, &root)?;
    Ok(check)
}

#[tauri::command(async)]
pub fn sync_now(state: State<AppState>) -> AppResult<()> {
    state
        .with_runner(|r| r.now())
        .ok_or_else(|| errors::SYNC_NOT_CONNECTED.err())
}

/// Minutes between automatic syncs; 0 turns them off.
#[tauri::command(async)]
pub fn sync_set_interval(
    state: State<AppState>,
    cfg: State<ConfigDir>,
    minutes: u32,
) -> AppResult<()> {
    if minutes > MAX_INTERVAL_MINUTES {
        return Err(errors::SYNC_INTERVAL.err());
    }
    let vault = state.vault()?;
    let Some(mut config) = config::sync_config(&cfg.0, &vault.root) else {
        return Err(errors::SYNC_NOT_CONNECTED.err());
    };
    config.interval_minutes = minutes;
    config::set_sync_config(&cfg.0, &vault.root, Some(config))?;
    state.with_runner(|r| r.wake());
    Ok(())
}

/// Replaces the token (after it expired, say) and syncs right away.
#[tauri::command(async)]
pub fn sync_token_set(
    state: State<AppState>,
    cfg: State<ConfigDir>,
    token: String,
) -> AppResult<()> {
    let vault = state.vault()?;
    let Some(config) = config::sync_config(&cfg.0, &vault.root) else {
        return Err(errors::SYNC_NOT_CONNECTED.err());
    };
    token::save(&config.url, &token)?;
    state.with_runner(|r| r.now());
    Ok(())
}

/// Stops syncing: the settings, token and base go; `.git` and all data stay.
#[tauri::command(async)]
pub fn sync_disconnect(state: State<AppState>, cfg: State<ConfigDir>) -> AppResult<()> {
    let vault = state.vault()?;
    state.set_runner(None);
    if let Some(config) = config::sync_config(&cfg.0, &vault.root) {
        token::clear(&config.url)?;
    }
    config::set_sync_config(&cfg.0, &vault.root, None)?;
    let conn = vault.db();
    sync_state::forget(&conn)
}
