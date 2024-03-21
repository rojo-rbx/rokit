use std::io::{Cursor, Read, Seek};

use anyhow::Context;
use reqwest::{
    blocking::Client,
    header::{ACCEPT, AUTHORIZATION, USER_AGENT},
};
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::auth::AuthManifest;
use crate::tool_id::ToolId;
use crate::tool_name::ToolName;
use crate::tool_source::Asset;

use super::Release;

const APP_NAME: &str = "LPGhatguy/aftman";

pub struct GitHubSource {
    client: Client,
    token: Option<String>,
}

impl GitHubSource {
    pub fn new(auth: Option<&AuthManifest>) -> Self {
        Self {
            client: Client::new(),
            token: auth.and_then(|t| t.github.clone()),
        }
    }

    pub fn get_all_releases(&self, name: &ToolName) -> anyhow::Result<Vec<Release>> {
        let url = format!("https://api.github.com/repos/{}/releases", name);
        let mut builder = self.client.get(url).header(USER_AGENT, APP_NAME);

        if let Some(token) = &self.token {
            builder = builder.header(AUTHORIZATION, format!("token {}", token));
        }

        let response_body = builder.send()?.text()?;

        let gh_releases: Vec<GitHubRelease> = serde_json::from_str(&response_body)
            .with_context(|| format!("Unexpected GitHub API response: {}", response_body))?;

        let releases: Vec<Release> = gh_releases
            .into_iter()
            .filter_map(|release| {
                let stripped = release
                    .tag_name
                    .strip_prefix('v')
                    .unwrap_or(release.tag_name.as_str());
                let version = stripped.parse::<Version>().ok()?;

                let assets = release
                    .assets
                    .into_iter()
                    .filter(|asset| asset.name.ends_with(".zip"))
                    .map(|asset| Asset::from_name_url(&asset.name, &asset.url))
                    .collect();

                Some(Release {
                    version,
                    assets,
                    prerelease: release.prerelease,
                })
            })
            .collect();

        Ok(releases)
    }

    pub fn get_release(&self, id: &ToolId) -> anyhow::Result<Release> {
        // TODO: Better implementation using individual release API instead of
        // using the release list API.

        let releases = self.get_all_releases(id.name())?;

        releases
            .into_iter()
            .find(|release| &release.version == id.version())
            .with_context(|| format!("Could not find release {}", id))
    }

    pub fn download_asset(&self, url: &str) -> anyhow::Result<impl Read + Seek> {
        let mut builder = self
            .client
            .get(url)
            .header(USER_AGENT, APP_NAME)
            .header(ACCEPT, "application/octet-stream");

        if let Some(token) = &self.token {
            builder = builder.header(AUTHORIZATION, format!("token {}", token));
        }

        let response = builder.send()?;
        let body = response.bytes()?.to_vec();

        Ok(Cursor::new(body))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub prerelease: bool,
    pub assets: Vec<GitHubReleaseAsset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubReleaseAsset {
    pub url: String,
    pub name: String,
}
