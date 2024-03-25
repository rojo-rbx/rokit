use anyhow::Result;
use clap::Parser;

use aftman::storage::Home;
use tracing::info;

/// Lists all existing tools managed by Aftman.
#[derive(Debug, Parser)]
pub struct ListSubcommand {}

impl ListSubcommand {
    pub async fn run(&self, home: &Home) -> Result<()> {
        let cache = home.tool_cache();
        let tools = cache
            .all_installed_ids()
            .into_iter()
            .map(|id| (id.clone(), cache.all_installed_versions_for_id(&id)))
            .collect::<Vec<_>>();

        if tools.is_empty() {
            info!("No tools installed.");
        } else {
            let mut lines = vec![String::from("Installed tools:\n")];

            for (id, mut versions) in tools {
                versions.reverse(); // List newest versions first

                let vers = versions
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                lines.push(id.to_string());
                lines.push(format!("  {vers}"));
            }

            info!("{}", lines.join("\n"));
        }

        Ok(())
    }
}
