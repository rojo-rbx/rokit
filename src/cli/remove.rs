use anyhow::{bail, Context, Result};
use clap::Parser;
use console::style;

use rokit::{
    discovery::discover_all_manifests, manifests::RokitManifest, storage::Home, tool::ToolAlias,
};

use crate::util::CliProgressTracker;

/// Removes a tool from Rokit.
#[derive(Debug, Parser)]
pub struct RemoveSubcommand {
    /// The alias of the tool to remove.
    pub alias: ToolAlias,
    /// Remove this tool globally instead of removing
    /// it from the nearest manifest file.
    #[clap(long)]
    pub global: bool,
}

impl RemoveSubcommand {
    pub async fn run(self, home: &Home) -> Result<()> {
        let tool_cache = home.tool_cache();
        let tool_storage = home.tool_storage();

        // 1. Load the manifest and check whether the tool
        // to be removed is present in the manifest
        let manifest_path = if self.global {
            home.path().to_path_buf()
        } else {
            let non_global_manifests = discover_all_manifests(true, true).await;
            non_global_manifests
                .first()
                .map(|m| m.path.parent().unwrap().to_path_buf())
                .context(
                    "No manifest was found for the current directory.\
                    \nRun `rokit init` in your project root to create one.",
                )?
        };

        let mut manifest = if self.global {
            RokitManifest::load_or_create(&manifest_path).await?
        } else {
            RokitManifest::load(&manifest_path).await?
        };
        if !manifest.has_tool(&self.alias) {
            bail!("Tool does not exist and can't be removed: {}", self.alias);
        }

        // 2. Remove the tool from the manifest
        let spec = manifest.get_tool(&self.alias).unwrap();
        let pt = CliProgressTracker::new_with_message("Removing", 2);

        manifest.remove_tool(&self.alias);
        manifest.save(manifest_path).await?;
        pt.task_completed();

        // 3. Uninstall the tool link
        if tool_cache.is_installed(&spec) {
            pt.update_message("Uninstalling");
            tool_storage.remove_tool_link(&self.alias).await?;
        }
        pt.task_completed();

        // 3. Finally, display a nice message to the user
        pt.finish_with_message(format!(
            "Removed version {} of tool {}{} {}",
            style(spec.version()).bold().yellow(),
            style(spec.name()).bold().magenta(),
            if self.alias.name() == spec.id().name() {
                String::new()
            } else {
                format!(
                    " with alias {}",
                    style(self.alias.to_string()).bold().cyan()
                )
            },
            pt.formatted_elapsed(),
        ));

        Ok(())
    }
}
