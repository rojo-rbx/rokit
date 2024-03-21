use anyhow::Result;
use clap::Parser;

use aftman::storage::Home;

/// Prints out information about currently trusted tools.
#[derive(Debug, Parser)]
pub struct GetTrustedToolsSubcommand {}

impl GetTrustedToolsSubcommand {
    pub async fn run(&self) -> Result<()> {
        let home = Home::from_env()?;
        let storage = home.trust_storage().await?;
        println!("Trusted tools:");
        for tool in storage.iter_tools() {
            println!("{tool}");
        }
        Ok(())
    }
}
