use reqwest_middleware::ClientWithMiddleware;
use semver::Version;
use serde::de::DeserializeOwned;
use tracing::{debug, instrument};

use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT},
    StatusCode,
};

use crate::tool::{ToolId, ToolSpec};

use super::{client::create_client, source::ReleaseArtifact, Artifact, ArtifactProvider};

const BASE_URL: &str = "https://api.github.com";

pub mod models;
mod result;

use self::models::Release;

pub use self::result::{GithubError, GithubResult};

#[derive(Debug, Clone)]
pub struct GithubProvider {
    client: ClientWithMiddleware,
    has_auth: bool,
}

impl GithubProvider {
    fn new_inner(pat: Option<String>) -> GithubResult<Self> {
        let has_auth = pat.is_some();
        let headers = {
            let mut headers = HeaderMap::new();
            headers.insert(USER_AGENT, HeaderValue::from_static("rokit"));
            headers.insert(
                HeaderName::from_static("x-github-api-version"),
                HeaderValue::from_static("2022-11-28"),
            );
            if let Some(pat) = pat {
                let token = format!("Bearer {pat}");
                headers.insert(AUTHORIZATION, HeaderValue::from_str(&token)?);
            }
            headers
        };

        let client = create_client(headers)?;

        Ok(Self { client, has_auth })
    }

    async fn get_json<T: DeserializeOwned>(&self, url: &str) -> GithubResult<T> {
        let response = self
            .client
            .get(url)
            .header(ACCEPT, "application/vnd.github.v3+json")
            .send()
            .await?
            .error_for_status()?;
        Ok(response.json().await?)
    }

    async fn get_bytes(&self, url: &str) -> GithubResult<Vec<u8>> {
        let response = self
            .client
            .get(url)
            .header(ACCEPT, HeaderValue::from_static("application/octet-stream"))
            .send()
            .await?
            .error_for_status()?;
        let bytes = response.bytes().await.map(|bytes| bytes.to_vec());
        Ok(bytes?)
    }

    /**
        Creates a new GitHub source instance.

        # Errors

        - If the GitHub API client could not be created.
    */
    pub fn new() -> GithubResult<Self> {
        Self::new_inner(None)
    }

    /**
        Creates a new authenticated GitHub source instance with a token.

        May be used with either personal access tokens or tokens generated using the GitHub device flow.

        # Errors

        - If the GitHub API client could not be created.
    */
    pub fn new_authenticated(pat: impl AsRef<str>) -> GithubResult<Self> {
        let pat: String = pat.as_ref().trim().to_string();
        // https://github.blog/2021-04-05-behind-githubs-new-authentication-token-formats/
        if pat.starts_with("ghp_") || pat.starts_with("gho_") {
            Self::new_inner(Some(pat))
        } else {
            Err(GithubError::UnrecognizedAccessToken)
        }
    }

    /**
        Verifies that the current authentication token is valid.

        Returns `true` if the token is valid, `false` if it is not.

        Always returns `false` if the source is not authenticated.

        # Errors

        - If the request to the GitHub API failed.
    */
    pub async fn verify_authentication(&self) -> GithubResult<bool> {
        if !self.has_auth {
            return Ok(false);
        }

        let url = format!("{BASE_URL}/rate_limit");
        let res = self.get_json::<serde_json::Value>(&url).await;

        match res {
            Ok(_) => Ok(true),
            Err(e) if is_unauthenticated(&e) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /**
        Fetches the latest release for a given tool.
    */
    #[instrument(skip(self), fields(%tool_id), level = "debug")]
    pub async fn get_latest_release(&self, tool_id: &ToolId) -> GithubResult<ReleaseArtifact> {
        debug!(id = %tool_id, "fetching latest release for tool");

        let url = format!(
            "{BASE_URL}/repos/{owner}/{repo}/releases/latest",
            owner = tool_id.author(),
            repo = tool_id.name(),
        );

        let release: Release = match self.get_json(&url).await {
            Err(e) if is_404(&e) => {
                return Err(GithubError::LatestReleaseNotFound(tool_id.clone().into()));
            }
            Err(e) => return Err(e),
            Ok(r) => r,
        };

        let version = release
            .tag_name
            .trim_start_matches('v')
            .parse::<Version>()
            .map_err(|e| GithubError::Other(e.to_string()))?;

        let tool_spec: ToolSpec = (tool_id.clone(), version).into();
        Ok(ReleaseArtifact {
            artifacts: artifacts_from_release(&release.clone(), &tool_spec),
            changelog: release.changelog,
        })
    }

    /**
        Fetches a specific release for a given tool.
    */
    #[instrument(skip(self), fields(%tool_spec), level = "debug")]
    pub async fn get_specific_release(
        &self,
        tool_spec: &ToolSpec,
    ) -> GithubResult<ReleaseArtifact> {
        debug!(spec = %tool_spec, "fetching release for tool");

        let url_with_prefix = format!(
            "{BASE_URL}/repos/{owner}/{repo}/releases/tags/v{tag}",
            owner = tool_spec.author(),
            repo = tool_spec.name(),
            tag = tool_spec.version(),
        );
        let url_without_prefix = format!(
            "{BASE_URL}/repos/{owner}/{repo}/releases/tags/{tag}",
            owner = tool_spec.author(),
            repo = tool_spec.name(),
            tag = tool_spec.version(),
        );

        let release: Release = match self.get_json(&url_with_prefix).await {
            Err(e) if is_404(&e) => match self.get_json(&url_without_prefix).await {
                Err(e) if is_404(&e) => {
                    return Err(GithubError::ReleaseNotFound(tool_spec.clone().into()));
                }
                Err(e) => return Err(e),
                Ok(r) => r,
            },
            Err(e) => return Err(e),
            Ok(r) => r,
        };

        Ok(ReleaseArtifact {
            artifacts: artifacts_from_release(&release.clone(), tool_spec),
            changelog: release.changelog,
        })
    }

    /**
        Downloads the contents of the given artifact.
    */
    #[instrument(skip(self, artifact), level = "debug")]
    pub async fn download_artifact_contents(&self, artifact: &Artifact) -> GithubResult<Vec<u8>> {
        assert_eq!(
            artifact.provider,
            ArtifactProvider::GitHub,
            "artifact must be from GitHub"
        );

        let id = artifact.id.as_ref().expect("GitHub artifacts have ids");
        let name = artifact.name.as_ref().expect("GitHub artifacts have names");
        debug!(id, name, "downloading artifact contents");

        let url = format!(
            "{BASE_URL}/repos/{owner}/{repo}/releases/assets/{id}",
            owner = artifact.tool_spec.author(),
            repo = artifact.tool_spec.name(),
        );

        self.get_bytes(&url).await
    }
}

fn is_404(err: &GithubError) -> bool {
    if let GithubError::Reqwest(reqwest_err) = err {
        if let Some(status) = reqwest_err.status() {
            return status == StatusCode::NOT_FOUND;
        }
    }
    false
}

fn is_unauthenticated(err: &GithubError) -> bool {
    if let GithubError::Reqwest(reqwest_err) = err {
        if let Some(status) = reqwest_err.status() {
            return matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN);
        }
    }
    false
}

fn artifacts_from_release(release: &Release, spec: &ToolSpec) -> Vec<Artifact> {
    release
        .assets
        .iter()
        .map(|asset| Artifact::from_github_release_asset(asset, spec))
        .collect::<Vec<_>>()
}
