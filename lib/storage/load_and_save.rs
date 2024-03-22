use std::path::{Path, MAIN_SEPARATOR};

use const_format::concatcp;
use tokio::fs::{read_to_string, write};

use super::{InstallCache, StorageResult, TrustCache};

const FILE_PATH_TRUST: &str = "trusted.txt";
const FILE_PATH_INSTALLED: &str = concatcp!("tool-storage", MAIN_SEPARATOR, "installed.txt");

impl TrustCache {
    /// Load the trust cache from the given home root path.
    pub(super) async fn load(home_path: impl AsRef<Path>) -> StorageResult<Self> {
        let path = home_path.as_ref().join(FILE_PATH_TRUST);
        match read_to_string(&path).await {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(TrustCache::new()),
            Err(e) => Err(e.into()),
            Ok(s) => Ok(TrustCache::from_str(s)),
        }
    }

    /// Save the trust cache to the given home root path.
    pub(super) async fn save(&self, home_path: impl AsRef<Path>) -> StorageResult<()> {
        let path = home_path.as_ref().join(FILE_PATH_TRUST);
        Ok(write(path, self.to_string()).await?)
    }
}

impl InstallCache {
    /// Load the install cache from the given home root path.
    pub(super) async fn load(home_path: impl AsRef<Path>) -> StorageResult<Self> {
        let path = home_path.as_ref().join(FILE_PATH_INSTALLED);
        match read_to_string(&path).await {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(InstallCache::new()),
            Err(e) => Err(e.into()),
            Ok(s) => Ok(InstallCache::from_str(s)),
        }
    }

    /// Save the install cache to the given home root path.
    pub(super) async fn save(&self, home_path: impl AsRef<Path>) -> StorageResult<()> {
        let path = home_path.as_ref().join(FILE_PATH_INSTALLED);
        Ok(write(path, self.to_string()).await?)
    }
}
