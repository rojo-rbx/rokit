use reqwest::{header::InvalidHeaderValue, Error as ReqwestError};
use thiserror::Error;

use crate::tool::{ToolId, ToolSpec};

#[derive(Debug, Error)]
pub enum GithubError {
    #[error("unrecognized access token format - must begin with `ghp_` or `gho_`.")]
    UnrecognizedAccessToken,
    #[error("no latest release was found for tool '{0}'")]
    LatestReleaseNotFound(Box<ToolId>),
    #[error("no release was found for tool '{0}'")]
    ReleaseNotFound(Box<ToolSpec>),
    #[error("failed to build client - invalid header value: {0}")]
    ReqwestHeader(Box<InvalidHeaderValue>),
    #[error("reqwest middleware error: {0}")]
    ReqwestMiddleware(Box<reqwest_middleware::Error>),
    #[error("reqwest error: {0}")]
    Reqwest(Box<reqwest::Error>),
    #[error("other error: {0}")]
    Other(String),
}

pub type GithubResult<T> = Result<T, GithubError>;

// FUTURE: Figure out some way to reduce this boxing boilerplate

impl From<InvalidHeaderValue> for GithubError {
    fn from(err: InvalidHeaderValue) -> Self {
        GithubError::ReqwestHeader(err.into())
    }
}

impl From<reqwest_middleware::Error> for GithubError {
    fn from(err: reqwest_middleware::Error) -> Self {
        GithubError::ReqwestMiddleware(err.into())
    }
}

impl From<ReqwestError> for GithubError {
    fn from(err: ReqwestError) -> Self {
        GithubError::Reqwest(err.into())
    }
}
