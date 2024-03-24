use std::collections::BTreeSet;

use anyhow::{Context, Result};
use clap::Parser;

use aftman::{description::Description, manifests::AftmanManifest, storage::Home};
use futures::{stream::FuturesUnordered, TryStreamExt};
use tokio::time::Instant;

use crate::util::{discover_aftman_manifest_dirs, github_tool_source, prompt_for_install_trust};

/// Adds a new tool to Aftman and installs it.
#[derive(Debug, Parser)]
pub struct InstallSubcommand {
    /// Skip checking if tools have been trusted before.
    /// It is recommended to only use this on CI machines.
    #[clap(long)]
    no_trust_check: bool,
    /// Force install all tools, even if they are already installed.
    #[clap(long)]
    force: bool,
}

impl InstallSubcommand {
    pub async fn run(&self, home: &Home) -> Result<()> {
        let force = self.force;
        let start = Instant::now();
        let (manifest_paths, source) = tokio::try_join!(
            discover_aftman_manifest_dirs(home),
            github_tool_source(home)
        )?;

        // 1. Gather tool specifications from all known manifests

        let manifests = manifest_paths
            .iter()
            .map(AftmanManifest::load)
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            .await
            .context("Failed to load manifest")?;

        let tools = manifests
            .iter()
            .flat_map(|manifest| manifest.tool_specs())
            .collect::<Vec<_>>();

        // 2. Check for trust

        let tools = if self.no_trust_check {
            tools
        } else {
            let mut trusted_specs = Vec::new();
            let trust_cache = home.trust_cache();
            for (tool_alias, tool_spec) in tools {
                let tool_id = tool_spec.clone().into_id();
                if !trust_cache.is_trusted(&tool_id) {
                    if prompt_for_install_trust(&tool_id).await? {
                        trust_cache.add_tool(tool_id);
                        trusted_specs.push((tool_alias, tool_spec));
                    }
                } else {
                    trusted_specs.push((tool_alias, tool_spec));
                }
            }
            trusted_specs
        };

        // NOTE: Deduplicate tool aliases and specs since they may appear in several manifests
        let tool_aliases = tools
            .iter()
            .map(|(alias, _)| alias.clone())
            .collect::<BTreeSet<_>>();
        let tool_specs = tools
            .into_iter()
            .map(|(_, spec)| spec)
            .collect::<BTreeSet<_>>();

        // 3. Find artifacts, download and install them

        let description = Description::current();
        let install_cache = home.install_cache();
        let tool_storage = home.tool_storage();

        let artifacts = tool_specs
            .into_iter()
            .map(|tool_spec| async {
                if install_cache.is_installed(&tool_spec) && !force {
                    tracing::info!("Skipping already installed {tool_spec}");
                    // HACK: Force the async closure to take ownership
                    // of tool_spec by returning it from the closure
                    return anyhow::Ok(tool_spec);
                }

                tracing::info!("Downloading {tool_spec}");
                let release = source
                    .find_release(&tool_spec)
                    .await?
                    .with_context(|| format!("Failed to find release for {tool_spec}"))?;
                let artifact = source
                    .find_compatible_artifacts(&tool_spec, &release, &description)
                    .first()
                    .cloned()
                    .with_context(|| format!("No compatible artifact found for {tool_spec}"))?;
                let contents = source
                    .download_artifact_contents(&artifact)
                    .await
                    .with_context(|| format!("Failed to download contents for {tool_spec}"))?;

                tracing::info!("Installing {tool_spec}");
                let extracted = artifact
                    .extract_contents(contents)
                    .await
                    .with_context(|| format!("Failed to extract contents for {tool_spec}"))?;
                tool_storage
                    .replace_tool_contents(&tool_spec, extracted)
                    .await?;

                install_cache.add_spec(tool_spec.clone());
                Ok(tool_spec)
            })
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            .await?;

        // 4. Link all of the (possibly new) aliases, we do this even if the
        // tool is already installed in case the link(s) have been corrupted
        // and the user tries to re-install tools to fix it.

        tool_aliases
            .iter()
            .map(|alias| tool_storage.create_tool_link(alias))
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            .await?;

        tracing::info!(
            "Completed in {:.2?} ({} tools, {} links)",
            start.elapsed(),
            artifacts.len(),
            tool_aliases.len(),
        );

        Ok(())
    }
}
