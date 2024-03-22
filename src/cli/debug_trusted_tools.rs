use anyhow::Result;
use clap::Parser;

use aftman::storage::Home;

/// Prints out information about currently trusted tools.
#[derive(Debug, Parser)]
pub struct DebugTrustedToolsSubcommand {}

impl DebugTrustedToolsSubcommand {
    pub async fn run(&self, home: &Home) -> Result<()> {
        println!("Trusted tools:");
        for tool in home.trust_cache().all_tools() {
            println!("{tool}");
        }
        Ok(())
    }
}
