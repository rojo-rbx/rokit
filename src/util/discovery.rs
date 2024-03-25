use std::{env::current_dir, path::PathBuf};

use anyhow::{Context, Result};

use aftman::{
    manifests::{
        discover_file_recursive, discover_files_recursive, AftmanManifest,
        AFTMAN_MANIFEST_FILE_NAME,
    },
    storage::Home,
    tool::{ToolAlias, ToolSpec},
};
use futures::{stream::FuturesUnordered, TryStreamExt};
use tokio::task::spawn_blocking;

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

pub async fn discover_closest_tool_spec(home: &Home, alias: &ToolAlias) -> Result<ToolSpec> {
    let cwd = spawn_blocking(current_dir)
        .await?
        .context("Failed to get current working directory")?;

    let dirs = discover_aftman_manifest_dirs(home).await?;
    let manifests = dirs
        .iter()
        .map(|dir| async move {
            let manifest = AftmanManifest::load(&dir)
                .await
                .with_context(|| format!("Failed to load manifest at {}", dir.display()))?;
            anyhow::Ok((dir, manifest))
        })
        .collect::<FuturesUnordered<_>>()
        .try_collect::<Vec<_>>()
        .await?;

    let specs = manifests
        .iter()
        .flat_map(|(dir, manifest)| {
            let spec = manifest.get_tool(alias)?;
            Some((*dir, spec))
        })
        .collect::<Vec<_>>();
    let (_, closest_spec) = specs
        .iter()
        .min_by_key(|(dir, _)| {
            dir.strip_prefix(&cwd)
                .unwrap_or_else(|_| dir)
                .components()
                .count()
        })
        .context("No tool spec found for the given alias")?;

    Ok(closest_spec.clone())
}
