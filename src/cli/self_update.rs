use anyhow::{bail, Context, Result};
use clap::Parser;
use console::style;
use semver::Version;

use rokit::{storage::Home, tool::ToolId};

use crate::util::{find_most_compatible_artifact, CliProgressTracker};

/// Updates Rokit to the latest version.
#[derive(Debug, Parser)]
pub struct SelfUpdateSubcommand {
    /// Update even if the latest version is already installed.
    #[clap(long, hide = true)]
    pub force: bool,
}

impl SelfUpdateSubcommand {
    pub async fn run(self, home: &Home) -> Result<()> {
        let repo = env!("CARGO_PKG_REPOSITORY")
            .trim_start_matches("https://github.com/")
            .trim_end_matches(".git");
        let Ok(tool_id) = repo.parse::<ToolId>() else {
            bail!(
                "Failed to parse manifest repository URL!\
                \nThis is a bug in Rokit, please report it at:
                \n{repo}"
            );
        };

        let pt = CliProgressTracker::new_with_message("Loading", 4);
        let source = home.artifact_source().await?;

        pt.task_completed();
        pt.update_message("Fetching");

        let artifacts = source.get_latest_release(&tool_id).await?;

        // Skip updating if we are already on the latest version
        let version_current = env!("CARGO_PKG_VERSION").parse::<Version>().unwrap();
        let version_latest = artifacts.first().unwrap().tool_spec.version().clone();
        if version_current >= version_latest && !self.force {
            let msg = format!(
                "Rokit is already up-to-date! {}\n\n\
                The latest version is {}.",
                pt.formatted_elapsed(),
                style(&version_latest).bold().magenta(),
            );
            pt.finish_with_message(msg);
            return Ok(());
        }

        // Download the most compatible artifact - this should always exist,
        // otherwise we wouldn't be able to run Rokit in the first place...?
        pt.task_completed();
        pt.update_message("Downloading");

        let artifact = find_most_compatible_artifact(&artifacts, &tool_id)
            .context("No compatible Rokit artifact was found (WAT???)")?;
        let artifact_contents = source
            .download_artifact_contents(&artifact)
            .await
            .context("Failed to download latest Rokit binary")?;

        // Extract the binary contents from the artifact
        pt.task_completed();
        pt.update_message("Extracting");
        let binary_contents = artifact
            .extract_contents(artifact_contents)
            .await
            .context("Failed to extract Rokit binary from archive")?;

        // Finally, we need to replace the current binary contents and all links to it.
        pt.task_completed();
        pt.update_message("Linking");

        let storage = home.tool_storage();
        storage.replace_rokit_contents(binary_contents).await;
        storage
            .recreate_all_links()
            .await
            .context("Failed to create new tool links")?;

        // Everything went well, yay!
        let msg = format!(
            "Rokit has been updated successfully! {}\n\
            \nYou are now running version {}, updated from {}.",
            pt.formatted_elapsed(),
            style(&version_latest).bold().magenta(),
            style(&version_current).bold().magenta(),
        );
        pt.finish_with_message(msg);

        Ok(())
    }
}
