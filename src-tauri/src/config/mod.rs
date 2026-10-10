//! App preferences outside any vault, in the app config dir: the last vault path, and how each
//! vault syncs.

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use serde::{Deserialize, Serialize};

use crate::error::AppResult;

/// Where app-level preferences live (the platform app config dir; a temp dir in tests).
pub struct ConfigDir(pub PathBuf);

#[derive(Default, Serialize, Deserialize)]
struct Config {
    vault: Option<PathBuf>,
    /// Sync settings per vault path. Kept here, never in the vault; the token is in the Keychain.
    #[serde(default)]
    sync: BTreeMap<String, SyncConfig>,
}

/// Where and how one vault syncs (spec §14). No token: that lives in the Keychain only.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncConfig {
    pub url: String,
    pub branch: String,
    pub username: Option<String>,
    pub author_name: String,
    pub author_email: String,
    /// Minutes between automatic syncs; 0 is off.
    pub interval_minutes: u32,
}

fn file(dir: &Path) -> PathBuf {
    dir.join("config.json")
}

/// Every change is read-modify-write, and the sync runner reads while commands write; one lock
/// keeps a change from losing another's.
fn lock() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

/// Applies `change` to the saved config as one step. A config that exists but doesn't parse is
/// left alone (an error), and so is one that can't be read, so neither a bad byte nor a permission
/// problem can make a save wipe every vault's sync settings.
fn update(dir: &Path, change: impl FnOnce(&mut Config)) -> AppResult<()> {
    let _held = lock();
    let mut cfg = match fs::read_to_string(file(dir)) {
        Ok(text) => serde_json::from_str(&text)?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Config::default(),
        Err(e) => return Err(e.into()),
    };
    change(&mut cfg);
    save(dir, &cfg)
}

fn load(dir: &Path) -> Config {
    let Ok(text) = fs::read_to_string(file(dir)) else {
        return Config::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

/// Writes a temp file and renames it over the config, so a reader never sees half a file.
fn save(dir: &Path, cfg: &Config) -> AppResult<()> {
    fs::create_dir_all(dir)?;
    let temp = dir.join("config.json.tmp");
    fs::write(&temp, serde_json::to_string_pretty(cfg)?)?;
    fs::rename(&temp, file(dir))?;
    Ok(())
}

pub fn last_vault(dir: &Path) -> Option<PathBuf> {
    load(dir).vault
}

pub fn remember_vault(dir: &Path, vault: &Path) -> AppResult<()> {
    update(dir, |cfg| cfg.vault = Some(vault.to_path_buf()))
}

fn key(vault: &Path) -> String {
    vault.to_string_lossy().into_owned()
}

pub fn sync_config(dir: &Path, vault: &Path) -> Option<SyncConfig> {
    load(dir).sync.get(&key(vault)).cloned()
}

/// Saves or (with `None`) forgets how `vault` syncs.
pub fn set_sync_config(dir: &Path, vault: &Path, sync: Option<SyncConfig>) -> AppResult<()> {
    update(dir, |cfg| {
        match sync {
            Some(sync) => cfg.sync.insert(key(vault), sync),
            None => cfg.sync.remove(&key(vault)),
        };
    })
}
