use anyhow::Result;
use clap::Parser;

use aftman::{description::Description, storage::Home};

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
