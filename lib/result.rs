use std::path::PathBuf;

use thiserror::Error;

use crate::sources::ArtifactFormat;

#[derive(Debug, Error)]
pub enum RokitError {
    #[error("home directory not found")]
    HomeNotFound,
    #[error("file not found: {0}")]
    FileNotFound(PathBuf),
    #[error("failed to extract artifact: unknown format")]
    ExtractUnknownFormat,
    #[error("failed to extract artifact: missing binary '{file_name}' in {format} file '{archive_name}'")]
    ExtractFileMissing {
        format: ArtifactFormat,
        file_name: String,
        archive_name: String,
    },
    #[error("failed to extract artifact:\n{source}\nresponse body first bytes:\n{body}")]
    ExtractError {
        source: Box<dyn std::error::Error + Send + Sync>,
        body: String,
    },
    #[error("unexpected invalid UTF-8")]
    InvalidUtf8,
    #[error("task join error: {0}")]
    TaskJoinError(#[from] tokio::task::JoinError),
    #[error("TOML parse error: {0}")]
    TomlParseError(#[from] toml_edit::TomlError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Zip file error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("GitHub error: {0}")]
    GitHub(#[from] octocrab::Error),
}

pub type RokitResult<T> = Result<T, RokitError>;
