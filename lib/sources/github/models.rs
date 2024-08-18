use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, Deserialize)]
pub struct GithubRelease {
    pub assets: Vec<GithubAsset>,
    pub tag_name: String,
    pub prerelease: bool,
    #[serde(rename = "body")]
    pub changelog: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GithubAsset {
    pub id: u64,
    pub url: Url,
    pub name: String,
}
