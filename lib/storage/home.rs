use std::env::var;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use super::{InstallCache, StorageError, StorageResult, TrustCache};

/**
    Aftman's home directory - this is where Aftman stores its
    configuration, tools, and other data. Can be cheaply cloned
    while still referring to the same underlying data.

    By default, this is `$HOME/.aftman`, but can be overridden
    by setting the `AFTMAN_ROOT` environment variable.
*/
#[derive(Debug, Clone)]
pub struct Home {
    path: Arc<Path>,
    saved: Arc<AtomicBool>,
    trust_cache: TrustCache,
    install_cache: InstallCache,
}

impl Home {
    /**
        Creates a new `Home` from the given path.
    */
    async fn load_from_path(path: impl Into<PathBuf>) -> StorageResult<Self> {
        let path: Arc<Path> = path.into().into();
        let saved = Arc::new(AtomicBool::new(false));

        let (trust_cache, install_cache) =
            tokio::try_join!(TrustCache::load(&path), InstallCache::load(&path))?;

        Ok(Self {
            path,
            saved,
            trust_cache,
            install_cache,
        })
    }

    /**
        Creates a new `Home` from the environment.

        This will read, and if necessary, create the Aftman home directory
        and its contents - including trust storage, tools storage, etc.

        If the `AFTMAN_ROOT` environment variable is set, this will use
        that as the home directory. Otherwise, it will use `$HOME/.aftman`.
    */
    pub async fn load_from_env() -> StorageResult<Self> {
        Ok(match var("AFTMAN_ROOT") {
            Ok(root_str) => Self::load_from_path(root_str).await?,
            Err(_) => {
                let path = dirs::home_dir()
                    .ok_or(StorageError::HomeNotFound)?
                    .join(".aftman");

                Self::load_from_path(path).await?
            }
        })
    }

    /**
        Returns a reference to the `TrustCache` for this `Home`.
    */
    pub fn trust_cache(&self) -> &TrustCache {
        &self.trust_cache
    }

    /**
        Returns a reference to the `InstallCache` for this `Home`.
    */
    pub fn install_cache(&self) -> &InstallCache {
        &self.install_cache
    }

    /**
        Saves the contents of this `Home` to disk.
    */
    pub async fn save(&self) -> StorageResult<()> {
        tokio::try_join!(
            self.trust_cache.save(&self.path),
            self.install_cache.save(&self.path)
        )?;
        self.saved.store(true, Ordering::SeqCst);
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
        if is_last && !self.saved.load(Ordering::SeqCst) {
            tracing::error!(
                "Aftman home was dropped without being saved!\
                \nChanges to trust, tools, and more may have been lost."
            )
        }
    }
}
