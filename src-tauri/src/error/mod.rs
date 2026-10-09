//! One error type for every command. Every error comes from the registry (`errors::*`) and
//! reaches the UI as `{ key, message }`; the UI shows the message and can match on the key.

pub mod registry;

#[cfg(test)]
mod tests;

use std::fmt;
use std::io::ErrorKind;

use serde::{Serialize, Serializer};

pub use registry as errors;

/// A registry entry: a stable key and the message shown when the throw site adds nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorDef {
    pub key: &'static str,
    pub message: &'static str,
}

impl ErrorDef {
    /// This error with its default message.
    pub fn err(self) -> AppError {
        AppError {
            key: self.key,
            message: self.message.to_string(),
        }
    }

    /// This error with a message written for this case (`NOT_FOUND.with("Note was not found")`).
    pub fn with(self, message: impl Into<String>) -> AppError {
        AppError {
            key: self.key,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppError {
    key: &'static str,
    message: String,
}

impl AppError {
    /// True when this error came from `def`.
    pub fn is(&self, def: ErrorDef) -> bool {
        self.key == def.key
    }

    /// The registry key (stored as a sync run's result).
    pub fn key(&self) -> &'static str {
        self.key
    }
}

pub type AppResult<T> = Result<T, AppError>;

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for AppError {}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("AppError", 2)?;
        st.serialize_field("key", self.key)?;
        st.serialize_field("message", &self.message)?;
        st.end()
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        if e.kind() == ErrorKind::PermissionDenied {
            return errors::NO_PERMISSION.err();
        }
        errors::IO.with(e.to_string())
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        errors::DB.with(e.to_string())
    }
}

impl From<notify::Error> for AppError {
    fn from(e: notify::Error) -> Self {
        errors::WATCH.with(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        errors::JSON.with(e.to_string())
    }
}
