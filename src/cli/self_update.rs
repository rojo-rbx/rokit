use anyhow::{bail, Result};
use clap::Parser;

use rokit::storage::Home;

/// Updates Rokit to the latest version.
#[derive(Debug, Parser)]
pub struct SelfUpdateSubcommand {}

impl SelfUpdateSubcommand {
    pub async fn run(self, _home: &Home) -> Result<()> {
        bail!("Command is not yet implemented")
    }
}
