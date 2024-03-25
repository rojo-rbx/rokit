use anyhow::{Context, Result};
use clap::Parser;
use console::style;

use rokit::storage::Home;

use crate::util::{finish_progress_bar, new_progress_bar};

/// Installs / re-installs Rokit, and updates all tool links.
#[derive(Debug, Parser)]
pub struct SelfInstallSubcommand {}

impl SelfInstallSubcommand {
    pub async fn run(&self, home: &Home) -> Result<()> {
        let storage = home.tool_storage();

        let pb = new_progress_bar("Linking", 1, 1);

        let (had_rokit_installed, was_rokit_updated) = storage.recreate_all_links().await.context(
            "Failed to recreate tool links!\
                \nYour installation may be corrupted.",
        )?;

        // TODO: Automatically populate the PATH variable
        let path_was_populated = false;
        let path_message_lines = if !path_was_populated {
            "\n\nBinaries for Rokit and tools have been added to your PATH.\
            \nPlease restart your terminal for the changes to take effect."
        } else {
            ""
        };

        let main_message = if !had_rokit_installed {
            "Rokit has been installed successfully!"
        } else if was_rokit_updated {
            "Rokit was re-linked successfully!"
        } else {
            "Rokit is already up-to-date."
        };

        let msg = format!(
            "{main_message} {}{path_message_lines}",
            style(format!("(took {:.2?})", pb.elapsed())).dim(),
        );
        finish_progress_bar(pb, msg);

        Ok(())
    }
}
