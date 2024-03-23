use std::{
    backtrace::Backtrace,
    io::{stdout, IsTerminal},
    time::Duration,
};

use octocrab::{models::repos::Release, Error, Octocrab, OctocrabBuilder, Result};
use reqwest::header::ACCEPT;
use secrecy::{ExposeSecret, SecretString};
use semver::Version;
use tokio::time::sleep;
use tracing::{info, instrument, trace};

use crate::{
    description::Description,
    tool::{ToolId, ToolSpec},
};

use super::Artifact;

const BASE_URI: &str = "https://api.github.com";

const ERR_AUTH_UNRECOGNIZED: &str =
    "Unrecognized access token format - must begin with `ghp_` or `gho_`.";
const ERR_AUTH_DEVICE_INTERACTIVE: &str =
    "Device authentication flow may only be used in an interactive terminal.";

// NOTE: Users typically install somewhat recent tools, and fetching
// a smaller number of releases here lets us install tools much faster.
const RESULTS_PER_PAGE: u8 = 8;

pub struct GitHubSource {
    client: Octocrab,
}

impl GitHubSource {
    /**
        Creates a new GitHub source instance.

        This instance is unauthenticated and may be rate limited and/or unable to access
        private resources. To authenticate using an access token, use `new_authenticated`.
    */
    pub fn new() -> Result<Self> {
        let client = crab_builder().build()?;
        Ok(Self { client })
    }

    /**
        Creates a new authorized GitHub source instance with a personal access token.

        May be used with either personal access tokens or tokens generated using the GitHub device flow.
    */
    pub fn new_authenticated(pat: impl AsRef<str>) -> Result<Self> {
        let pat: String = pat.as_ref().trim().to_string();
        // https://github.blog/2021-04-05-behind-githubs-new-authentication-token-formats/
        if pat.starts_with("ghp_") {
            Ok(Self {
                client: crab_builder().personal_token(pat).build()?,
            })
        } else if pat.starts_with("gho_") {
            Ok(Self {
                client: crab_builder().user_access_token(pat).build()?,
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
    pub async fn auth_with_device<C, I, S>(&self, client_id: C, scope: I) -> Result<String>
    where
        C: Into<SecretString>,
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        if !stdout().is_terminal() {
            return Err(Error::Other {
                source: ERR_AUTH_DEVICE_INTERACTIVE.into(),
                backtrace: Backtrace::capture(),
            });
        }

        let client_id = client_id.into();
        let codes = self
            .client
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
            let status = codes.poll_once(&self.client, &client_id).await?;
            if status.is_left() {
                break status.unwrap_left();
            }
        };

        info!("Authentication successful!");
        let token = oauth.access_token.expose_secret().clone();

        Ok(token)
    }

    /**
        Fetches a page of releases for a given tool.
    */
    #[instrument(skip(self), fields(%tool_id), level = "trace")]
    pub async fn get_releases(&self, tool_id: &ToolId, page: u32) -> Result<Vec<Release>> {
        trace!("fetching releases for tool");

        let repository = self.client.repos(tool_id.author(), tool_id.name());
        let releases = repository
            .releases()
            .list()
            .per_page(RESULTS_PER_PAGE)
            .page(page)
            .send()
            .await?;

        Ok(releases.items)
    }

    /**
        Fetches a specific release for a given tool.
    */
    #[instrument(skip(self), fields(%tool_spec), level = "trace")]
    pub async fn find_release(&self, tool_spec: &ToolSpec) -> Result<Option<Release>> {
        trace!("fetching release for tool");

        let repository = self.client.repos(tool_spec.author(), tool_spec.name());
        let releases = repository.releases();

        let tag_with_prefix = format!("v{}", tool_spec.version());
        let tag_without_prefix = tool_spec.version().to_string();
        let (response_with_prefix, response_without_prefix) = tokio::join!(
            releases.get_by_tag(&tag_with_prefix),
            releases.get_by_tag(&tag_without_prefix),
        );

        if response_with_prefix.is_err() && response_without_prefix.is_err() {
            #[allow(clippy::unnecessary_unwrap)]
            return Err(response_with_prefix.unwrap_err());
        }

        let opt_with_prefix = response_with_prefix.ok();
        let opt_without_prefix = response_without_prefix.ok();
        Ok(opt_with_prefix.or(opt_without_prefix))
    }

    /**
        Finds the latest version of a tool, optionally allowing prereleases.

        If no releases are found, or no non-prerelease releases are found, this will return `None`.
    */
    #[instrument(skip(self), fields(%tool_id), level = "trace")]
    pub async fn find_latest_version(
        &self,
        tool_id: &ToolId,
        allow_prereleases: bool,
    ) -> Result<Option<Version>> {
        trace!("fetching latest version for tool");

        let releases = self.get_releases(tool_id, 1).await?;
        Ok(releases.into_iter().find_map(|release| {
            if allow_prereleases || !release.prerelease {
                let version = release.tag_name.trim_start_matches('v');
                Version::parse(version).ok()
            } else {
                None
            }
        }))
    }

    /**
        Finds compatible release artifacts for the given release and description.

        The resulting list of artifacts will be sorted by preferred compatibility.

        See [`Description::is_compatible_with`] and
        [`Description::sort_by_preferred_compat`] for more information.
    */
    pub fn find_compatible_artifacts(
        &self,
        tool_spec: &ToolSpec,
        release: &Release,
        description: &Description,
    ) -> Vec<Artifact> {
        let mut compatible_artifacts = release
            .assets
            .iter()
            .filter_map(|asset| {
                if let Some(asset_desc) = Description::detect(asset.name.as_str()) {
                    if description.is_compatible_with(&asset_desc) {
                        let artifact = Artifact::from_github_release_asset(asset, tool_spec);
                        Some((asset_desc, artifact))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        compatible_artifacts.sort_by(|(desc_a, _), (desc_b, _)| {
            description.sort_by_preferred_compat(desc_a, desc_b)
        });

        compatible_artifacts
            .into_iter()
            .map(|(_, artifact)| artifact)
            .collect()
    }
}

// So generic, such wow

use octocrab::{DefaultOctocrabBuilderConfig, NoAuth, NoSvc, NotLayerReady};

fn crab_builder() -> OctocrabBuilder<NoSvc, DefaultOctocrabBuilderConfig, NoAuth, NotLayerReady> {
    OctocrabBuilder::new()
        .base_uri(BASE_URI)
        .unwrap()
        .add_header(ACCEPT, String::from("application/json"))
}
