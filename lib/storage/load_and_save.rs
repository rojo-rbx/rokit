use std::path::{Path, MAIN_SEPARATOR};

use const_format::concatcp;
use tokio::fs::{read_to_string, write};

use super::{InstalledStorage, StorageResult, TrustStorage};

const FILE_PATH_TRUST: &str = "trusted.txt";
const FILE_PATH_INSTALLED: &str = concatcp!("tool-storage", MAIN_SEPARATOR, "installed.txt");

impl TrustStorage {
    pub(super) async fn load(home_path: impl AsRef<Path>) -> StorageResult<Self> {
        let path = home_path.as_ref().join(FILE_PATH_TRUST);
        match read_to_string(&path).await {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(TrustStorage::new()),
            Err(e) => Err(e.into()),
            Ok(s) => Ok(TrustStorage::from_str(s)),
        }
    }

    pub(super) async fn save(&self, home_path: impl AsRef<Path>) -> StorageResult<()> {
        let path = home_path.as_ref().join(FILE_PATH_TRUST);
        Ok(write(path, self.to_string()).await?)
    }
}

impl InstalledStorage {
    pub(super) async fn load(home_path: impl AsRef<Path>) -> StorageResult<Self> {
        let path = home_path.as_ref().join(FILE_PATH_INSTALLED);
        match read_to_string(&path).await {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(InstalledStorage::new()),
            Err(e) => Err(e.into()),
            Ok(s) => Ok(InstalledStorage::from_str(s)),
        }
    }

    pub(super) async fn save(&self, home_path: impl AsRef<Path>) -> StorageResult<()> {
        let path = home_path.as_ref().join(FILE_PATH_INSTALLED);
        Ok(write(path, self.to_string()).await?)
    }
}
