use std::env::var;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::fs::create_dir_all;

use crate::manifests::AuthManifest;
use crate::result::{RokitError, RokitResult};
use crate::sources::ArtifactSource;

use super::{ToolCache, ToolStorage};

/**
    Rokit's home directory - this is where Rokit stores its
    configuration, tools, and other data. Can be cheaply cloned
    while still referring to the same underlying data.

    By default, this is `$HOME/.rokit`, but can be overridden
    by setting the `ROKIT_ROOT` environment variable.
*/
#[derive(Debug, Clone)]
pub struct Home {
    path: Arc<Path>,
    tool_storage: ToolStorage,
    tool_cache: ToolCache,
}

impl Home {
    /**
        Creates a new `Home` from the given path.
    */
    async fn load_from_path(path: impl Into<PathBuf>) -> RokitResult<Self> {
        let path: Arc<Path> = path.into().into();

        let (tool_storage, tool_cache) =
            tokio::try_join!(ToolStorage::load(&path), ToolCache::load(&path))?;

        Ok(Self {
            path,
            tool_storage,
            tool_cache,
        })
    }

    /**
        Creates a new `Home` from the environment.

        This will read, and if necessary, create the Rokit home directory
        and its contents - including trust storage, tools storage, etc.

        If the `ROKIT_ROOT` environment variable is set, this will use
        that as the home directory. Otherwise, it will use `$HOME/.rokit`.

        # Errors

        - If the home directory could not be read or created.
    */
    pub async fn load_from_env() -> RokitResult<Self> {
        if let Ok(root_str) = var("ROKIT_ROOT") {
            Self::load_from_path(root_str).await
        } else {
            let path = dirs::home_dir()
                .ok_or(RokitError::HomeNotFound)?
                .join(".rokit");
            create_dir_all(&path).await?;
            Self::load_from_path(path).await
        }
    }

    /**
        Gets a reference to the path for this `Home`.
    */
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /**
        Returns a reference to the `ToolStorage` for this `Home`.
    */
    #[must_use]
    pub fn tool_storage(&self) -> &ToolStorage {
        &self.tool_storage
    }

    /**
        Returns a reference to the `ToolCache` for this `Home`.
    */
    #[must_use]
    pub fn tool_cache(&self) -> &ToolCache {
        &self.tool_cache
    }

    /**
        Creates a new `ArtifactSource` for this `Home`.

        This will load any stored authentication from disk and use
        it to authenticate with the artifact source and various providers.

        # Errors

        - If the auth manifest could not be loaded or created.
        - If the artifact source could not be created.
    */
    pub async fn artifact_source(&self) -> RokitResult<ArtifactSource> {
        let auth = AuthManifest::load_or_create(&self.path).await?;
        ArtifactSource::new_authenticated(&auth.get_all_tokens())
    }

    /**
        Saves the contents of this `Home` to disk.

        # Errors

        - If the contents could not be saved to disk.
    */
    pub async fn save(&self) -> RokitResult<()> {
        self.tool_cache.save(&self.path).await?;
        Ok(())
    }
}

/*
    Implement Drop with an error message if the Home was dropped
    without being saved - this should never happen since a Home
    should always be loaded once on startup and saved on shutdown
    in the CLI, but this detail may be missed during refactoring.

    In the future, if AsyncDrop ever becomes a thing, we can just
    force the save to happen in the Drop implementation instead.
*/
impl Drop for Home {
    fn drop(&mut self) {
        let is_last = Arc::strong_count(&self.path) <= 1;
        if !is_last {
            return;
        }
        if self.tool_cache.needs_saving() || self.tool_storage.needs_saving() {
            tracing::error!(
                "Rokit home was dropped without saving!\
                \nChanges to trust, tools, and more may have been lost."
            );
        }
    }
}
