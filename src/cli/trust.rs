use anyhow::{bail, Result};
use clap::Parser;
use tracing::info;

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

        let cache = home.tool_cache();
        let (added_tools, existing_tools) = self
            .tools
            .into_iter()
            .partition::<Vec<_>, _>(|tool| cache.add_trust(tool.clone()));

        if added_tools.len() == 1 && existing_tools.is_empty() {
            info!("Tool has been marked as trusted: {}", added_tools[0]);
        } else if existing_tools.len() == 1 && added_tools.is_empty() {
            info!("Tool was already trusted: {}", existing_tools[0]);
        } else {
            let mut lines = Vec::new();

            if !added_tools.is_empty() {
                lines.push(String::from(
                    "The following tools have been marked as trusted:",
                ));
                for tool in added_tools {
                    lines.push(format!("  - {tool}"));
                }
            }

            if !existing_tools.is_empty() {
                lines.push(String::from("The following tools were already trusted:"));
                for tool in existing_tools {
                    lines.push(format!("  - {tool}"));
                }
            }

            info!("{}", lines.join("\n"));
        }

        Ok(())
    }
}
