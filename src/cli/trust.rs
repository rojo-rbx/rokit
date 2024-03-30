use anyhow::{bail, Result};
use clap::Parser;
use console::style;

use rokit::{storage::Home, tool::ToolId};

use crate::util::CliProgressTracker;

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

        // NOTE: We use a progress bar only to show the final message to the
        // user below, to maintain consistent formatting with other commands.
        let pt = CliProgressTracker::new_with_message("Trusting", 1);

        let cache = home.tool_cache();
        let (added_tools, existing_tools) = self
            .tools
            .into_iter()
            .partition::<Vec<_>, _>(|tool| cache.add_trust(tool.clone()));

        if added_tools.len() == 1 && existing_tools.is_empty() {
            // Special case 1 with shorter output - a singular tool was added
            pt.finish_with_message(format!(
                "Tool {} is now trusted {}",
                added_tools[0],
                pt.formatted_elapsed(),
            ));
        } else if existing_tools.len() == 1 && added_tools.is_empty() {
            // Special case 2 with shorter output - a singular tool was already trusted
            pt.finish_with_message(format!(
                "Tool {} was already trusted {}",
                existing_tools[0],
                pt.formatted_elapsed(),
            ));
        } else {
            // General case with multiple tools added and/or already trusted
            let mut lines = Vec::new();
            let list_bullet = style("•").dim();

            if !added_tools.is_empty() {
                lines.push(String::from("These tools are now trusted:"));
                for tool in &added_tools {
                    lines.push(format!("  {list_bullet} {tool}"));
                }
            }

            if !existing_tools.is_empty() {
                lines.push(String::from("These tools were already trusted:"));
                for tool in &existing_tools {
                    lines.push(format!("  {list_bullet} {tool}"));
                }
            }

            pt.finish_with_message(format!(
                "Changed trust for {} tool{} {}\n\n{}",
                added_tools.len(),
                if added_tools.len() == 1 { "" } else { "s" },
                pt.formatted_elapsed(),
                lines.join("\n")
            ));
        }

        Ok(())
    }
}
