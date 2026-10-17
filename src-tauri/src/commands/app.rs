//! App settings and feedback.

use std::collections::HashMap;

use serde_json::Value;
use tauri::{AppHandle, Runtime, State};

use crate::error::AppResult;
use crate::state::AppState;
use crate::{feedback, settings};

/// A newer release, if there is one.
#[cfg(desktop)]
#[tauri::command]
pub async fn app_update_check<R: Runtime>(
    app: AppHandle<R>,
) -> AppResult<Option<crate::update::Available>> {
    crate::update::check(&app).await
}

/// Installs the newest release, then quits as usual and starts again as the new version. `false`
/// when there was nothing newer to install.
#[cfg(desktop)]
#[tauri::command]
pub async fn app_update_install<R: Runtime>(app: AppHandle<R>) -> AppResult<bool> {
    let installed = crate::update::install(&app).await?;
    if installed {
        crate::restart(&app);
    }
    Ok(installed)
}

/// The UI has saved everything it held after `QUIT_EVENT`; quitting can go on.
#[tauri::command]
pub fn app_flushed(state: State<AppState>) {
    state.set_flushed();
}

#[tauri::command(async)]
pub fn settings_all(state: State<AppState>) -> AppResult<HashMap<String, Value>> {
    state.with(|db, _| settings::all(db))
}

#[tauri::command(async)]
pub fn settings_set(state: State<AppState>, key: String, value: Value) -> AppResult<()> {
    state.with(|db, _| settings::set(db, &key, &value))
}

#[tauri::command(async)]
pub fn feedback_send<R: Runtime>(
    app: AppHandle<R>,
    state: State<AppState>,
    kind: String,
    body: String,
) -> AppResult<()> {
    let version = app.package_info().version.to_string();
    state.with(|db, _| feedback::add(db, &kind, &body, &version))
}
