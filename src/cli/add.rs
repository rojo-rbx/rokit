use anyhow::{bail, Context, Result};
use clap::Parser;
use console::style;

use rokit::{
    description::Description,
    manifests::RokitManifest,
    storage::Home,
    tool::{ToolAlias, ToolId},
};

use crate::util::{
    discover_rokit_manifest_dir, finish_progress_bar, github_tool_source, new_progress_bar,
    prompt_for_trust, ToolIdOrSpec,
};

/// Adds a new tool to Rokit and installs it.
#[derive(Debug, Parser)]
pub struct AddSubcommand {
    /// A tool identifier or specification describing where
    /// to get the tool, and optionally what version to install.
    pub tool: ToolIdOrSpec,
    /// The name that will be used to run the tool.
    pub alias: Option<ToolAlias>,
    /// Add this tool globally instead of adding
    /// it to the nearest manifest file.
    #[clap(long)]
    pub global: bool,
    /// Force add and install the tool, even
    /// if it is already added or installed.
    #[clap(long)]
    force: bool,
}

impl AddSubcommand {
    pub async fn run(&self, home: &Home) -> Result<()> {
        let id: ToolId = self.tool.clone().into();
        let alias: ToolAlias = match self.alias.as_ref() {
            Some(alias) => alias.clone(),
            None => self.tool.clone().into(),
        };

        let tool_cache = home.tool_cache();
        let tool_storage = home.tool_storage();

        // 1. Check for trust, or prompt the user to trust the tool
        if !tool_cache.is_trusted(&id) {
            if !self.force && !prompt_for_trust(id.clone()).await? {
                bail!("Tool is not trusted - operation was aborted");
            }
            tool_cache.add_trust(id.clone());
        }

        // 2. Load tool source, manifest, and do a preflight check
        // to make sure we don't overwrite any existing tool(s)
        let source = github_tool_source(home).await?;
        let manifest_path = if self.global {
            home.path().to_path_buf()
        } else {
            discover_rokit_manifest_dir().await?
        };

        let mut manifest = if self.global {
            RokitManifest::load_or_create(&manifest_path).await?
        } else {
            RokitManifest::load(&manifest_path).await?
        };
        if manifest.has_tool(&alias) && !self.force {
            let global_flag = if self.global { "--global " } else { "" };
            bail!(
                "Tool already exists and can't be added: {id}\n\
                \n  - To update the tool, run `rokit update {global_flag}{id}`\
                \n  - To remove the tool, run `rokit remove {global_flag}{id}`"
            );
        }

        // 3. If we only got an id without a specified version, we
        // will fetch the latest non-prerelease release and use that
        let pb = new_progress_bar("Fetching", 5, 1);
        let spec = match self.tool.clone() {
            ToolIdOrSpec::Spec(spec) => {
                pb.inc(1);
                spec
            }
            ToolIdOrSpec::Id(id) => {
                let version = source
                    .find_latest_version(&id, false)
                    .await?
                    .with_context(|| format!("Failed to find latest release for {id}"))?;
                pb.inc(1);
                id.into_spec(version)
            }
        };

        // 4. Add the tool spec to the desired manifest file and save it
        manifest.add_tool(&alias, &spec);
        manifest.save(manifest_path).await?;

        // 5. Download and install the tool
        let description = Description::current();
        if !tool_cache.is_installed(&spec) || self.force {
            pb.set_message("Downloading");
            let release = source
                .find_release(&spec)
                .await?
                .with_context(|| format!("Failed to find release for {spec}"))?;
            pb.inc(1);
            let artifact = source
                .find_compatible_artifacts(&spec, &release, &description)
                .first()
                .cloned()
                .with_context(|| format!("No compatible artifact found for {spec}"))?;
            pb.inc(1);
            let contents = source
                .download_artifact_contents(&artifact)
                .await
                .with_context(|| format!("Failed to download contents for {spec}"))?;
            pb.inc(1);

            pb.set_message("Installing");
            let extracted = artifact
                .extract_contents(contents)
                .await
                .with_context(|| format!("Failed to extract contents for {spec}"))?;
            tool_storage.replace_tool_contents(&spec, extracted).await?;
            pb.inc(1);

            tool_cache.add_installed(spec.clone());
        } else {
            pb.inc(4);
        }

        // 6. Create the tool alias link
        pb.set_message("Linking");
        tool_storage.create_tool_link(&alias).await?;

        // 7. Finally, display a nice message to the user
        let msg = format!(
            "Added version {} of tool {}{} {}",
            style(spec.version()).bold().yellow(),
            style(spec.name()).bold().magenta(),
            if alias.name() != id.name() {
                format!(" with alias {}", style(alias.to_string()).bold().cyan())
            } else {
                "".into()
            },
            style(format!("(took {:.2?})", pb.elapsed())).dim(),
        );
        finish_progress_bar(pb, msg);

        Ok(())
    }
}
