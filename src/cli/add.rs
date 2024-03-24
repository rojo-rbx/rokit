use std::env::current_dir;

use anyhow::{bail, Context, Result};
use clap::Parser;

use aftman::{
    manifests::{AftmanManifest, AuthManifest},
    sources::{ArtifactProvider, GitHubSource},
    storage::Home,
    tool::ToolAlias,
};

use crate::util::ToolIdOrSpec;

/// Adds a new tool to Aftman and installs it.
#[derive(Debug, Parser)]
pub struct AddSubcommand {
    /// A tool identifier or specification describing where
    /// to get the tool and what version to install.
    pub tool_spec: ToolIdOrSpec,

    /// The name that will be used to run the tool.
    pub tool_alias: Option<ToolAlias>,

    /// Install this tool globally by adding it to ~/.aftman/aftman.toml
    /// instead of installing it to the nearest aftman.toml file.
    #[clap(long)]
    pub global: bool,
}

impl AddSubcommand {
    pub async fn run(&self, home: &Home) -> Result<()> {
        // We might be wanting to add a private tool, so load our tool source with auth
        // FUTURE: Some kind of generic solution for tool sources and auth for them
        let auth = AuthManifest::load_or_create(home.path()).await?;
        let source = match auth.get_token(ArtifactProvider::GitHub) {
            Some(token) => GitHubSource::new_authenticated(token)?,
            None => GitHubSource::new()?,
        };

        // If we only got an id without a specified version, we
        // will fetch the latest no-prerelease release and use that
        let spec = match self.tool_spec.clone() {
            ToolIdOrSpec::Spec(spec) => spec,
            ToolIdOrSpec::Id(id) => {
                let version = source
                    .find_latest_version(&id, false)
                    .await?
                    .with_context(|| format!("No non-prerelease releases were found for {id}"))?;
                id.into_spec(version)
            }
        };
        let id = spec.clone().into_id();
        let alias = self
            .tool_alias
            .clone()
            .unwrap_or_else(|| id.clone().into_alias());

        // Fetch the release for the tool
        let _release = source
            .find_release(&spec)
            .await?
            .with_context(|| format!("No release was found for {spec}"))?;

        // Add the tool spec to the desired manifest file
        if self.global {
            let mut manifest = AftmanManifest::load_or_create(home.path()).await?;
            if manifest.add_tool(&alias, &spec) {
                manifest.save(home.path()).await?;
                tracing::info!("Added tool successfully: {id}");
            } else {
                bail!(
                    "Tool already exists and can't be added: {id}\
                    \n  - To update the tool, run `aftman update --global {id}`\
                    \n  - To remove the tool, run `aftman remove --global {id}`"
                );
            }
        } else {
            let cwd = current_dir()?;
            let mut manifest = AftmanManifest::load(&cwd).await.context(
                "No manifest was found in the current directory.\
                \nRun `aftman init` to initialize a new project.",
            )?;
            if manifest.add_tool(&alias, &spec) {
                manifest.save(cwd).await?;
                tracing::info!("Added tool successfully: {id}");
            } else {
                bail!(
                    "Tool already exists and can't be added: {id}\
                    \n  - To update the tool, run `aftman update {id}`\
                    \n  - To remove the tool, run `aftman remove {id}`"
                );
            }
        }

        // TODO: Install the tool

        Ok(())
    }
}
