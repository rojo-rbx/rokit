use anyhow::{Context, Result};
use clap::Parser;
use tokio::time::Instant;

use rokit::storage::Home;

mod add;
mod install;
mod list;
mod self_install;
mod trust;

use self::add::AddSubcommand;
use self::install::InstallSubcommand;
use self::list::ListSubcommand;
use self::self_install::SelfInstallSubcommand;
use self::trust::TrustSubcommand;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommand: Subcommand,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        // 1. Load Rokit data structures
        let start_home = Instant::now();
        let home = Home::load_from_env().await.context(
            "Failed to load Rokit home!\
            \nYour installation or environment may be corrupted.",
        )?;
        tracing::trace!(
            elapsed = ?start_home.elapsed(),
            "Rokit loaded"
        );

        // 2. Run the subcommand and capture the result - note that we
        // do not (!!!) use the question mark operator here, because we
        // want to save our data below even if the subcommand fails.
        let start_command = Instant::now();
        let result = self.subcommand.run(&home).await;
        tracing::trace!(
            elapsed = ?start_command.elapsed(),
            success = result.is_ok(),
            "Rokit ran",
        );

        // 3. Save Rokit data structures to disk
        let start_save = Instant::now();
        home.save().await.context(
            "Failed to save Rokit data!\
            \nChanges to trust, tools, and more may have been lost.",
        )?;
        tracing::trace!(
            elapsed = ?start_save.elapsed(),
            "Rokit saved"
        );

        // 4. Return the result of the subcommand
        result
    }
}

#[derive(Debug, Parser)]
pub enum Subcommand {
    Add(AddSubcommand),
    Install(InstallSubcommand),
    List(ListSubcommand),
    SelfInstall(SelfInstallSubcommand),
    Trust(TrustSubcommand),
}

impl Subcommand {
    pub async fn run(self, home: &Home) -> Result<()> {
        match self {
            Self::Add(cmd) => cmd.run(home).await,
            Self::Install(cmd) => cmd.run(home).await,
            Self::List(cmd) => cmd.run(home).await,
            Self::SelfInstall(cmd) => cmd.run(home).await,
            Self::Trust(cmd) => cmd.run(home).await,
        }
    }
}
