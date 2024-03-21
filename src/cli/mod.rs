use anyhow::Result;
use clap::Parser;

mod get_system_info;
mod get_trusted_tools;
mod list;

use self::get_system_info::GetSystemInfoSubcommand;
use self::get_trusted_tools::GetTrustedToolsSubcommand;
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
    // Public subcommands
    List(ListSubcommand),
    // Hidden subcommands (for debugging)
    #[clap(hide = true)]
    GetSystemInfo(GetSystemInfoSubcommand),
    #[clap(hide = true)]
    GetTrustedTools(GetTrustedToolsSubcommand),
}

impl Subcommand {
    pub async fn run(self) -> Result<()> {
        match self {
            // Public subcommands
            Self::List(cmd) => cmd.run().await,
            // Hidden subcommands
            Self::GetSystemInfo(cmd) => cmd.run().await,
            Self::GetTrustedTools(cmd) => cmd.run().await,
        }
    }
}
