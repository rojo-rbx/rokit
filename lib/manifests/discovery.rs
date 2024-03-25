use std::{env::current_dir, path::PathBuf};

use tokio::task::spawn_blocking;

use crate::result::RokitResult;

/**
    Discovers the given file in the current directory or any of its parents.

    See [`discover_files_recursive`] for more information.
*/
pub async fn discover_file_recursive(
    file_name: impl Into<PathBuf>,
) -> RokitResult<Option<PathBuf>> {
    let file_name = file_name.into();
    spawn_blocking(move || {
        let cwd = current_dir()?;
        let mut current_dir = cwd.as_path();
        loop {
            let file_path = current_dir.join(&file_name);
            if file_path.is_file() {
                return Ok(Some(file_path));
            }

            match current_dir.parent() {
                Some(parent) => current_dir = parent,
                None => break,
            }
        }
        Ok(None)
    })
    .await?
}

/**
    Discovers all files with the given name in the current directory or any of its parents.

    This function will search for files with the given name in the current directory
    and all of its parents. If any files are found, their paths will be returned.

    # Errors

    - If the current directory could not be determined
    - If an I/O error occurred while searching for the files
*/
pub async fn discover_files_recursive(file_name: impl Into<PathBuf>) -> RokitResult<Vec<PathBuf>> {
    let file_name = file_name.into();
    spawn_blocking(move || {
        let cwd = current_dir()?;
        let mut current_dir = cwd.as_path();
        let mut files = Vec::new();
        loop {
            let file_path = current_dir.join(&file_name);
            if file_path.is_file() {
                files.push(file_path);
            }

            match current_dir.parent() {
                Some(parent) => current_dir = parent,
                None => break,
            }
        }
        Ok(files)
    })
    .await?
}
