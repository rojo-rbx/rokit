use std::{fmt, str::FromStr};

use octocrab::models::repos::Asset;
use url::Url;

use crate::tool::ToolSpec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArtifactProvider {
    GitHub,
}

impl ArtifactProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GitHub => "github",
        }
    }
}

impl FromStr for ArtifactProvider {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let l = s.trim().to_lowercase();
        match l.as_str() {
            "github" => Ok(Self::GitHub),
            _ => Err(format!("unknown artifact provider '{l}'")),
        }
    }
}

impl fmt::Display for ArtifactProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    pub provider: ArtifactProvider,
    pub tool_spec: ToolSpec,
    pub source_url: Url,
    pub download_url: Url,
}

impl Artifact {
    pub(crate) fn from_github_release_asset(asset: &Asset, spec: &ToolSpec) -> Self {
        Self {
            provider: ArtifactProvider::GitHub,
            tool_spec: spec.clone(),
            source_url: asset.url.clone(),
            download_url: asset.browser_download_url.clone(),
        }
    }
}
