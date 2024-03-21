use anyhow::Result;
use clap::Parser;

/// Lists all existing tools managed by Aftman.
#[derive(Debug, Parser)]
pub struct ListSubcommand {}

impl ListSubcommand {
    pub async fn run(&self) -> Result<()> {
        Ok(())
    }
}
