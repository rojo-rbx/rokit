use anyhow::{bail, Context, Result};
use clap::Parser;

use aftman::{
    manifests::{AftmanManifest, AuthManifest},
    sources::{ArtifactProvider, GitHubSource},
    storage::Home,
    tool::{ToolAlias, ToolId},
};

use crate::util::{discover_aftman_manifest_dir, ToolIdOrSpec};

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
}

impl AddSubcommand {
    pub async fn run(&self, home: &Home) -> Result<()> {
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

        // We might be wanting to add a private tool, so load our tool source with auth
        // FUTURE: Some kind of generic solution for tool sources and auth for them
        let auth = AuthManifest::load_or_create(home.path()).await?;
        let source = match auth.get_token(ArtifactProvider::GitHub) {
            Some(token) => GitHubSource::new_authenticated(token)?,
            None => GitHubSource::new()?,
        };

        // Load manifest and do a preflight check to make sure we don't overwrite any tool
        let mut manifest = if self.global {
            AftmanManifest::load_or_create(&manifest_path).await?
        } else {
            AftmanManifest::load(&manifest_path).await?
        };
        if manifest.has_tool(&alias) {
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
                tracing::info!("Looking for latest version of {id}...");
                let version = source
                    .find_latest_version(&id, false)
                    .await?
                    .with_context(|| format!("Failed to find latest release for {id}"))?;
                id.into_spec(version)
            }
        };

        // Add the tool spec to the desired manifest file and save it
        manifest.add_tool(&alias, &spec);
        manifest.save(home.path()).await?;
        tracing::info!("Added tool successfully: {id}");

        // TODO: Install the tool

        Ok(())
    }
}
