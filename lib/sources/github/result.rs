use thiserror::Error;

use crate::tool::{ToolId, ToolSpec};

#[derive(Debug, Error)]
pub enum GithubError {
    #[error("unrecognized access token format - must begin with `ghp_` or `gho_`.")]
    UnrecognizedAccessToken,
    #[error("no latest release was found for tool '{0}'")]
    LatestReleaseNotFound(ToolId),
    #[error("no release was found for tool '{0}'")]
    ReleaseNotFound(ToolSpec),
    #[error("failed to build client - invalid header value: {0}")]
    ReqwestHeader(#[from] reqwest::header::InvalidHeaderValue),
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("other error: {0}")]
    Other(String),
}

pub type GithubResult<T> = Result<T, GithubError>;
