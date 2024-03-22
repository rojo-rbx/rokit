use anyhow::Result;
use clap::Parser;

use aftman::storage::Home;

/// Lists all existing tools managed by Aftman.
#[derive(Debug, Parser)]
pub struct ListSubcommand {}

impl ListSubcommand {
    pub async fn run(&self, _home: &Home) -> Result<()> {
        Ok(())
    }
}
