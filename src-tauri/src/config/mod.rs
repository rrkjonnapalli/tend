//! App preferences outside any vault, in the app config dir: the last vault path, and how each
//! vault syncs.

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

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

fn load(dir: &Path) -> Config {
    let Ok(text) = fs::read_to_string(file(dir)) else {
        return Config::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

fn save(dir: &Path, cfg: &Config) -> AppResult<()> {
    fs::create_dir_all(dir)?;
    fs::write(file(dir), serde_json::to_string_pretty(cfg)?)?;
    Ok(())
}

pub fn last_vault(dir: &Path) -> Option<PathBuf> {
    load(dir).vault
}

pub fn remember_vault(dir: &Path, vault: &Path) -> AppResult<()> {
    let mut cfg = load(dir);
    cfg.vault = Some(vault.to_path_buf());
    save(dir, &cfg)
}

fn key(vault: &Path) -> String {
    vault.to_string_lossy().into_owned()
}

pub fn sync_config(dir: &Path, vault: &Path) -> Option<SyncConfig> {
    load(dir).sync.get(&key(vault)).cloned()
}

/// Saves or (with `None`) forgets how `vault` syncs.
pub fn set_sync_config(dir: &Path, vault: &Path, sync: Option<SyncConfig>) -> AppResult<()> {
    let mut cfg = load(dir);
    match sync {
        Some(sync) => cfg.sync.insert(key(vault), sync),
        None => cfg.sync.remove(&key(vault)),
    };
    save(dir, &cfg)
}
