use anyhow::Result;
use clap::Parser;

use aftman::{storage::Home, system::Description};

/// Prints out information about the system detected by Aftman.
#[derive(Debug, Parser)]
pub struct DebugSystemInfoSubcommand {}

impl DebugSystemInfoSubcommand {
    pub async fn run(&self, _home: &Home) -> Result<()> {
        let desc = Description::current();
        println!("Current system information:");
        println!("{desc:#?}");
        Ok(())
    }
}
