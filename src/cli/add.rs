use anyhow::Result;
use clap::Parser;

use aftman::{storage::Home, tool::ToolAlias};

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
        // TODO: Implement the add subcommand

        Ok(())
    }
}
