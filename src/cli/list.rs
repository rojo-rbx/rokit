use anyhow::Result;
use clap::Parser;

use console::style;
use rokit::{storage::Home, tool::ToolId};

/// Lists all existing tools managed by Rokit.
#[derive(Debug, Parser)]
pub struct ListSubcommand {
    /// A specific tool identifier to list versions for.
    id: Option<ToolId>,
}

impl ListSubcommand {
    pub async fn run(self, home: &Home) -> Result<()> {
        let cache = home.tool_cache();
        let tools = cache
            .all_installed_ids()
            .into_iter()
            .map(|id| (id.clone(), cache.all_installed_versions_for_id(&id)))
            .collect::<Vec<_>>();

        let header;
        let mut lines = vec![];

        let list_bullet = style("•").dim();
        if tools.is_empty() {
            header = String::from("🛠️  No tools are installed.");
        } else if let Some(id) = self.id {
            let mut versions = cache.all_installed_versions_for_id(&id);
            versions.reverse(); // List newest versions first
            if !versions.is_empty() {
                header = format!("🛠️  Installed versions of {id}:");
                for version in versions {
                    lines.push(format!("  {list_bullet} {version}"));
                }
            } else {
                header = format!("🛠️  No versions of {id} are installed.");
            }
        } else {
            header = String::from("🛠️  All installed tools:");
            for (id, mut versions) in tools {
                versions.reverse(); // List newest versions first
                lines.push(id.to_string());
                for version in versions {
                    lines.push(format!("  {list_bullet} {version}"));
                }
            }
        }

        let extra_newline = if !lines.is_empty() { "\n" } else { "" };
        let message = lines
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        println!("{header}\n{extra_newline}{message}");

        Ok(())
    }
}
