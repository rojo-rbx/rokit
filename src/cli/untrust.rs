use anyhow::{bail, Result};
use clap::Parser;

use aftman::{storage::Home, tool::ToolId};

/// Mark the given tool(s) as no longer trusted.
#[derive(Debug, Parser)]
pub struct UntrustSubcommand {
    /// The tool(s) to mark as no longer trusted.
    pub tools: Vec<ToolId>,
}

impl UntrustSubcommand {
    pub async fn run(self, home: &Home) -> Result<()> {
        if self.tools.is_empty() {
            bail!("Please provide at least one tool to remove trust for.");
        }

        let trust_storage = home.trust();
        let (removed_tools, existing_tools) = self
            .tools
            .into_iter()
            .partition::<Vec<_>, _>(|tool| trust_storage.remove_tool(tool));

        if !removed_tools.is_empty() {
            println!("The following tools are no longer trusted:");
            for tool in removed_tools {
                println!("  - {tool}");
            }
        }
        if !existing_tools.is_empty() {
            println!("The following tools were not trusted and have not changed:");
            for tool in existing_tools {
                println!("  - {tool}");
            }
        }

        Ok(())
    }
}
