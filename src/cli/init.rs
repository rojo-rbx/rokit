use anyhow::{bail, Result};
use clap::Parser;

use rokit::storage::Home;

/// Initializes a new Rokit project in the current directory.
#[derive(Debug, Parser)]
pub struct InitSubcommand {}

impl InitSubcommand {
    pub async fn run(self, _home: &Home) -> Result<()> {
        bail!("Command is not yet implemented")
    }
}
