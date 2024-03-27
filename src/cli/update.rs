use anyhow::{bail, Result};
use clap::Parser;

use rokit::{storage::Home, tool::ToolAlias};

/// Updates all tools, or specific tools, to the latest version.
#[derive(Debug, Parser)]
pub struct UpdateSubcommand {
    /// The aliases of the tools to update. Omit to update all tools.
    pub tools: Vec<ToolAlias>,
    /// Update tools globally instead of using the nearest manifest file.
    #[clap(long)]
    pub global: bool,
}

impl UpdateSubcommand {
    pub async fn run(self, _home: &Home) -> Result<()> {
        bail!("Command is not yet implemented")
    }
}
