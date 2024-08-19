use anyhow::{bail, Result};
use clap::Parser;
use console::style;
use rokit::{
    discovery::{discover_all_manifests, discover_tool_spec},
    manifests::RokitManifest,
    storage::Home,
    tool::ToolAlias,
};
use tokio::fs::{read_dir, remove_dir, remove_dir_all, remove_file};

use crate::util::{CliProgressTracker, ToolAliasOrId};

/// Removes a tool from Rokit and uninstalls it.
#[derive(Debug, Parser)]
pub struct UninstallSubcommand {
    /// The tool alias or identifier to uninstall.
    pub tool: ToolAliasOrId,
}

impl UninstallSubcommand {
    pub async fn run(self, home: &Home) -> Result<()> {
        let tool_storage = home.tool_storage();
        let tool_cache = home.tool_cache();

        let alias: ToolAlias = match self.tool {
            ToolAliasOrId::Alias(alias) => alias,
            ToolAliasOrId::Id(id) => id.into(),
        };
        let Some(spec) = discover_tool_spec(&alias, true, false).await else {
            bail!("Failed to find tool '{alias}' in any project manifest file.")
        };

        // 1. Remove the tool from all manifests that contain it
        let pt = CliProgressTracker::new_with_message("Uninstalling", 1);
        let manifests = discover_all_manifests(true, false).await;
        for manifest in manifests {
            let manifest_path = manifest.path.parent().unwrap();
            let mut manifest = RokitManifest::load(&manifest_path).await?;
            if manifest.has_tool(&alias) {
                manifest.remove_tool(&alias);
                manifest.save(&manifest_path).await?;
            }
        }

        // 2. Uninstall the tool binary and remove it from the install cache
        let tool_path = tool_storage.tool_path(&spec);
        let tool_dir = tool_path.ancestors().nth(2).unwrap();
        let author_dir = tool_dir.parent().unwrap();

        remove_file(tool_storage.alias_path(&alias)).await?;
        remove_dir_all(tool_dir).await?;
        if read_dir(&author_dir).await?.next_entry().await?.is_none() {
            remove_dir(author_dir).await?;
        }

        let _ = tool_cache.remove_installed(&spec);

        // 3. Finally, display a nice message to the user
        pt.finish_with_message(format!(
            "Uninstalled tool {} {}",
            style(spec.name()).bold().magenta(),
            pt.formatted_elapsed()
        ));

        Ok(())
    }
}
