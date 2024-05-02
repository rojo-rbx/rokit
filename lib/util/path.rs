#![allow(dead_code)] // Some path utilities are only used on Windows

use std::path::{Path, PathBuf};

/**
    Cleans up a path and simplifies it for writing to storage or environment variables.

    This will currently:

    - De-UNC a path, removing the `\\?\` prefix
*/
pub fn simplify_path(path: impl AsRef<Path>) -> PathBuf {
    dunce::simplified(path.as_ref()).to_path_buf()
}
