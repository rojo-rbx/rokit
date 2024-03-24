use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AftmanError {
    #[error("home directory not found")]
    HomeNotFound,
    #[error("file not found: {0}")]
    FileNotFound(PathBuf),
    #[error("task join error: {0}")]
    TaskJoinError(#[from] tokio::task::JoinError),
    #[error("TOML parse error: {0}")]
    TomlParseError(#[from] toml_edit::TomlError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type AftmanResult<T> = Result<T, AftmanError>;
