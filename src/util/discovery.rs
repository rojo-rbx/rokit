use std::path::PathBuf;

use anyhow::{Context, Result};

use aftman::{
    manifests::{discover_file_recursive, discover_files_recursive, AFTMAN_MANIFEST_FILE_NAME},
    storage::Home,
};

pub async fn discover_aftman_manifest_dir() -> Result<PathBuf> {
    let file_path = discover_file_recursive(AFTMAN_MANIFEST_FILE_NAME)
        .await?
        .context(
            "No manifest was found for the current directory.\
            \nRun `aftman init` in your project root to create one.",
        )?;
    let dir_path = file_path
        .parent()
        .context("Invalid file path returned during manifest discovery")?;
    Ok(dir_path.to_path_buf())
}

pub async fn discover_aftman_manifest_dirs(home: &Home) -> Result<Vec<PathBuf>> {
    let mut dirs = vec![home.path().to_path_buf()];

    let file_paths = discover_files_recursive(AFTMAN_MANIFEST_FILE_NAME).await?;
    for file_path in file_paths {
        let dir_path = file_path
            .parent()
            .context("Invalid file path returned during manifest discovery")?;
        dirs.push(dir_path.to_path_buf());
    }

    Ok(dirs)
}
