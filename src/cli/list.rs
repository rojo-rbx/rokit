use anyhow::Result;
use clap::Parser;

use aftman::storage::Home;

/// Lists all existing tools managed by Aftman.
#[derive(Debug, Parser)]
pub struct ListSubcommand {}

impl ListSubcommand {
    pub async fn run(&self, home: &Home) -> Result<()> {
        let cache = home.install_cache();
        let tools = cache
            .all_ids()
            .into_iter()
            .map(|id| (id.clone(), cache.all_versions_for_id(&id)))
            .collect::<Vec<_>>();

        if tools.is_empty() {
            println!("No tools installed.");
        } else {
            println!("Installed tools:\n");
            for (id, mut versions) in tools {
                versions.reverse(); // List newest versions first

                let vers = versions
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                println!("{id}\n  {vers}");
            }
        }

        Ok(())
    }
}
