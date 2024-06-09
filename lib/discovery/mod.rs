use std::{
    collections::HashMap,
    env::var_os,
    path::{Path, PathBuf},
};

use futures::{stream::FuturesOrdered, StreamExt};
use tokio::fs::read_to_string;

use crate::{
    manifests::RokitManifest,
    storage::Home,
    system::current_dir,
    tool::{ToolAlias, ToolSpec},
};

use self::{aftman::AftmanManifest, foreman::ForemanManifest};

mod aftman;
mod foreman;
mod rokit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ManifestKind {
    Foreman,
    Aftman,
    Rokit,
}

trait Manifest
where
    Self: Sized,
{
    fn home_dir() -> &'static str;
    fn manifest_file_name() -> &'static str;
    fn parse_manifest(contents: &str) -> Option<Self>;
    fn into_tools(self) -> HashMap<ToolAlias, ToolSpec>;
}

/**
    A discovered manifest.

    Contains tools as well as the path where the manifest was found.
*/
#[derive(Debug, Clone)]
pub struct DiscoveredManifest {
    _kind: ManifestKind,
    pub path: PathBuf,
    pub tools: HashMap<ToolAlias, ToolSpec>,
}

fn search_paths(cwd: &Path, rokit_only: bool, skip_home: bool) -> Vec<(ManifestKind, PathBuf)> {
    let mut ordered_paths = Vec::new();

    // Gather paths from current directory and up
    let mut current = Some(cwd);
    while let Some(dir) = current {
        ordered_paths.push((
            ManifestKind::Rokit,
            dir.join(RokitManifest::manifest_file_name()),
        ));
        if !rokit_only {
            ordered_paths.push((
                ManifestKind::Aftman,
                dir.join(AftmanManifest::manifest_file_name()),
            ));
            ordered_paths.push((
                ManifestKind::Foreman,
                dir.join(ForemanManifest::manifest_file_name()),
            ));
        }
        current = dir.parent();
    }

    // Gather paths from program-specific home directories, if desired
    if !skip_home {
        if let Some(home) = dirs::home_dir() {
            ordered_paths.push((
                ManifestKind::Rokit,
                home.join(RokitManifest::home_dir())
                    .join(RokitManifest::manifest_file_name()),
            ));
            if !rokit_only {
                ordered_paths.push((
                    ManifestKind::Aftman,
                    home.join(AftmanManifest::home_dir())
                        .join(AftmanManifest::manifest_file_name()),
                ));
                ordered_paths.push((
                    ManifestKind::Foreman,
                    home.join(ForemanManifest::home_dir())
                        .join(ForemanManifest::manifest_file_name()),
                ));
            }
        }
    }

    ordered_paths
}

/**
    Discovers all known tool manifests in the current directory and its ancestors, as well as home directories.

    This is a slow operation that reads many potential files - use `discover_tool_spec` if possible.
*/
pub async fn discover_all_manifests(rokit_only: bool, skip_home: bool) -> Vec<DiscoveredManifest> {
    let cwd = current_dir().await;

    let found_manifest_contents = search_paths(&cwd, rokit_only, skip_home)
        .into_iter()
        .map(|(kind, path)| async move {
            let contents = read_to_string(&path).await.ok()?;
            Some((kind, path, contents))
        })
        .collect::<FuturesOrdered<_>>()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    found_manifest_contents
        .into_iter()
        .filter_map(|(kind, path, contents)| {
            let tools = match kind {
                ManifestKind::Rokit => RokitManifest::parse_manifest(&contents)?.into_tools(),
                ManifestKind::Aftman => AftmanManifest::parse_manifest(&contents)?.into_tools(),
                ManifestKind::Foreman => ForemanManifest::parse_manifest(&contents)?.into_tools(),
            };
            Some(DiscoveredManifest {
                _kind: kind,
                path,
                tools,
            })
        })
        .collect()
}

/**
    Discovers a tool spec by searching for manifests in the current directory and its ancestors.

    This is a fast operation that reads only the necessary files.
*/
pub async fn discover_tool_spec(
    alias: &ToolAlias,
    rokit_only: bool,
    skip_home: bool,
) -> Option<ToolSpec> {
    let cwd = current_dir().await;

    for (kind, path) in search_paths(&cwd, rokit_only, skip_home) {
        let Ok(contents) = read_to_string(&path).await else {
            continue;
        };

        let tools = match kind {
            ManifestKind::Rokit => RokitManifest::parse_manifest(&contents)?.into_tools(),
            ManifestKind::Aftman => AftmanManifest::parse_manifest(&contents)?.into_tools(),
            ManifestKind::Foreman => ForemanManifest::parse_manifest(&contents)?.into_tools(),
        };

        if let Some(spec) = tools.get(alias) {
            return Some(spec.clone());
        }
    }

    None
}

/**
    Discovers a tool explicitly **not** managed by Rokit,
    by traversing the system PATH environment variable.

    This means that if Rokit manages a tool with the given alias,
    and an executable for it is in the PATH, this function will
    ignore that and prefer other tools on the system instead.
*/
pub async fn discover_non_rokit_tool(home: &Home, alias: &ToolAlias) -> Option<PathBuf> {
    let cwd = current_dir().await;

    let binary_name = alias.name().to_string();
    let home_path = home.path().to_owned();
    let search_paths = var_os("PATH")?;

    let mut found_tool_paths = which::which_in_all(binary_name, Some(search_paths), &cwd)
        .ok()
        .into_iter()
        .flatten()
        .filter(|path| !path.starts_with(&home_path));

    found_tool_paths.next()
}
