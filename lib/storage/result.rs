use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("home directory not found")]
    HomeNotFound,
    #[error("file not found: {0}")]
    FileNotFound(PathBuf),
    #[error("task join error: {0}")]
    TaskJoinError(#[from] tokio::task::JoinError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type StorageResult<T> = Result<T, StorageError>;
