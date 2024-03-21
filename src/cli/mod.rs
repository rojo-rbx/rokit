use anyhow::Result;
use clap::Parser;

mod debug_system_info;
mod debug_trusted_tools;
mod list;

use self::debug_system_info::GetSystemInfoSubcommand;
use self::debug_trusted_tools::GetTrustedToolsSubcommand;
use self::list::ListSubcommand;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Args {
    #[clap(subcommand)]
    pub subcommand: Subcommand,
}

impl Args {
    pub async fn run(self) -> Result<()> {
        self.subcommand.run().await
    }
}

#[derive(Debug, Parser)]
pub enum Subcommand {
    // Hidden subcommands (for debugging)
    #[clap(hide = true)]
    DebugSystemInfo(GetSystemInfoSubcommand),
    #[clap(hide = true)]
    DebugTrustedTools(GetTrustedToolsSubcommand),
    // Public subcommands
    List(ListSubcommand),
}

impl Subcommand {
    pub async fn run(self) -> Result<()> {
        match self {
            // Hidden subcommands
            Self::DebugSystemInfo(cmd) => cmd.run().await,
            Self::DebugTrustedTools(cmd) => cmd.run().await,
            // Public subcommands
            Self::List(cmd) => cmd.run().await,
        }
    }
}
