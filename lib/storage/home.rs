use std::env::var;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::fs::read_to_string;

use super::{StorageError, StorageResult, TrustStorage};

/**
    Aftman's home directory.

    This is where Aftman stores its configuration, tools, and other data.

    By default, this is `$HOME/.aftman`, but can be overridden
    by setting the `AFTMAN_ROOT` environment variable.
*/
#[derive(Debug, Clone)]
pub struct Home {
    path: Arc<Path>,
}

impl Home {
    /**
        Creates a new `Home` from the given path.
    */
    fn from_path(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into().into(),
        }
    }

    /**
        Creates a new `Home` from the environment.

        If the `AFTMAN_ROOT` environment variable is set, this will use
        that as the home directory. Otherwise, it will use `$HOME/.aftman`.
    */
    pub fn from_env() -> StorageResult<Self> {
        Ok(match var("AFTMAN_ROOT") {
            Ok(root_str) => Self::from_path(root_str),
            Err(_) => {
                let path = dirs::home_dir()
                    .ok_or(StorageError::HomeNotFound)?
                    .join(".aftman");

                Self::from_path(path)
            }
        })
    }

    /**
        Reads the trust storage for this `Home`.

        This function will return an error if the trust storage file
        cannot be read - if it does not exist, it will be created.
    */
    pub async fn trust_storage(&self) -> StorageResult<TrustStorage> {
        let path = self.path.join("trusted.txt");
        match read_to_string(&path).await {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(TrustStorage::new()),
            Err(e) => Err(e.into()),
            Ok(s) => Ok(TrustStorage::from_str(s)),
        }
    }
}
