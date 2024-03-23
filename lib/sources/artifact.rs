use octocrab::models::repos::Asset;
use url::Url;

use crate::tool::ToolSpec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArtifactProvider {
    GitHub,
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
