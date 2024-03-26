use std::collections::HashMap;

use crate::{
    result::RokitResult,
    tool::{ToolId, ToolSpec},
};

use super::{github::GithubProvider, Artifact, ArtifactProvider};

/**
    A source for artifacts.

    Provides high-level access abstracting over individual providers such as GitHub, ...
*/
#[derive(Debug, Clone)]
pub struct ArtifactSource {
    github: GithubProvider,
}

impl ArtifactSource {
    /**
        Creates a new artifact source.

        This source is unauthenticated and may be rate limited and/or unable to access
        private resources. To authenticate using auth tokens, use `new_authenticated`.
    */
    pub fn new() -> RokitResult<Self> {
        let github = GithubProvider::new()?;
        Ok(Self { github })
    }

    /**
        Creates a new authenticated artifact source.
    */
    pub fn new_authenticated(auth: &HashMap<ArtifactProvider, String>) -> RokitResult<Self> {
        let github = match auth.get(&ArtifactProvider::GitHub) {
            Some(token) => GithubProvider::new_authenticated(token)?,
            None => GithubProvider::new()?,
        };
        Ok(Self { github })
    }

    /**
        Gets the latest release for a tool.
    */
    pub async fn get_latest_release(
        &self,
        provider: ArtifactProvider,
        id: &ToolId,
    ) -> RokitResult<Vec<Artifact>> {
        Ok(match provider {
            ArtifactProvider::GitHub => self.github.get_latest_release(id).await?,
        })
    }

    /**
        Gets a specific release for a tool.
    */
    pub async fn get_specific_release(
        &self,
        provider: ArtifactProvider,
        spec: &ToolSpec,
    ) -> RokitResult<Vec<Artifact>> {
        Ok(match provider {
            ArtifactProvider::GitHub => self.github.get_specific_release(spec).await?,
        })
    }

    /**
        Downloads the contents of an artifact.
    */
    pub async fn download_artifact_contents(&self, artifact: &Artifact) -> RokitResult<Vec<u8>> {
        Ok(match &artifact.provider {
            ArtifactProvider::GitHub => self.github.download_artifact_contents(artifact).await?,
        })
    }
}
