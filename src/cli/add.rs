use anyhow::{Context, Result};
use clap::Parser;

use aftman::{sources::GitHubSource, storage::Home, tool::ToolAlias};
use semver::Version;

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
        let source = GitHubSource::new()?;

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

        // Fetch the release for the tool
        let _release = source
            .find_release(&spec)
            .await?
            .with_context(|| format!("No release was found for {spec}"))?;

        // TODO: Add the tool spec to the desired manifest file, and install it

        Ok(())
    }
}
