//! App preferences outside any vault: only the last vault path, in the app config dir.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::AppResult;

/// Where app-level preferences live (the platform app config dir; a temp dir in tests).
pub struct ConfigDir(pub PathBuf);

#[derive(Default, Serialize, Deserialize)]
struct Config {
    vault: Option<PathBuf>,
}

fn file(dir: &Path) -> PathBuf {
    dir.join("config.json")
}

pub fn last_vault(dir: &Path) -> Option<PathBuf> {
    let text = fs::read_to_string(file(dir)).ok()?;
    serde_json::from_str::<Config>(&text).ok()?.vault
}

pub fn remember_vault(dir: &Path, vault: &Path) -> AppResult<()> {
    fs::create_dir_all(dir)?;
    let cfg = Config {
        vault: Some(vault.to_path_buf()),
    };
    fs::write(file(dir), serde_json::to_string_pretty(&cfg)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_the_last_vault() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(last_vault(dir.path()), None);
        remember_vault(dir.path(), Path::new("/tmp/v")).unwrap();
        assert_eq!(last_vault(dir.path()), Some(PathBuf::from("/tmp/v")));
    }
}
