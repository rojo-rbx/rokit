use anyhow::{Context, Result};
use clap::Parser;
use console::style;

use rokit::{storage::Home, system::add_to_path};
use tracing::warn;

use crate::util::{finish_progress_bar, new_progress_bar};

/// Installs / re-installs Rokit, and updates all tool links.
#[derive(Debug, Parser)]
pub struct SelfInstallSubcommand {}

impl SelfInstallSubcommand {
    pub async fn run(&self, home: &Home) -> Result<()> {
        let storage = home.tool_storage();

        let pb = new_progress_bar("Linking", 2, 1);
        let (had_rokit_installed, was_rokit_updated) = storage.recreate_all_links().await.context(
            "Failed to recreate tool links!\
            \nYour installation may be corrupted.",
        )?;

        pb.inc(1);
        pb.set_message("Pathifying");

        let path_was_changed = add_to_path(home)
            .await
            .inspect_err(|e| {
                warn!(
                    "Failed to automatically add Rokit to your PATH!\
                    \nPlease add `~/.rokit/bin` to be able to run tools.
                    \nError: {e:?}",
                )
            })
            .unwrap_or(false);
        let path_message_lines = if path_was_changed {
            format!(
                "\n\nExecutables for Rokit and tools have been added to {}.\
                \nPlease restart your terminal for the changes to take effect.",
                style("$PATH").bold()
            )
        } else {
            String::new()
        };

        let main_message = if !had_rokit_installed {
            "Rokit has been installed successfully!"
        } else if was_rokit_updated {
            "Rokit was re-linked successfully!"
        } else {
            "Rokit links are already up-to-date."
        };

        let help_message = style("rokit --help").bold().green();
        let post_message = if path_was_changed {
            format!("\n\nThen, run `{help_message}` to get started using Rokit.")
        } else {
            format!("\n\nRun `{help_message}` to get started using Rokit.")
        };

        let msg = format!(
            "{main_message} {}{path_message_lines}{post_message}",
            style(format!("(took {:.2?})", pb.elapsed())).dim(),
        );
        finish_progress_bar(pb, msg);

        Ok(())
    }
}
