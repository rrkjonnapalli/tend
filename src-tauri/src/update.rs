//! App updates (desktop; phones update from their stores). The feed is the latest GitHub release's
//! `latest.json` (`tauri.conf.json` → `plugins.updater`), and an update installs only when it is
//! signed with Tend's key, whose public half is in that config.

use serde::Serialize;
use tauri::{AppHandle, Runtime};
use tauri_plugin_updater::UpdaterExt;

use crate::error::{errors, AppError, AppResult};

impl From<tauri_plugin_updater::Error> for AppError {
    fn from(e: tauri_plugin_updater::Error) -> Self {
        errors::UPDATE_FAILED.with(format!("{} ({e})", errors::UPDATE_FAILED.message))
    }
}

/// A newer release than the running app.
#[derive(Serialize)]
pub struct Available {
    pub version: String,
    /// The release notes, as written on the release.
    pub notes: Option<String>,
}

pub async fn check<R: Runtime>(app: &AppHandle<R>) -> AppResult<Option<Available>> {
    let found = app.updater()?.check().await?;
    Ok(found.map(|u| Available {
        version: u.version,
        notes: u.body,
    }))
}

/// Downloads and installs the newest release over the running app; `false` when there is none.
/// The app keeps running the old version until it restarts.
pub async fn install<R: Runtime>(app: &AppHandle<R>) -> AppResult<bool> {
    let Some(update) = app.updater()?.check().await? else {
        return Ok(false);
    };
    update.download_and_install(|_, _| {}, || {}).await?;
    Ok(true)
}
