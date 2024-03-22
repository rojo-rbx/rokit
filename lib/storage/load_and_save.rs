use std::path::Path;

use tokio::fs::{read_to_string, write};

use super::{StorageResult, TrustStorage};

const FILE_PATH_TRUST: &str = "trusted.txt";

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
