//! One error type for every command. It reaches the UI as `{ code, message }`.

use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("An incompatible database exists in db/app.sqlite")]
    IncompatibleDb,
    #[error("No vault is open")]
    NoVault,
    #[error("{0}")]
    InvalidName(String),
    #[error("{0} was not found")]
    NotFound(String),
    /// A domain rule was broken (status rules, moves into self, …). `code` is stable for the UI.
    #[error("{message}")]
    Rule { code: &'static str, message: String },
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Db(#[from] rusqlite::Error),
    #[error("{0}")]
    Watch(#[from] notify::Error),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
}

impl AppError {
    pub fn rule(code: &'static str, message: impl Into<String>) -> Self {
        AppError::Rule {
            code,
            message: message.into(),
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            AppError::IncompatibleDb => "incompatible_db",
            AppError::NoVault => "no_vault",
            AppError::InvalidName(_) => "invalid_name",
            AppError::NotFound(_) => "not_found",
            AppError::Rule { code, .. } => code,
            AppError::Io(_) => "io",
            AppError::Db(_) => "db",
            AppError::Watch(_) => "watch",
            AppError::Json(_) => "json",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("AppError", 2)?;
        st.serialize_field("code", self.code())?;
        st.serialize_field("message", &self.to_string())?;
        st.end()
    }
}

pub type AppResult<T> = Result<T, AppError>;
