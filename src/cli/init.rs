use anyhow::{bail, Context, Result};
use clap::Parser;
use console::style;

use rokit::{manifests::RokitManifest, storage::Home, system::current_dir};

use crate::util::CliProgressTracker;

/// Initializes a new Rokit project in the current directory.
#[derive(Debug, Parser)]
pub struct InitSubcommand {
    /// Overwrite an existing Rokit project in the current directory.
    #[clap(long, hide = true)]
    pub force: bool,
}

impl InitSubcommand {
    pub async fn run(self, _: &Home) -> Result<()> {
        let cwd = current_dir().await;

        if RokitManifest::load(&cwd).await.is_ok() && !self.force {
            bail!(
                "A Rokit project already exists in this directory.\n\
                \nRun `{}` to add a new tool, or `{}` to update existing tools.\
                \nRun `{}` to see all available commands.",
                style("rokit add").bold().green(),
                style("rokit update").bold().green(),
                style("rokit --help").bold().green()
            )
        }

        let manifest = RokitManifest::load_or_create(&cwd)
            .await
            .context("Failed to create new Rokit manifest")?;

        // FUTURE: Maybe ask the user if they want to add some common tools here?
        // We could use `dialoguer` and its multi-select prompt for this - and we
        // already have a list of common tools supported in the `add` subcommand.

        // NOTE: We use a progress bar only to show the final message to the
        // user below, to maintain consistent formatting with other commands.
        let pt = CliProgressTracker::new_with_message("Initializing", 1);

        manifest
            .save(cwd)
            .await
            .context("Failed to save new Rokit manifest")?;

        pt.finish_with_message(format!(
            "Initialized new Rokit project successfully! {}\n\
            \nYou can now run `{}` to add new tools to your project.",
            pt.formatted_elapsed(),
            style("rokit add").bold().green()
        ));

        Ok(())
    }
}
