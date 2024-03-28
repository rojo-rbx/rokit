use std::{fmt, str::FromStr};

use octocrab::models::repos::Asset;
use tracing::instrument;
use url::Url;

use crate::{
    descriptor::Descriptor,
    result::{RokitError, RokitResult},
    tool::ToolSpec,
};

use super::{
    decompression::decompress_gzip,
    extraction::{extract_tar_file, extract_zip_file},
};

/**
    An artifact provider supported by Rokit.
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
    An artifact format supported by Rokit.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArtifactFormat {
    Zip,
    Tar,
    TarGz,
}

impl ArtifactFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Zip => "zip",
            Self::Tar => "tar",
            Self::TarGz => "tar.gz",
        }
    }

    pub fn from_path_or_url(path_or_url: impl AsRef<str>) -> Option<Self> {
        let l = path_or_url.as_ref().trim().to_lowercase();
        match l.as_str() {
            s if s.ends_with(".zip") => Some(Self::Zip),
            s if s.ends_with(".tar") => Some(Self::Tar),
            s if s.ends_with(".tar.gz") => Some(Self::TarGz),
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
            "tar" => Ok(Self::Tar),
            "tar.gz" => Ok(Self::TarGz),
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
    An artifact found by Rokit, to be downloaded and installed.
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    pub provider: ArtifactProvider,
    pub format: Option<ArtifactFormat>,
    pub id: Option<String>,
    pub url: Option<Url>,
    pub name: Option<String>,
    pub tool_spec: ToolSpec,
}

impl Artifact {
    pub(crate) fn from_github_release_asset(asset: &Asset, spec: &ToolSpec) -> Self {
        let format = ArtifactFormat::from_path_or_url(&asset.name);
        Self {
            provider: ArtifactProvider::GitHub,
            format,
            id: Some(asset.id.to_string()),
            url: Some(asset.url.clone()),
            name: Some(asset.name.clone()),
            tool_spec: spec.clone(),
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
    pub async fn extract_contents(&self, contents: Vec<u8>) -> RokitResult<Vec<u8>> {
        let format = self.format.ok_or(RokitError::ExtractUnknownFormat)?;

        let file_name = self.tool_spec.name().to_string();
        let file_res = match format {
            ArtifactFormat::Zip => extract_zip_file(&contents, &file_name).await,
            ArtifactFormat::Tar => extract_tar_file(&contents, &file_name).await,
            ArtifactFormat::TarGz => {
                let tar = decompress_gzip(&contents).await?;
                extract_tar_file(&tar, &file_name).await
            }
        };

        let file_opt = file_res.map_err(|err| RokitError::ExtractError {
            source: err.into(),
            body: {
                if contents.len() > 128 + 6 {
                    let bytes = contents.iter().cloned().take(128).collect::<Vec<_>>();
                    format!("{} <...>", String::from_utf8_lossy(bytes.as_slice()).trim())
                } else {
                    String::from_utf8_lossy(&contents).to_string()
                }
            },
        })?;

        file_opt.ok_or(RokitError::ExtractFileMissing {
            format,
            file_name: self.tool_spec.name().to_string(),
            archive_name: self.name.clone().unwrap_or_default(),
        })
    }

    /**
        Sorts the given artifacts by system compatibility.

        See also:

        - [`Descriptor::current_system`]
        - [`Descriptor::is_compatible_with`]
        - [`Descriptor::sort_by_preferred_compat`]
    */
    pub fn sort_by_system_compatibility(artifacts: impl AsRef<[Self]>) -> Vec<Self> {
        let current_desc = Descriptor::current_system();

        let mut compatible_artifacts = artifacts
            .as_ref()
            .iter()
            .filter_map(|artifact| {
                let name = artifact.name.as_deref()?;
                if let Some(asset_desc) = Descriptor::detect(name) {
                    if current_desc.is_compatible_with(&asset_desc) {
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
            current_desc.sort_by_preferred_compat(desc_a, desc_b)
        });

        compatible_artifacts
            .into_iter()
            .map(|(_, artifact)| artifact.clone())
            .collect()
    }
}
