use aftman::storage::Home;
use anyhow::{Context, Result};
use clap::Parser;

mod debug_system_info;
mod debug_trusted_tools;
mod list;
mod trust;
mod untrust;

use self::debug_system_info::DebugSystemInfoSubcommand;
use self::debug_trusted_tools::DebugTrustedToolsSubcommand;
use self::list::ListSubcommand;
use self::trust::TrustSubcommand;
use self::untrust::UntrustSubcommand;

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
    DebugSystemInfo(DebugSystemInfoSubcommand),
    #[clap(hide = true)]
    DebugTrustedTools(DebugTrustedToolsSubcommand),
    // Public subcommands
    List(ListSubcommand),
    Trust(TrustSubcommand),
    Untrust(UntrustSubcommand),
}

impl Subcommand {
    pub async fn run(self) -> Result<()> {
        let home = Home::load_from_env()
            .await
            .context("Failed to load Aftman home!")?;

        let result = match self {
            // Hidden subcommands
            Self::DebugSystemInfo(cmd) => cmd.run(&home).await,
            Self::DebugTrustedTools(cmd) => cmd.run(&home).await,
            // Public subcommands
            Self::List(cmd) => cmd.run(&home).await,
            Self::Trust(cmd) => cmd.run(&home).await,
            Self::Untrust(cmd) => cmd.run(&home).await,
        };

        home.save().await.context(
            "Failed to save Aftman data!\
            \nChanges to trust, tools, and more may have been lost.",
        )?;

        result
    }
}
