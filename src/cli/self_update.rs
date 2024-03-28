use anyhow::{bail, Context, Result};
use clap::Parser;

use console::style;
use rokit::{
    manifests::AuthManifest,
    sources::{Artifact, ArtifactProvider, ArtifactSource},
    storage::Home,
    tool::ToolId,
};
use semver::Version;

use crate::util::{finish_progress_bar, new_progress_bar};

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

        let pb = new_progress_bar("Loading", 4, 1);

        // NOTE: Auth is not really necessary here since we know Rokit is not
        // a private repository, but it may still help against rate limiting.
        let auth = AuthManifest::load(home.path()).await?;
        let source = ArtifactSource::new_authenticated(&auth.get_all_tokens())?;

        pb.inc(1);
        pb.set_message("Fetching");

        let artifacts = source
            .get_latest_release(ArtifactProvider::GitHub, &tool_id)
            .await?;

        // Skip updating if we are already on the latest version
        let version_current = env!("CARGO_PKG_VERSION").parse::<Version>().unwrap();
        let version_latest = artifacts.first().unwrap().tool_spec.version().clone();
        if version_current >= version_latest && !self.force {
            let msg = format!(
                "Rokit is already up-to-date! {}\n\n\
                The latest version is {}.",
                style(format!("(took {:.2?})", pb.elapsed())).dim(),
                style(&version_latest).bold().magenta(),
            );
            finish_progress_bar(&pb, msg);
            return Ok(());
        }

        // Download the most compatible artifact - this should always exist,
        // otherwise we wouldn't be able to run Rokit in the first place...?
        pb.inc(1);
        pb.set_message("Downloading");

        let artifact = Artifact::sort_by_system_compatibility(&artifacts)
            .first()
            .cloned()
            .context("No compatible Rokit artifact was found (WAT???)")?;
        let artifact_contents = source
            .download_artifact_contents(&artifact)
            .await
            .context("Failed to download latest Rokit binary")?;

        // Extract the binary contents from the artifact
        pb.inc(1);
        pb.set_message("Extracting");
        let binary_contents = artifact
            .extract_contents(artifact_contents)
            .await
            .context("Failed to extract Rokit binary from archive")?;

        // Finally, we need to replace the current binary contents and all links to it.
        pb.inc(1);
        pb.set_message("Linking");

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
            style(format!("(took {:.2?})", pb.elapsed())).dim(),
            style(&version_latest).bold().magenta(),
            style(&version_current).bold().magenta(),
        );
        finish_progress_bar(&pb, msg);

        Ok(())
    }
}
