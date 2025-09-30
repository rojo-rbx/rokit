#![allow(clippy::missing_errors_doc)]

use reqwest_middleware::ClientWithMiddleware;
use semver::Version;
use serde::de::DeserializeOwned;
use tracing::{debug, instrument};

use reqwest::{
    StatusCode,
    header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderName, HeaderValue},
};

use crate::tool::{ToolId, ToolSpec, util::to_xyz_version};

use super::{Artifact, ArtifactProvider, Release, client::create_client};

const BASE_URL: &str = "https://api.github.com";

pub mod models;
mod result;

use self::models::GithubRelease;

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

        Note that this does not verify the formatting or validity of the token,
        use the `verify_authentication` method for checking with the GitHub API.

        # Errors

        - If the GitHub API client could not be created.
    */
    pub fn new_authenticated(pat: impl AsRef<str>) -> GithubResult<Self> {
        let pat: String = pat.as_ref().trim().to_string();
        Self::new_inner(Some(pat))
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
    pub async fn get_latest_release(&self, tool_id: &ToolId) -> GithubResult<Release> {
        debug!(id = %tool_id, "fetching latest release for tool");

        let url = format!(
            "{BASE_URL}/repos/{owner}/{repo}/releases/latest",
            owner = tool_id.author(),
            repo = tool_id.name(),
        );

        let release: GithubRelease = match self.get_json(&url).await {
            Err(e) if is_404(&e) => {
                return Err(GithubError::LatestReleaseNotFound(tool_id.clone().into()));
            }
            Err(e) => return Err(e),
            Ok(r) => r,
        };

        let version_str = release.tag_name.trim_start_matches('v');
        let version_str_xyz = to_xyz_version(version_str);
        let version = version_str_xyz
            .parse::<Version>()
            .map_err(|e| GithubError::Other(e.to_string()))?;

        let tool_spec: ToolSpec = (tool_id.clone(), version).into();
        Ok(Release {
            changelog: release.changelog.clone(),
            artifacts: artifacts_from_release(&release, &tool_spec),
        })
    }

    /**
        Fetches a specific release for a given tool.
    */
    #[instrument(skip(self), fields(%tool_spec), level = "debug")]
    pub async fn get_specific_release(&self, tool_spec: &ToolSpec) -> GithubResult<Release> {
        debug!(spec = %tool_spec, "fetching release for tool");
        let mut tags_to_try = vec![tool_spec.version().to_string()];
        let version = tool_spec.version();
        if version.patch == 0 && version.pre.is_empty() && version.build.is_empty() {
            tags_to_try.push(format!("{}.{}", version.major, version.minor));
        }
        for tag in tags_to_try {
            let url_with_prefix = format!(
                "{BASE_URL}/repos/{owner}/{repo}/releases/tags/v{tag}",
                owner = tool_spec.author(),
                repo = tool_spec.name(),
            );
            let url_without_prefix = format!(
                "{BASE_URL}/repos/{owner}/{repo}/releases/tags/{tag}",
                owner = tool_spec.author(),
                repo = tool_spec.name(),
            );
            match self.get_json::<GithubRelease>(&url_with_prefix).await {
                Ok(release) => {
                    return Ok(Release {
                        changelog: release.changelog.clone(),
                        artifacts: artifacts_from_release(&release, tool_spec),
                    });
                }
                Err(e) if !is_404(&e) => return Err(e),
                _ => {}
            }
            match self.get_json::<GithubRelease>(&url_without_prefix).await {
                Ok(release) => {
                    return Ok(Release {
                        changelog: release.changelog.clone(),
                        artifacts: artifacts_from_release(&release, tool_spec),
                    });
                }
                Err(e) if !is_404(&e) => return Err(e),
                _ => {}
            }
        }
        Err(GithubError::ReleaseNotFound(tool_spec.clone().into()))
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
    if let GithubError::Reqwest(reqwest_err) = err
        && let Some(status) = reqwest_err.status()
    {
        return status == StatusCode::NOT_FOUND;
    }
    false
}

fn is_unauthenticated(err: &GithubError) -> bool {
    if let GithubError::Reqwest(reqwest_err) = err
        && let Some(status) = reqwest_err.status()
    {
        return matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN);
    }
    false
}

fn artifacts_from_release(release: &GithubRelease, spec: &ToolSpec) -> Vec<Artifact> {
    release
        .assets
        .iter()
        .map(|asset| Artifact::from_github_release_asset(asset, spec))
        .collect::<Vec<_>>()
}
