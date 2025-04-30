use std::{
    env::{split_paths, var_os},
    path::MAIN_SEPARATOR_STR,
};

use crate::{result::RokitResult, storage::Home};

#[cfg(unix)]
mod shell;

#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod windows;

/**
    Tries to add the Rokit binaries directory to the system PATH.

    Returns `true` if the directory was added to the PATH, `false` otherwise.

    # Errors

    - If the directory could not be added to the PATH.
*/
pub async fn add_to_path(home: &Home) -> RokitResult<bool> {
    #[cfg(unix)]
    {
        self::unix::add_to_path(home).await
    }
    #[cfg(windows)]
    {
        self::windows::add_to_path(home).await
    }
}

/**
    Checks if the Rokit binaries directory is in the system PATH.

    Returns `true` if the directory is in the PATH, `false` otherwise.
*/
#[must_use]
pub fn exists_in_path(_home: &Home) -> bool {
    let pattern = format!(".rokit{MAIN_SEPARATOR_STR}bin");
    var_os("PATH").is_some_and(|path| split_paths(&path).any(|item| item.ends_with(&pattern)))
}
