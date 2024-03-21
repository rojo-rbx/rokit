use std::env;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::sync::Arc;

use anyhow::format_err;

use crate::dirs::home_dir;

/// Defines the root that everything else in Aftman is stored relative to.
///
/// This type encourages good organization, helps us behave predictably, and
/// enables better tests for Aftman.
#[derive(Debug, Clone)]
pub struct Home {
    path: Arc<Path>,

    #[cfg(test)]
    #[allow(unused)]
    temp: Option<Arc<tempfile::TempDir>>,
}

impl Home {
    pub fn from_env() -> anyhow::Result<Self> {
        // Users can override the Aftman home directory via the AFTMAN_ROOT
        // environment variable.
        if let Ok(var) = env::var("AFTMAN_ROOT") {
            return Ok(Self::from_path(var));
        }

        let mut path =
            home_dir().ok_or_else(|| format_err!("Home directory could not be found."))?;

        path.push(".aftman");

        Ok(Self::from_path(path))
    }

    #[cfg(test)]
    pub fn new_temp() -> anyhow::Result<Self> {
        let temp = tempfile::TempDir::new()?;

        Ok(Self {
            path: temp.path().to_path_buf().into(),
            temp: Some(Arc::new(temp)),
        })
    }

    fn from_path<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            path: path.into().into(),

            #[cfg(test)]
            temp: None,
        }
    }

    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }

    pub fn path_str(&self) -> String {
        rehomeify(&self.path)
    }

    pub fn bin_dir(&self) -> PathBuf {
        self.path.join("bin")
    }

    pub fn bin_dir_str(&self) -> String {
        rehomeify(&self.bin_dir())
    }
}

/// Returns a human-friendly version of `path`, re-substituting `$HOME` if
/// it is present in the path.
fn rehomeify(path: &Path) -> String {
    if let Ok(home) = env::var("HOME") {
        let prefix = Path::new(&home);
        if let Ok(stripped) = path.strip_prefix(prefix) {
            let rest = stripped.to_str().unwrap();

            return format!("$HOME{MAIN_SEPARATOR}{rest}");
        }
    }

    path.to_str().unwrap().to_owned()
}
