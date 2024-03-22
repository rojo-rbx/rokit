use anyhow::{bail, Result};
use clap::Parser;

use aftman::{storage::Home, tool::ToolId};

/// Mark the given tool(s) as being trusted.
#[derive(Debug, Parser)]
pub struct TrustSubcommand {
    /// The tool(s) to mark as trusted.
    pub tools: Vec<ToolId>,
}

impl TrustSubcommand {
    pub async fn run(self, home: &Home) -> Result<()> {
        if self.tools.is_empty() {
            bail!("Please provide at least one tool to trust.");
        }

        let trust_storage = home.trust();
        let (new_tools, existing_tools) = self
            .tools
            .into_iter()
            .partition::<Vec<_>, _>(|tool| trust_storage.add_tool(tool.clone()));

        if !new_tools.is_empty() {
            println!("Trusted new tools:");
            for tool in new_tools {
                println!("  - {tool}");
            }
        }
        if !existing_tools.is_empty() {
            println!("Already trusted tools:");
            for tool in existing_tools {
                println!("  - {tool}");
            }
        }

        Ok(())
    }
}
