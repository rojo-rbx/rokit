use anyhow::{bail, Context, Result};
use clap::Parser;

use console::style;
use futures::{stream::FuturesUnordered, TryStreamExt};
use rokit::{
    discovery::discover_all_manifests,
    manifests::{AuthManifest, RokitManifest},
    sources::{Artifact, ArtifactProvider, ArtifactSource},
    storage::Home,
};

use crate::util::{finish_progress_bar, new_progress_bar, ToolAliasOrIdOrSpec, ToolIdOrSpec};

/// Updates all tools, or specific tools, to the latest version.
#[derive(Debug, Parser)]
pub struct UpdateSubcommand {
    /// The tools to update - can be aliases, ids, or specifications.
    /// Omit to update all tools.
    pub tools: Vec<ToolAliasOrIdOrSpec>,
    /// Update tools globally instead of using the nearest manifest file.
    #[clap(long)]
    pub global: bool,
}

impl UpdateSubcommand {
    pub async fn run(self, home: &Home) -> Result<()> {
        // 1. Load tool source and the desired manifest
        let auth = AuthManifest::load(home.path()).await?;
        let source = ArtifactSource::new_authenticated(&auth.get_all_tokens())?;
        let manifest_path = if self.global {
            home.path().to_path_buf()
        } else {
            let non_global_manifests = discover_all_manifests(true, true).await;
            non_global_manifests
                .first()
                .map(|m| m.path.parent().unwrap().to_path_buf())
                .context(
                    "No manifest was found for the current directory.\
                    \nRun `rokit init` in your project root to create one.",
                )?
        };

        let mut manifest = if self.global {
            RokitManifest::load_or_create(&manifest_path).await?
        } else {
            RokitManifest::load(&manifest_path).await?
        };

        // 2. Try to convert aliases into ids using existing tools,
        // or fill with existing tools if no tools were provided
        let tools = if self.tools.is_empty() {
            manifest
                .tool_specs()
                .iter()
                .cloned()
                .map(|(alias, spec)| (alias, ToolIdOrSpec::Id(spec.into_id())))
                .collect::<Vec<_>>()
        } else {
            // FUTURE: Refactor this logic here below, it's quite difficult to read
            self.tools
                .into_iter()
                .map(|tool| {
                    // NOTE: If we were given a tool id or spec, we need
                    // to find the proper alias for it in the manifest,
                    // which may or may not be correlated to the id or spec!
                    let alias = if let ToolAliasOrIdOrSpec::Alias(alias) = &tool {
                        alias.clone()
                    } else {
                        let search_id = match &tool {
                            ToolAliasOrIdOrSpec::Id(id) => id.clone(),
                            ToolAliasOrIdOrSpec::Spec(spec) => spec.clone().into_id(),
                            ToolAliasOrIdOrSpec::Alias(_) => unreachable!(),
                        };
                        let found = manifest
                            .tool_specs()
                            .iter()
                            .flat_map(|(a, s)| {
                                if s.clone().into_id() == search_id {
                                    Some(a.clone())
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();
                        if found.is_empty() {
                            bail!(
                                "No tool with the id '{search_id}' has been added to this project.\
                                \nYou can add the tool to the project using `{}`.",
                                style("rokit add").bold().green(),
                            )
                        } else if found.len() > 1 {
                            bail!(
                                "Multiple tools with the id '{search_id}' have been added to this project.\
                                \nPlease specify the tool by its alias, or update the manifest manually."
                            )
                        } else {
                            found.first().unwrap().clone()
                        }
                    };
                    // Transform tool alias, id, or spec -> (alias, id or spec)
                    match tool {
                        ToolAliasOrIdOrSpec::Id(id) => Ok((alias, id.into())),
                        ToolAliasOrIdOrSpec::Spec(spec) => Ok((alias, spec.into())),
                        ToolAliasOrIdOrSpec::Alias(alias) => {
                            let spec = manifest.get_tool(&alias).with_context(|| {
                                format!(
                                "No tool with the alias '{alias}' has been added to this project.\
                                \nYou can add the tool to the project using `{}`.",
                                style("rokit add").bold().green(),
                            )
                            })?;
                            let id = ToolIdOrSpec::Id(spec.into_id());
                            Ok::<_, anyhow::Error>((alias, id))
                        }
                    }
                })
                .collect::<Result<Vec<_>>>()?
        };
        let pb = new_progress_bar("Fetching", tools.len(), 3);

        // 3. Fetch the latest or desired versions of the tools
        let tool_releases = tools
            .into_iter()
            .map(|(alias, tool)| async {
                let (alias, id, artifacts) = match tool {
                    ToolIdOrSpec::Spec(spec) => {
                        let artifacts = source
                            .get_specific_release(ArtifactProvider::GitHub, &spec)
                            .await
                            .with_context(|| {
                                format!(
                                    "Failed to fetch release for '{spec}'!\
                                    \nMake sure the given tool version exists."
                                )
                            })?;
                        (alias, spec.into_id(), artifacts)
                    }
                    ToolIdOrSpec::Id(id) => {
                        let artifacts = source
                            .get_latest_release(ArtifactProvider::GitHub, &id)
                            .await
                            .with_context(|| {
                                format!(
                                    "Failed to fetch latest release for '{id}'!\
                                    \nMake sure the given tool identifier exists."
                                )
                            })?;
                        (alias, id, artifacts)
                    }
                };

                let artifact = Artifact::sort_by_system_compatibility(&artifacts)
                    .first()
                    .cloned()
                    .with_context(|| format!("No compatible artifact found for {id}"))?;

                pb.inc(1);
                Ok::<_, anyhow::Error>((alias, id, artifact))
            })
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            .await?;

        // 4. Modify the manifest with the desired new tools, save
        pb.set_message("Modifying");
        let tools_changed = tool_releases
            .iter()
            .flat_map(|(alias, _, artifact)| {
                let spec_old = manifest.get_tool(alias).unwrap();
                let spec_new = artifact.tool_spec.clone();
                if spec_old == spec_new {
                    None
                } else {
                    Some((alias.clone(), spec_old, spec_new))
                }
            })
            .collect::<Vec<_>>();
        for (alias, _, spec_new) in &tools_changed {
            manifest.update_tool(alias, spec_new);
            pb.inc(1);
        }
        manifest.save(&manifest_path).await?;

        // 5. Finally, display a nice message to the user
        if !tools_changed.is_empty() {
            let bullet = style("•").dim();
            let msg = format!(
                "Updated versions for {} tool{} {}\n\n{}\n\n\
                Run `{}` to install the updated tools.",
                style(tools_changed.len()).bold().magenta(),
                if tools_changed.len() == 1 { "" } else { "s" },
                style(format!("(took {:.2?})", pb.elapsed())).dim(),
                tools_changed
                    .into_iter()
                    .map(|(alias, spec_old, spec_new)| {
                        format!(
                            "{bullet} {} {} → {}",
                            style(alias.to_string()).bold().cyan(),
                            style(spec_old.version()).yellow(),
                            style(spec_new.version()).bold().yellow()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
                style("rokit install").bold().green(),
            );
            finish_progress_bar(pb, msg);
        } else {
            let msg = format!(
                "All tools are already up-to-date! {}",
                style(format!("(took {:.2?})", pb.elapsed())).dim()
            );
            finish_progress_bar(pb, msg);
        }

        // FUTURE: Install the newly updated tools automatically

        Ok(())
    }
}
