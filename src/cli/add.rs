use anyhow::{bail, Context, Result};
use clap::Parser;
use console::style;

use rokit::{
    discovery::discover_all_manifests,
    manifests::RokitManifest,
    storage::Home,
    tool::{ToolAlias, ToolId},
};

use crate::util::{
    find_most_compatible_artifact, prompt_for_trust, CliProgressTracker, ToolIdOrSpec,
};

/// Adds a new tool to Rokit and installs it.
#[derive(Debug, Parser)]
pub struct AddSubcommand {
    /// A tool identifier or specification describing where
    /// to get the tool, and optionally what version to install.
    pub tool: ToolIdOrSpec,
    /// The name that will be used to run the tool.
    pub alias: Option<ToolAlias>,
    /// Add this tool globally instead of adding
    /// it to the nearest manifest file.
    #[clap(long)]
    pub global: bool,
    /// Force add and install the tool, even
    /// if it is already added or installed.
    #[clap(long)]
    pub force: bool,
}

impl AddSubcommand {
    pub async fn run(self, home: &Home) -> Result<()> {
        let id: ToolId = self.tool.clone().into();
        let alias: ToolAlias = match self.alias.as_ref() {
            Some(alias) => alias.clone(),
            None => self.tool.clone().into(),
        };

        let tool_cache = home.tool_cache();
        let tool_storage = home.tool_storage();
        let source = home.artifact_source().await?;

        // 1. Check for trust, or prompt the user to trust the tool
        if !tool_cache.is_trusted(&id) {
            if !self.force && !prompt_for_trust(id.clone()).await? {
                bail!("Tool is not trusted - operation was aborted");
            }
            let _ = tool_cache.add_trust(id.clone());
        }

        // 2. Load manifest and do a preflight check to
        // ensure we don't overwrite any existing tool(s)
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
        if manifest.has_tool(&alias) && !self.force {
            let global_flag = if self.global { "--global " } else { "" };
            bail!(
                "Tool already exists and can't be added: {id}\n\
                \n  - To update the tool, run `rokit update {global_flag}{id}`\
                \n  - To remove the tool, run `rokit remove {global_flag}{id}`"
            );
        }

        // 3. If we only got an id without a specified version, we
        // will fetch the latest non-prerelease release and use that
        let pt = CliProgressTracker::new_with_message("Fetching", 3);
        let (spec, artifact) = match self.tool.clone() {
            ToolIdOrSpec::Spec(spec) => {
                let release_artifact = source.get_specific_release(&spec).await?;
                let artifact = find_most_compatible_artifact(&release_artifact.artifacts, &id)?;
                (spec, artifact)
            }
            ToolIdOrSpec::Id(id) => {
                let release_artifact = source.get_latest_release(&id).await?;
                let artifact = find_most_compatible_artifact(&release_artifact.artifacts, &id)?;
                (artifact.tool_spec.clone(), artifact)
            }
        };
        pt.task_completed();

        // 4. Add the tool spec to the desired manifest file and save it
        manifest.add_tool(&alias, &spec);
        manifest.save(manifest_path).await?;

        // 5. Download and install the tool
        if !tool_cache.is_installed(&spec) || self.force {
            let contents = source
                .download_artifact_contents(&artifact)
                .await
                .with_context(|| format!("Failed to download contents for {spec}"))?;
            pt.task_completed();
            pt.update_message("Installing");
            let extracted = artifact
                .extract_contents(contents)
                .await
                .with_context(|| format!("Failed to extract contents for {spec}"))?;
            tool_storage.replace_tool_contents(&spec, extracted).await?;
            pt.task_completed();
            let _ = tool_cache.add_installed(spec.clone());
        } else {
            pt.task_completed();
            pt.task_completed();
        }

        // 6. Create the tool alias link
        pt.update_message("Linking");
        tool_storage.create_tool_link(&alias).await?;

        // 7. Finally, display a nice message to the user
        pt.finish_with_message(format!(
            "Added version {} of tool {}{} {}",
            style(spec.version()).bold().yellow(),
            style(spec.name()).bold().magenta(),
            if alias.name() == id.name() {
                String::new()
            } else {
                format!(" with alias {}", style(alias.to_string()).bold().cyan())
            },
            pt.formatted_elapsed(),
        ));

        Ok(())
    }
}
