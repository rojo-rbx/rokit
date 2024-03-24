use std::{fmt, str::FromStr};

use octocrab::models::repos::Asset;
use tracing::{debug, instrument};
use url::Url;

use crate::{
    result::{AftmanError, AftmanResult},
    tool::ToolSpec,
};

use super::extraction::extract_zip_file;

/**
    An artifact provider supported by Aftman.
*/
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

/**
    An artifact format supported by Aftman.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArtifactFormat {
    Zip,
}

impl ArtifactFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Zip => "zip",
        }
    }

    pub fn from_path_or_url(path_or_url: impl AsRef<str>) -> Option<Self> {
        let l = path_or_url.as_ref().trim().to_lowercase();
        match l.as_str() {
            s if s.ends_with(".zip") => Some(Self::Zip),
            _ => None,
        }
    }
}

impl FromStr for ArtifactFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let l = s.trim().to_lowercase();
        match l.as_str() {
            "zip" => Ok(Self::Zip),
            _ => Err(format!("unknown artifact format '{l}'")),
        }
    }
}

impl fmt::Display for ArtifactFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

/**
    An artifact found by Aftman, to be downloaded and installed.
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    pub provider: ArtifactProvider,
    pub format: Option<ArtifactFormat>,
    pub tool_spec: ToolSpec,
    pub source_url: Url,
    pub download_url: Url,
}

impl Artifact {
    pub(crate) fn from_github_release_asset(asset: &Asset, spec: &ToolSpec) -> Self {
        let download_url = asset.browser_download_url.clone();
        let format = ArtifactFormat::from_path_or_url(&asset.name).or_else(|| {
            // TODO: The url path here is percent-encoded ... we should
            // probably decode it first before guessing the artifact format
            ArtifactFormat::from_path_or_url(download_url.path())
        });
        Self {
            provider: ArtifactProvider::GitHub,
            format,
            tool_spec: spec.clone(),
            source_url: asset.url.clone(),
            download_url,
        }
    }

    /**
        Extract the contents of the artifact.

        The given contents must be the raw bytes of the artifact,
        in the expected format, as downloaded from the download URL.

        This generally means that, as long as the same artifact provider
        is used to both create and download the artifact, the format
        should be known and the contents should be in the correct format.
    */
    #[instrument(skip(self, contents), level = "debug")]
    pub async fn extract_contents(&self, contents: Vec<u8>) -> AftmanResult<Vec<u8>> {
        debug!("Extracting artifact contents");

        let format = self.format.ok_or(AftmanError::ExtractUnknownFormat)?;

        let file_name = self.tool_spec.name().to_string();
        let file_opt = match format {
            ArtifactFormat::Zip => extract_zip_file(contents, &file_name).await?,
        };

        file_opt.ok_or(AftmanError::ExtractFileMissing)
    }
}
