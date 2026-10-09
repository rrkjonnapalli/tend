//! The sync runner (spec §15): one background thread per open, connected vault. It syncs when the
//! vault opens (after the scan), every `interval_minutes`, on "Sync now", and once more when the
//! app quits. It has its own database connection, so a slow network never blocks the app's
//! commands. Failures retry with backoff up to an hour; a refused token, a lost base or a newer
//! vault stop automatic runs until "Sync now".

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime};

use super::git::Remote;
use super::run::{self, Outcome};
use super::token;
use crate::config::{self, SyncConfig};
use crate::db;
use crate::error::{errors, AppError, AppResult};
use crate::util::now_ms;
use crate::vault;

/// The UI listens for these.
pub const STATUS_EVENT: &str = "sync:status";
pub const APPLIED_EVENT: &str = "sync:applied";

const FIRST_RETRY: Duration = Duration::from_secs(60);
const MAX_RETRY: Duration = Duration::from_secs(60 * 60);
const SECS_PER_MINUTE: u64 = 60;

enum Trigger {
    /// "Sync now".
    Now,
    /// The settings changed: work out the next wait again.
    Wake,
    /// The app is quitting: sync once more, then tell the sender.
    Close(Sender<()>),
    Stop,
}

/// What a run tells the UI.
#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub running: bool,
    /// Why the last run failed; `None` after a success or before the first run.
    pub error: Option<AppError>,
    /// When that run failed (ms). A run that fails before it starts isn't in `sync_state`.
    pub failed_at: Option<i64>,
}

/// Where the runner works: the vault, and the folder holding its sync settings.
#[derive(Clone)]
pub struct Place {
    pub root: PathBuf,
    pub config_dir: PathBuf,
}

/// The runner's status, shared with its thread.
type Shared = Arc<Mutex<Status>>;

pub struct Runner {
    tx: Sender<Trigger>,
    status: Shared,
}

impl Runner {
    pub fn start<R: Runtime>(app: AppHandle<R>, place: Place) -> Runner {
        let (tx, rx) = mpsc::channel();
        let status = Shared::default();
        let shared = status.clone();
        std::thread::spawn(move || work(&app, &place, &rx, &shared));
        Runner { tx, status }
    }

    pub fn status(&self) -> Status {
        lock(&self.status).clone()
    }

    pub fn now(&self) {
        let _ = self.tx.send(Trigger::Now);
    }

    pub fn wake(&self) {
        let _ = self.tx.send(Trigger::Wake);
    }

    pub fn stop(&self) {
        let _ = self.tx.send(Trigger::Stop);
    }

    /// Runs a last sync and waits for it, at most `limit`. A cut-short run counts as not done; the
    /// next open syncs again.
    pub fn close(&self, limit: Duration) {
        let (done, wait) = mpsc::channel();
        if self.tx.send(Trigger::Close(done)).is_ok() {
            let _ = wait.recv_timeout(limit);
        }
    }
}

/// How automatic runs stand after the last one.
#[derive(Default)]
struct Backoff {
    failures: u32,
    /// Waiting for the user (bad token, lost base, newer vault): no automatic runs.
    paused: bool,
}

impl Backoff {
    fn after(&mut self, result: &AppResult<Outcome>) {
        match result {
            Ok(_) => *self = Backoff::default(),
            Err(e) if needs_user(e) => self.paused = true,
            Err(_) => self.failures += 1,
        }
    }

    /// How long to wait for the next automatic run; `None` waits for a trigger only.
    fn wait(&self, interval_minutes: u32) -> Option<Duration> {
        if self.paused {
            return None;
        }
        if self.failures > 0 {
            let doubled = FIRST_RETRY.saturating_mul(1 << (self.failures - 1).min(10));
            return Some(doubled.min(MAX_RETRY));
        }
        if interval_minutes == 0 {
            return None;
        }
        Some(Duration::from_secs(
            u64::from(interval_minutes) * SECS_PER_MINUTE,
        ))
    }
}

fn needs_user(e: &AppError) -> bool {
    let keys = [
        errors::SYNC_AUTH,
        errors::SYNC_TOKEN,
        errors::SYNC_BASE_LOST,
        errors::SYNC_NOT_A_VAULT,
        errors::SNAPSHOT_TOO_NEW,
        errors::SYNC_NOT_CONNECTED,
    ];
    keys.iter().any(|def| e.is(*def))
}

fn work<R: Runtime>(app: &AppHandle<R>, place: &Place, rx: &Receiver<Trigger>, status: &Shared) {
    let mut backoff = Backoff::default();
    backoff.after(&run_reported(app, place, status));
    loop {
        let interval =
            config::sync_config(&place.config_dir, &place.root).map_or(0, |c| c.interval_minutes);
        let next = match backoff.wait(interval) {
            Some(wait) => rx.recv_timeout(wait),
            None => rx.recv().map_err(|_| RecvTimeoutError::Disconnected),
        };
        match next {
            Ok(Trigger::Stop) | Err(RecvTimeoutError::Disconnected) => break,
            Ok(Trigger::Wake) => continue,
            Ok(Trigger::Close(done)) => {
                let _ = run_reported(app, place, status);
                let _ = done.send(());
                break;
            }
            Ok(Trigger::Now) => {
                backoff = Backoff::default();
                backoff.after(&run_reported(app, place, status));
            }
            Err(RecvTimeoutError::Timeout) => backoff.after(&run_reported(app, place, status)),
        }
    }
}

/// One run, with the UI told when it starts and ends, and to reload when rows changed.
fn run_reported<R: Runtime>(
    app: &AppHandle<R>,
    place: &Place,
    status: &Shared,
) -> AppResult<Outcome> {
    report(app, status, |s| s.running = true);
    let result = run_once(place);
    report(app, status, |s| {
        s.running = false;
        s.error = result.as_ref().err().cloned();
        s.failed_at = s.error.as_ref().map(|_| now_ms());
    });
    let changed = matches!(result, Ok(Outcome::Pulled | Outcome::Pushed));
    if changed {
        let _ = app.emit(APPLIED_EVENT, ());
    }
    result
}

/// Updates the shared status and sends it to the UI.
fn report<R: Runtime>(app: &AppHandle<R>, status: &Shared, change: impl FnOnce(&mut Status)) {
    let now = {
        let mut s = lock(status);
        change(&mut s);
        s.clone()
    };
    let _ = app.emit(STATUS_EVENT, now);
}

fn lock(status: &Shared) -> std::sync::MutexGuard<'_, Status> {
    // A poisoned lock only means the thread panicked mid-update; the status is still readable.
    status.lock().unwrap_or_else(|e| e.into_inner())
}

fn run_once(place: &Place) -> AppResult<Outcome> {
    let Some(cfg) = config::sync_config(&place.config_dir, &place.root) else {
        return Err(errors::SYNC_NOT_CONNECTED.err());
    };
    let remote = remote_of(&cfg)?;
    let (mut conn, _) = db::open(&vault::db_path(&place.root))?;
    run::sync(&mut conn, &place.root, &remote)
}

/// The Git remote for saved settings, with the token from the Keychain.
pub fn remote_of(cfg: &SyncConfig) -> AppResult<Remote> {
    Ok(Remote {
        url: cfg.url.clone(),
        branch: cfg.branch.clone(),
        username: cfg.username.clone(),
        token: token::load(&cfg.url)?,
        author_name: cfg.author_name.clone(),
        author_email: cfg.author_email.clone(),
    })
}

#[cfg(test)]
mod tests;
