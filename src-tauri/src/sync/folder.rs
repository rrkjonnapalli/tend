//! Where a phone keeps a vault: the user never browses Tend's own storage, so each remote gets a
//! folder named after its path, `vaults/<host>/<owner>/<repo>` (docs/mobile/tend-mobile-spec.md §10).

use std::path::{Path, PathBuf};

use crate::error::{errors, AppResult};

/// The folder under `data_dir` for the remote at `url`, such as
/// `https://github.com/Rniverse/Notes.git` → `vaults/github.com/rniverse/notes`. Lowercased, since
/// hosts treat these names without case and Android's file system doesn't.
pub fn phone_vault(data_dir: &Path, url: &str) -> AppResult<PathBuf> {
    let [host, owner, repo] = parts(url).ok_or_else(|| errors::SYNC_URL.err())?;
    Ok(data_dir.join("vaults").join(host).join(owner).join(repo))
}

/// Host, owner and repo of an `https://` (or `http://`) remote; anything else is refused.
fn parts(url: &str) -> Option<[String; 3]> {
    let rest = url.trim().strip_prefix("https://").or_else(|| url.trim().strip_prefix("http://"))?;
    let rest = rest.trim_end_matches('/');
    let rest = rest.strip_suffix(".git").unwrap_or(rest);
    let mut segments = rest.split('/');
    let host = segments.next()?;
    let host = host.rsplit('@').next()?; // drop a `user@` before the host
    let owner = segments.next()?;
    let repo = segments.next()?;
    let extra = segments.next().is_some();
    let names = [host, owner, repo];
    let safe = names.iter().all(|n| is_safe(n));
    (safe && !extra).then(|| names.map(str::to_lowercase))
}

/// One path segment that can't climb out of `vaults/` or name nothing.
fn is_safe(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains('\\')
}

#[cfg(test)]
mod tests;
