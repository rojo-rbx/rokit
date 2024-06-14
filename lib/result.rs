use std::io::Error as IoError;
use std::path::PathBuf;

use postcard::Error as PostcardError;
use serde_json::Error as JsonError;
use thiserror::Error;
use tokio::task::JoinError;
use toml_edit::TomlError;
use zip::result::ZipError;

use crate::sources::{github::GithubError, ExtractError};

#[derive(Debug, Error)]
pub enum RokitError {
    #[error("home directory not found")]
    HomeNotFound,
    #[error("file not found: {0}")]
    FileNotFound(PathBuf),
    #[error("unexpected invalid UTF-8")]
    InvalidUtf8,
    #[error("failed to extract artifact: {0}")]
    Extract(Box<ExtractError>),
    #[error("task join error: {0}")]
    TaskJoinError(Box<JoinError>),
    #[error("TOML parse error: {0}")]
    TomlParseError(Box<TomlError>),
    #[error("I/O error: {0}")]
    Io(Box<IoError>),
    #[error("JSON error: {0}")]
    Json(Box<JsonError>),
    #[error("Postcard error: {0}")]
    Postcard(Box<PostcardError>),
    #[error("Zip file error: {0}")]
    Zip(Box<ZipError>),
    #[error("GitHub error: {0}")]
    GitHub(Box<GithubError>),
}

pub type RokitResult<T> = Result<T, RokitError>;

// FUTURE: Figure out some way to reduce this boxing boilerplate

impl From<ExtractError> for RokitError {
    fn from(err: ExtractError) -> Self {
        RokitError::Extract(err.into())
    }
}

impl From<JoinError> for RokitError {
    fn from(err: JoinError) -> Self {
        RokitError::TaskJoinError(err.into())
    }
}

impl From<TomlError> for RokitError {
    fn from(err: TomlError) -> Self {
        RokitError::TomlParseError(err.into())
    }
}

impl From<IoError> for RokitError {
    fn from(err: IoError) -> Self {
        RokitError::Io(err.into())
    }
}

impl From<JsonError> for RokitError {
    fn from(err: JsonError) -> Self {
        RokitError::Json(Box::new(err))
    }
}

impl From<PostcardError> for RokitError {
    fn from(err: PostcardError) -> Self {
        RokitError::Postcard(err.into())
    }
}

impl From<ZipError> for RokitError {
    fn from(err: ZipError) -> Self {
        RokitError::Zip(err.into())
    }
}

impl From<GithubError> for RokitError {
    fn from(err: GithubError) -> Self {
        RokitError::GitHub(err.into())
    }
}
