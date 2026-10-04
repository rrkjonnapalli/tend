//! The open vault, shared by every command.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, RwLock};

use rusqlite::Connection;

use crate::error::{AppError, AppResult};

pub struct Vault {
    pub root: PathBuf,
    conn: Mutex<Connection>,
    /// Items the last scan skipped because their names break the rule.
    skipped: Mutex<Vec<String>>,
}

impl Vault {
    pub fn new(root: PathBuf, conn: Connection) -> Self {
        Vault { root, conn: Mutex::new(conn), skipped: Mutex::default() }
    }

    pub fn db(&self) -> MutexGuard<'_, Connection> {
        // A poisoned lock only means another command panicked; the connection is still usable.
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn skipped(&self) -> Vec<String> {
        self.skipped.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn set_skipped(&self, items: Vec<String>) {
        *self.skipped.lock().unwrap_or_else(|e| e.into_inner()) = items;
    }

    pub fn notes_dir(&self) -> PathBuf {
        self.root.join("notes")
    }
}

/// Anything that lives as long as the open vault (the file watcher) and must stop with it.
pub type Guard = Box<dyn Send + Sync>;

#[derive(Default)]
pub struct AppState {
    vault: RwLock<Option<Arc<Vault>>>,
    guard: Mutex<Option<Guard>>,
}

impl AppState {
    pub fn vault(&self) -> AppResult<Arc<Vault>> {
        self.vault
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
            .ok_or(AppError::NoVault)
    }

    /// Runs `f` with the open vault's database locked for the whole call.
    pub fn with<T>(&self, f: impl FnOnce(&mut Connection, &Vault) -> AppResult<T>) -> AppResult<T> {
        let vault = self.vault()?;
        let mut db = vault.db();
        f(&mut db, &vault)
    }

    pub fn set(&self, vault: Arc<Vault>) {
        // Drop the old watcher before the new vault takes over.
        *self.guard.lock().unwrap_or_else(|e| e.into_inner()) = None;
        *self.vault.write().unwrap_or_else(|e| e.into_inner()) = Some(vault);
    }

    pub fn set_guard(&self, guard: Guard) {
        *self.guard.lock().unwrap_or_else(|e| e.into_inner()) = Some(guard);
    }
}
