use aftman::storage::Home;
use anyhow::{Context, Result};
use clap::Parser;
use tokio::time::Instant;

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
pub struct Cli {
    #[clap(subcommand)]
    pub subcommand: Subcommand,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        // 1. Load aftman data structures
        let start_home = Instant::now();
        let home = Home::load_from_env().await.context(
            "Failed to load Aftman home!\
            \nYour installation or environment may be corrupted.",
        )?;
        tracing::trace!(
            elapsed = ?start_home.elapsed(),
            "Aftman loaded"
        );

        // 2. Run the subcommand and capture the result - note that we
        // do not (!!!) use the question mark operator here, because we
        // want to save our data below even if the subcommand fails.
        let start_command = Instant::now();
        let result = self.subcommand.run(&home).await;
        tracing::trace!(
            elapsed = ?start_command.elapsed(),
            success = result.is_ok(),
            "Aftman ran",
        );

        // 3. Save aftman data structures to disk
        let start_save = Instant::now();
        home.save().await.context(
            "Failed to save Aftman data!\
            \nChanges to trust, tools, and more may have been lost.",
        )?;
        tracing::trace!(
            elapsed = ?start_save.elapsed(),
            "Aftman saved"
        );

        // 4. Return the result of the subcommand
        result
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
    pub async fn run(self, home: &Home) -> Result<()> {
        match self {
            // Hidden subcommands
            Self::DebugSystemInfo(cmd) => cmd.run(home).await,
            Self::DebugTrustedTools(cmd) => cmd.run(home).await,
            // Public subcommands
            Self::List(cmd) => cmd.run(home).await,
            Self::Trust(cmd) => cmd.run(home).await,
            Self::Untrust(cmd) => cmd.run(home).await,
        }
    }
}
