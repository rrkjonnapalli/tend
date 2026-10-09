//! The open vault, shared by every command.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, RwLock};

use rusqlite::Connection;

use crate::error::{errors, AppResult};
use crate::notes::Skipped;
use crate::sync::runner::Runner;

pub struct Vault {
    pub root: PathBuf,
    conn: Mutex<Connection>,
    /// Items the last scan skipped because their names break the rule.
    skipped: Mutex<Vec<Skipped>>,
}

impl Vault {
    pub fn new(root: PathBuf, conn: Connection) -> Self {
        Vault {
            root,
            conn: Mutex::new(conn),
            skipped: Mutex::default(),
        }
    }

    pub fn db(&self) -> MutexGuard<'_, Connection> {
        // A poisoned lock only means another command panicked; the connection is still usable.
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn skipped(&self) -> Vec<Skipped> {
        self.skipped
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    pub fn set_skipped(&self, items: Vec<Skipped>) {
        *self.skipped.lock().unwrap_or_else(|e| e.into_inner()) = items;
    }

    /// The notes root, which is the vault folder itself.
    pub fn notes_dir(&self) -> PathBuf {
        self.root.clone()
    }
}

/// Anything that lives as long as the open vault (the file watcher) and must stop with it.
pub type Guard = Box<dyn Send + Sync>;

#[derive(Default)]
pub struct AppState {
    vault: RwLock<Option<Arc<Vault>>>,
    guard: Mutex<Option<Guard>>,
    /// The open vault's sync runner, when it is connected.
    runner: Mutex<Option<Runner>>,
    /// Set once the app has started quitting, so the last sync runs only once.
    closing: AtomicBool,
}

impl AppState {
    pub fn vault(&self) -> AppResult<Arc<Vault>> {
        self.vault
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
            .ok_or(errors::NO_VAULT.err())
    }

    /// Runs `f` with the open vault's database locked for the whole call.
    pub fn with<T>(&self, f: impl FnOnce(&mut Connection, &Vault) -> AppResult<T>) -> AppResult<T> {
        let vault = self.vault()?;
        let mut db = vault.db();
        f(&mut db, &vault)
    }

    pub fn set(&self, vault: Arc<Vault>) {
        // Stop the old vault's watcher and sync runner before the new vault takes over.
        *self.guard.lock().unwrap_or_else(|e| e.into_inner()) = None;
        self.set_runner(None);
        *self.vault.write().unwrap_or_else(|e| e.into_inner()) = Some(vault);
    }

    pub fn set_guard(&self, guard: Guard) {
        *self.guard.lock().unwrap_or_else(|e| e.into_inner()) = Some(guard);
    }

    /// Replaces the sync runner; the old one stops.
    pub fn set_runner(&self, runner: Option<Runner>) {
        let mut slot = self.runner.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(old) = slot.take() {
            old.stop();
        }
        *slot = runner;
    }

    /// Runs `f` with the sync runner, when the open vault has one.
    pub fn with_runner<T>(&self, f: impl FnOnce(&Runner) -> T) -> Option<T> {
        let slot = self.runner.lock().unwrap_or_else(|e| e.into_inner());
        slot.as_ref().map(f)
    }

    /// True the first time only: the app is quitting now.
    pub fn start_closing(&self) -> bool {
        !self.closing.swap(true, Ordering::SeqCst)
    }
}
