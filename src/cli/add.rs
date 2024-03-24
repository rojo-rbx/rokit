use anyhow::{bail, Context, Result};
use clap::Parser;

use aftman::{
    description::Description,
    manifests::AftmanManifest,
    storage::Home,
    tool::{ToolAlias, ToolId},
};
use tokio::time::Instant;

use crate::util::{
    discover_aftman_manifest_dir, github_tool_source, prompt_for_install_trust, ToolIdOrSpec,
};

/// Adds a new tool to Aftman and installs it.
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
    force: bool,
}

impl AddSubcommand {
    pub async fn run(&self, home: &Home) -> Result<()> {
        let start = Instant::now();

        let id: ToolId = self.tool.clone().into();
        let alias: ToolAlias = match self.alias.as_ref() {
            Some(alias) => alias.clone(),
            None => self.tool.clone().into(),
        };

        let manifest_path = if self.global {
            home.path().to_path_buf()
        } else {
            discover_aftman_manifest_dir().await?
        };

        let source = github_tool_source(home).await?;

        // Check for trust, or prompt the user to trust the tool
        let trust_cache = home.trust_cache();
        if !trust_cache.is_trusted(&id) {
            if !self.force && !prompt_for_install_trust(&id).await? {
                bail!("Tool is not trusted - installation was aborted");
            }
            trust_cache.add_tool(id.clone());
        }

        // Load manifest and do a preflight check to make sure we don't overwrite any tool
        let mut manifest = if self.global {
            AftmanManifest::load_or_create(&manifest_path).await?
        } else {
            AftmanManifest::load(&manifest_path).await?
        };
        if manifest.has_tool(&alias) && !self.force {
            let global_flag = if self.global { "--global " } else { "" };
            bail!(
                "Tool already exists and can't be added: {id}\n\
                \n  - To update the tool, run `aftman update {global_flag}{id}`\
                \n  - To remove the tool, run `aftman remove {global_flag}{id}`"
            );
        }

        // If we only got an id without a specified version, we
        // will fetch the latest non-prerelease release and use that
        let spec = match self.tool.clone() {
            ToolIdOrSpec::Spec(spec) => spec,
            ToolIdOrSpec::Id(id) => {
                tracing::info!("Looking for the latest version of {id}...");
                let version = source
                    .find_latest_version(&id, false)
                    .await?
                    .with_context(|| format!("Failed to find latest release for {id}"))?;
                id.into_spec(version)
            }
        };

        // Add the tool spec to the desired manifest file and save it
        manifest.add_tool(&alias, &spec);
        manifest.save(manifest_path).await?;
        tracing::info!("Added tool successfully: {spec}");

        // Install the tool and create the link for its alias
        let description = Description::current();
        let install_cache = home.install_cache();
        let tool_storage = home.tool_storage();
        if !install_cache.is_installed(&spec) && !self.force {
            tracing::info!("Downloading {spec}");
            let release = source
                .find_release(&spec)
                .await?
                .with_context(|| format!("Failed to find release for {spec}"))?;
            let artifact = source
                .find_compatible_artifacts(&spec, &release, &description)
                .first()
                .cloned()
                .with_context(|| format!("No compatible artifact found for {spec}"))?;
            let contents = source
                .download_artifact_contents(&artifact)
                .await
                .with_context(|| format!("Failed to download contents for {spec}"))?;

            tracing::info!("Installing {spec}");
            let extracted = artifact
                .extract_contents(contents)
                .await
                .with_context(|| format!("Failed to extract contents for {spec}"))?;
            tool_storage.replace_tool_contents(&spec, extracted).await?;

            install_cache.add_spec(spec.clone());
        }

        tool_storage.create_tool_link(&alias).await?;
        tracing::info!("Completed in {:.2?}", start.elapsed());

        Ok(())
    }
}
