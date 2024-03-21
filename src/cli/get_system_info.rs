use aftman::system::Description;
use anyhow::Result;
use clap::Parser;

/// Prints out information about the system detected by Aftman.
#[derive(Debug, Parser)]
pub struct GetSystemInfoSubcommand {}

impl GetSystemInfoSubcommand {
    pub async fn run(&self) -> Result<()> {
        let desc = Description::current();
        println!("Current system information:");
        println!("{desc:#?}");
        Ok(())
    }
}
