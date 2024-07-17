use anyhow::{Context, Result};
use clap::{ArgAction, CommandFactory, Parser};
use tokio::time::Instant;
use tracing::level_filters::LevelFilter;

use rokit::storage::Home;
use rokit::system::ProcessParent;

use crate::util::init_tracing;

mod add;
mod authenticate;
mod init;
mod install;
mod list;
mod self_install;
mod self_update;
mod system_info;
mod trust;
mod update;

use self::add::AddSubcommand;
use self::authenticate::AuthenticateSubcommand;
use self::init::InitSubcommand;
use self::install::InstallSubcommand;
use self::list::ListSubcommand;
use self::self_install::SelfInstallSubcommand;
use self::self_update::SelfUpdateSubcommand;
use self::system_info::SystemInfoSubcommand;
use self::trust::TrustSubcommand;
use self::update::UpdateSubcommand;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommand: Option<Subcommand>,
    #[clap(flatten)]
    pub options: GlobalOptions,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        // Enable the appropriate level of tracing / logging
        init_tracing(self.options.tracing_level_filter());

        // If we didn't get a subcommand, we should either print the help,
        // or automatically run self-install if launched from the explorer
        let (auto_self_install, command) = if let Some(subcommand) = self.subcommand {
            (false, subcommand)
        } else if ProcessParent::get()
            .await
            .is_some_and(ProcessParent::is_launcher)
        {
            let subcommand = Subcommand::SelfInstall(SelfInstallSubcommand {});
            (true, subcommand)
        } else {
            Cli::command().print_help()?;
            std::process::exit(0);
        };

        // Load Rokit data structures
        let start_home = Instant::now();
        let home = Home::load_from_env().await.context(
            "Failed to load Rokit home!\
            \nYour installation or environment may be corrupted.",
        )?;
        tracing::trace!(
            elapsed = ?start_home.elapsed(),
            "Rokit loaded"
        );

        // Run the subcommand and capture the result - note that we
        // do not (!!!) use the question mark operator here, because
        // we want to save our data below even if the subcommand fails.
        let start_command = Instant::now();
        let result = command.run(&home).await;
        tracing::trace!(
            elapsed = ?start_command.elapsed(),
            success = result.is_ok(),
            "Rokit ran",
        );

        // Save Rokit data structures to disk
        let start_save = Instant::now();
        home.save().await.context(
            "Failed to save Rokit data!\
            \nChanges to trust, tools, and more may have been lost.",
        )?;
        tracing::trace!(
            elapsed = ?start_save.elapsed(),
            "Rokit saved"
        );

        // Wait for user input if we automatically ran the
        // self-install from clicking Rokit in the explorer,
        // so that the window doesn't immediately close.
        if auto_self_install {
            dialoguer::Input::new()
                .with_prompt("Press Enter to continue")
                .show_default(false)
                .allow_empty(true)
                .report(false)
                .default(true)
                .interact()
                .ok();
        }

        // Return the result of the subcommand
        result
    }
}

#[derive(Debug, Parser)]
pub enum Subcommand {
    Add(AddSubcommand),
    Authenticate(AuthenticateSubcommand),
    Init(InitSubcommand),
    Install(InstallSubcommand),
    List(ListSubcommand),
    SelfInstall(SelfInstallSubcommand),
    SelfUpdate(SelfUpdateSubcommand),
    SystemInfo(SystemInfoSubcommand),
    Trust(TrustSubcommand),
    Update(UpdateSubcommand),
}

impl Subcommand {
    pub async fn run(self, home: &Home) -> Result<()> {
        match self {
            Self::Add(cmd) => cmd.run(home).await,
            Self::Authenticate(cmd) => cmd.run(home).await,
            Self::Init(cmd) => cmd.run(home).await,
            Self::Install(cmd) => cmd.run(home).await,
            Self::List(cmd) => cmd.run(home).await,
            Self::SelfInstall(cmd) => cmd.run(home).await,
            Self::SelfUpdate(cmd) => cmd.run(home).await,
            Self::SystemInfo(cmd) => cmd.run(home).await,
            Self::Trust(cmd) => cmd.run(home).await,
            Self::Update(cmd) => cmd.run(home).await,
        }
    }
}

#[derive(Debug, Parser)]
pub struct GlobalOptions {
    #[clap(short, long, action = ArgAction::Count)]
    pub verbose: u8,
}

impl GlobalOptions {
    pub fn tracing_level_filter(&self) -> LevelFilter {
        match self.verbose {
            0 => LevelFilter::INFO,
            1 => LevelFilter::DEBUG,
            _ => LevelFilter::TRACE,
        }
    }
}
