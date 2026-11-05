//! The sync token, kept only in the system's credential store (the macOS Keychain). It is never
//! written to the vault, the database or a config file, and is read only for a Git call.

use std::sync::Once;

use keyring_core::{Entry, Error};

use crate::error::{errors, AppError, AppResult};

/// The Keychain service; the account is the remote URL, so each remote has its own token.
const SERVICE: &str = "app.tend.sync";

impl From<Error> for AppError {
    fn from(e: Error) -> Self {
        errors::SYNC_TOKEN.with(e.to_string())
    }
}

/// Picks the platform's store once per process. Tests use an in-memory one.
fn init() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        if let Some(store) = platform_store() {
            keyring_core::set_default_store(store);
        }
    });
}

#[cfg(test)]
fn platform_store() -> Option<std::sync::Arc<keyring_core::CredentialStore>> {
    keyring_core::mock::Store::new().ok().map(|s| s as _)
}

#[cfg(all(not(test), target_os = "macos"))]
fn platform_store() -> Option<std::sync::Arc<keyring_core::CredentialStore>> {
    apple_native_keyring_store::keychain::Store::new()
        .ok()
        .map(|s| s as _)
}

#[cfg(all(not(test), target_os = "ios"))]
fn platform_store() -> Option<std::sync::Arc<keyring_core::CredentialStore>> {
    apple_native_keyring_store::protected::Store::new()
        .ok()
        .map(|s| s as _)
}

#[cfg(all(not(test), target_os = "android"))]
fn platform_store() -> Option<std::sync::Arc<keyring_core::CredentialStore>> {
    android_native_keyring_store::Store::new().ok().map(|s| s as _)
}

/// No store on this platform yet: every call reports `SYNC_TOKEN`.
#[cfg(all(
    not(test),
    not(any(target_os = "macos", target_os = "ios", target_os = "android"))
))]
fn platform_store() -> Option<std::sync::Arc<keyring_core::CredentialStore>> {
    None
}

fn entry(url: &str) -> AppResult<Entry> {
    init();
    Ok(Entry::new(SERVICE, url)?)
}

pub fn save(url: &str, token: &str) -> AppResult<()> {
    entry(url)?.set_password(token)?;
    Ok(())
}

/// The token for `url`, or `None` when none is saved.
pub fn load(url: &str) -> AppResult<Option<String>> {
    match entry(url)?.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn clear(url: &str) -> AppResult<()> {
    match entry(url)?.delete_credential() {
        Ok(()) | Err(Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests;
