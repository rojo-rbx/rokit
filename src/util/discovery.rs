use std::path::PathBuf;

use aftman::{manifests::AFTMAN_MANIFEST_FILE_NAME, system::discover_file_recursive};
use anyhow::{Context, Result};

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
