use std::{
    backtrace::Backtrace,
    io::{stdout, IsTerminal},
    time::Duration,
};

use http::{
    header::{ACCEPT, USER_AGENT},
    HeaderMap, HeaderValue, StatusCode,
};
use octocrab::{models::repos::Release, Error, Octocrab, OctocrabBuilder, Result};
use secrecy::{ExposeSecret, SecretString};
use semver::Version;
use tokio::time::sleep;
use tracing::{debug, info, instrument};

use crate::tool::{ToolId, ToolSpec};

use super::{Artifact, ArtifactProvider};

const BASE_URI: &str = "https://api.github.com";

const ERR_AUTH_UNRECOGNIZED: &str =
    "Unrecognized access token format - must begin with `ghp_` or `gho_`.";
const _ERR_AUTH_DEVICE_INTERACTIVE: &str =
    "Device authentication flow may only be used in an interactive terminal.";

#[derive(Debug, Clone)]
pub struct GithubProvider {
    gh_client: Octocrab,
    dl_client: reqwest::Client,
}

impl GithubProvider {
    /**
        Creates a new GitHub source instance.
    */
    pub fn new() -> Result<Self> {
        let gh_client = crab_builder().build()?;
        let dl_client = build_dl_client(None)?;
        Ok(Self {
            gh_client,
            dl_client,
        })
    }

    /**
        Creates a new authenticated GitHub source instance with a token.

        May be used with either personal access tokens or tokens generated using the GitHub device flow.
    */
    pub fn new_authenticated(pat: impl AsRef<str>) -> Result<Self> {
        let pat: String = pat.as_ref().trim().to_string();
        // https://github.blog/2021-04-05-behind-githubs-new-authentication-token-formats/
        if pat.starts_with("ghp_") {
            Ok(Self {
                gh_client: crab_builder().personal_token(pat.clone()).build()?,
                dl_client: build_dl_client(Some(pat))?,
            })
        } else if pat.starts_with("gho_") {
            Ok(Self {
                gh_client: crab_builder().user_access_token(pat.clone()).build()?,
                dl_client: build_dl_client(Some(pat))?,
            })
        } else {
            Err(Error::Other {
                source: ERR_AUTH_UNRECOGNIZED.into(),
                backtrace: Backtrace::capture(),
            })
        }
    }

    /**
        Authenticates with GitHub using the device flow.

        Note that this will emit messages using `info` to guide the
        user through the authentication process, and requires user interaction.
        If the user does not interact, this will keep polling the GitHub API for a
        maximum of 15 minutes (900 seconds) before timing out and returning an error.

        Returns the access token if authentication is successful, but *does not* store it.
        A new client instance must be created using `new_authenticated` to use it.
    */
    pub async fn _auth_with_device<C, I, S>(&self, client_id: C, scope: I) -> Result<String>
    where
        C: Into<SecretString>,
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        if !stdout().is_terminal() {
            return Err(Error::Other {
                source: _ERR_AUTH_DEVICE_INTERACTIVE.into(),
                backtrace: Backtrace::capture(),
            });
        }

        let client_id = client_id.into();
        let codes = self
            .gh_client
            .authenticate_as_device(&client_id, scope)
            .await?;

        info!(
            "Authentication is awaiting your approval.\
            \nPlease visit the authentication page: {}\
            \nAnd enter the verification code: {}",
            codes.verification_uri, codes.user_code
        );

        let oauth = loop {
            sleep(Duration::from_secs(codes.interval)).await;
            let status = codes.poll_once(&self.gh_client, &client_id).await?;
            if status.is_left() {
                break status.unwrap_left();
            }
        };

        info!("Authentication successful!");
        let token = oauth.access_token.expose_secret().clone();

        Ok(token)
    }

    /**
        Fetches the latest release for a given tool.
    */
    #[instrument(skip(self), fields(%tool_id), level = "debug")]
    pub async fn get_latest_release(&self, tool_id: &ToolId) -> Result<Vec<Artifact>> {
        debug!(id = %tool_id, "fetching latest release for tool");

        let repository = self.gh_client.repos(tool_id.author(), tool_id.name());
        let releases = repository.releases();

        let release = releases.get_latest().await?;
        let version = release
            .tag_name
            .trim_start_matches('v')
            .parse::<Version>()
            .map_err(other_err)?;

        let tool_spec: ToolSpec = (tool_id.clone(), version).into();
        Ok(artifacts_from_release(release, &tool_spec))
    }

    /**
        Fetches a specific release for a given tool.
    */
    #[instrument(skip(self), fields(%tool_spec), level = "debug")]
    pub async fn get_specific_release(&self, tool_spec: &ToolSpec) -> Result<Vec<Artifact>> {
        debug!(spec = %tool_spec, "fetching release for tool");

        let repository = self.gh_client.repos(tool_spec.author(), tool_spec.name());
        let releases = repository.releases();

        let tag_with_prefix = format!("v{}", tool_spec.version());
        let tag_without_prefix = tool_spec.version().to_string();

        let release = match releases.get_by_tag(&tag_with_prefix).await {
            Err(err) if is_github_not_found(&err) => releases.get_by_tag(&tag_without_prefix).await,
            Err(err) => Err(err),
            Ok(release) => Ok(release),
        }?;

        Ok(artifacts_from_release(release, tool_spec))
    }

    /**
        Downloads the contents of the given artifact.
    */
    #[instrument(skip(self, artifact), level = "debug")]
    pub async fn download_artifact_contents(&self, artifact: &Artifact) -> Result<Vec<u8>> {
        assert_eq!(
            artifact.provider,
            ArtifactProvider::GitHub,
            "artifact must be from GitHub"
        );

        let id = artifact.id.as_ref().expect("GitHub artifacts have ids");
        let name = artifact.name.as_ref().expect("GitHub artifacts have names");
        debug!(id, name, "downloading artifact contents");

        let url = format!(
            "{BASE_URI}/repos/{owner}/{repo}/releases/assets/{id}",
            owner = artifact.tool_spec.author(),
            repo = artifact.tool_spec.name(),
        );

        let response = self.dl_client.get(url).send().await.map_err(other_err)?;
        let bytes = response.bytes().await.map_err(other_err)?;

        Ok(bytes.to_vec())
    }
}

fn artifacts_from_release(release: Release, spec: &ToolSpec) -> Vec<Artifact> {
    release
        .assets
        .iter()
        .map(|asset| Artifact::from_github_release_asset(asset, spec))
        .collect::<Vec<_>>()
}

fn other_err(err: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Error {
    Error::Other {
        source: err.into(),
        backtrace: Backtrace::capture(),
    }
}

fn build_dl_client(pat: Option<String>) -> Result<reqwest::Client> {
    let headers = {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/octet-stream"));
        headers.insert(USER_AGENT, HeaderValue::from_static("rokit"));
        if let Some(pat) = pat {
            let token = format!("Bearer {pat}");
            headers.insert(
                "Authorization",
                HeaderValue::from_str(&token).map_err(other_err)?,
            );
        }
        headers
    };
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(other_err)?;
    Ok(client)
}

// So generic, such wow

use octocrab::{DefaultOctocrabBuilderConfig, NoAuth, NoSvc, NotLayerReady};

fn crab_builder() -> OctocrabBuilder<NoSvc, DefaultOctocrabBuilderConfig, NoAuth, NotLayerReady> {
    OctocrabBuilder::new()
        .base_uri(BASE_URI)
        .unwrap()
        .add_header(ACCEPT, String::from("application/json"))
}

fn is_github_not_found(err: &Error) -> bool {
    if let Error::GitHub { source, .. } = err {
        source.status_code == StatusCode::NOT_FOUND
    } else {
        false
    }
}
