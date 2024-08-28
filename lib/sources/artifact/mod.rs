use tracing::instrument;
use url::Url;

use crate::{
    descriptor::{Descriptor, OS},
    result::RokitResult,
    tool::ToolSpec,
};

use super::{
    decompression::decompress_gzip,
    extraction::{extract_tar_file, extract_zip_file},
    github::models::GithubAsset,
    ExtractError,
};

mod format;
mod provider;
mod sorting;
mod util;

use self::sorting::sort_preferred_artifact;
use self::sorting::sort_preferred_formats;
use self::util::split_filename_and_extensions;

pub use self::format::ArtifactFormat;
pub use self::provider::ArtifactProvider;

/**
    A release found by Rokit, containing a list
    of artifacts, and optionally a changelog.
*/
#[derive(Debug, Clone)]
pub struct Release {
    pub changelog: Option<String>,
    pub artifacts: Vec<Artifact>,
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
    pub(crate) fn from_github_release_asset(asset: &GithubAsset, spec: &ToolSpec) -> Self {
        let (name, extensions) = split_filename_and_extensions(&asset.name);
        let format = ArtifactFormat::from_extensions(extensions);
        Self {
            provider: ArtifactProvider::GitHub,
            format,
            id: Some(asset.id.to_string()),
            url: Some(asset.url.clone()),
            name: Some(name.to_string()),
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
        let format = self.format.ok_or(ExtractError::UnknownFormat)?;

        let file_name = self.tool_spec.name().to_string();
        let file_res = match format {
            ArtifactFormat::Zip => extract_zip_file(&contents, &file_name).await,
            ArtifactFormat::Tar => extract_tar_file(&contents, &file_name).await,
            ArtifactFormat::TarGz => {
                let tar = decompress_gzip(&contents).await?;
                extract_tar_file(&tar, &file_name).await
            }
            ArtifactFormat::Gz => decompress_gzip(&contents).await.map(Some),
        };

        // Make sure we got back the file we need ...

        let file_opt = file_res.map_err(|err| ExtractError::Generic {
            source: err.into(),
            body: {
                if contents.len() > 128 + 6 {
                    let bytes = contents.iter().copied().take(128).collect::<Vec<_>>();
                    format!("{} <...>", String::from_utf8_lossy(bytes.as_slice()).trim())
                } else {
                    String::from_utf8_lossy(&contents).to_string()
                }
            },
        })?;

        let file_bytes = file_opt.ok_or_else(|| ExtractError::FileMissing {
            format,
            file_name: self.tool_spec.name().to_string(),
            archive_name: self.name.clone().unwrap_or_default(),
        })?;

        // ... and parse the OS from the executable binary, or error,
        // to ensure that the user will actually be able to run it

        let os_current = OS::current_system();
        let os_file = OS::detect_from_executable(&file_bytes);
        if os_file.is_some_and(|os| os != os_current) {
            Err(ExtractError::OSMismatch {
                current_os: os_current,
                file_os: os_file.unwrap(),
                file_name: self.tool_spec.name().to_string(),
                archive_name: self.name.clone().unwrap_or_default(),
            })?;
        }

        Ok(file_bytes)
    }

    /**
        Sorts the given artifacts by their compatibility with the current system.

        See also:

        - [`Descriptor::current_system`]
        - [`Descriptor::is_compatible_with`]
        - [`Descriptor::sort_by_preferred_compat`]
    */
    pub fn sort_by_system_compatibility(artifacts: impl AsRef<[Self]>) -> Vec<Self> {
        Self::sort_by_system_compatibility_inner(artifacts, false)
    }

    /**
        Tries to find a partially compatible artifact, to be used as a fallback
        during artifact selection if [`Artifact::sort_by_system_compatibility`]
        finds no system-compatible artifacts to use.

        Note that this not is guaranteed to be compatible with the current
        system, the contents of the artifact should be checked before use.
    */
    pub fn find_partially_compatible_fallback(artifacts: impl AsRef<[Self]>) -> Option<Self> {
        Self::sort_by_system_compatibility_inner(artifacts, true)
            .into_iter()
            .next()
    }

    fn sort_by_system_compatibility_inner(
        artifacts: impl AsRef<[Self]>,
        allow_partial_compatibility: bool,
    ) -> Vec<Self> {
        let current_desc = Descriptor::current_system();

        let mut compatible_artifacts = artifacts
            .as_ref()
            .iter()
            .filter_map(|artifact| {
                let name = artifact.name.as_deref()?;
                if let Some(asset_desc) = Descriptor::detect(name) {
                    let is_fully_compatible = current_desc.is_compatible_with(&asset_desc);
                    let is_os_compatible = current_desc.os() == asset_desc.os();
                    if is_fully_compatible || (allow_partial_compatibility && is_os_compatible) {
                        Some((asset_desc, artifact))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        compatible_artifacts.sort_by(|(desc_a, artifact_a), (desc_b, artifact_b)| {
            current_desc
                .sort_by_preferred_compat(desc_a, desc_b)
                .then_with(|| sort_preferred_artifact(artifact_a, artifact_b))
                .then_with(|| sort_preferred_formats(artifact_a, artifact_b))
        });

        compatible_artifacts
            .into_iter()
            .map(|(_, artifact)| artifact.clone())
            .collect()
    }
}
