use anyhow::{bail, Context, Result};
use clap::Parser;
use console::style;
use futures::{stream::FuturesUnordered, TryStreamExt};

use rokit::{discovery::discover_all_manifests, manifests::RokitManifest, storage::Home};

use crate::util::{
    find_most_compatible_artifact, CliProgressTracker, ToolAliasOrIdOrSpec, ToolIdOrSpec,
};

/// Updates all tools, or specific tools, to the latest version.
#[derive(Debug, Parser)]
pub struct UpdateSubcommand {
    /// The tools to update - can be aliases, ids, or specifications.
    /// Omit to update all tools.
    pub tools: Vec<ToolAliasOrIdOrSpec>,
    /// Update tools globally instead of using the nearest manifest file.
    #[clap(long)]
    pub global: bool,
    /// Check for updates without actually updating the tools.
    #[clap(long)]
    pub check: bool,
}

impl UpdateSubcommand {
    pub async fn run(self, home: &Home) -> Result<()> {
        // 1. Load tool source and the desired manifest
        let source = home.artifact_source().await?;
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
                .map(|(alias, spec)| (alias, ToolIdOrSpec::Id(spec.id().clone())))
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
                            ToolAliasOrIdOrSpec::Spec(spec) => spec.id().clone(),
                            ToolAliasOrIdOrSpec::Alias(_) => unreachable!(),
                        };
                        let found = manifest
                            .tool_specs()
                            .iter()
                            .filter_map(|(a, s)| {
                                if s.id() == &search_id {
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
                        }
                        found.first().unwrap().clone()
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
                            let id = ToolIdOrSpec::Id(spec.id().clone());
                            Ok::<_, anyhow::Error>((alias, id))
                        }
                    }
                })
                .collect::<Result<Vec<_>>>()?
        };
        let pt = CliProgressTracker::new_with_message_and_subtasks("Fetching", tools.len(), 3);

        // 3. Fetch the latest or desired versions of the tools
        let tool_releases = tools
            .into_iter()
            .map(|(alias, tool)| async {
                let (alias, id, artifacts) = match tool {
                    ToolIdOrSpec::Spec(spec) => {
                        let artifacts =
                            source.get_specific_release(&spec).await.with_context(|| {
                                format!(
                                    "Failed to fetch release for '{spec}'!\
                                    \nMake sure the given tool version exists."
                                )
                            })?;
                        (alias, spec.id().clone(), artifacts)
                    }
                    ToolIdOrSpec::Id(id) => {
                        let artifacts =
                            source.get_latest_release(&id).await.with_context(|| {
                                format!(
                                    "Failed to fetch latest release for '{id}'!\
                                    \nMake sure the given tool identifier exists."
                                )
                            })?;
                        (alias, id, artifacts)
                    }
                };

                let artifact = find_most_compatible_artifact(&artifacts.artifacts, &id)?;
                pt.subtask_completed();

                Ok::<_, anyhow::Error>((alias, id, artifact))
            })
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            .await?;

        // 4. Check if the --check flag was used, and if so, check for updates
        let tools_changed = tool_releases
            .iter()
            .filter_map(|(alias, _, artifact)| {
                let spec_old = manifest.get_tool(alias).unwrap();
                let spec_new = artifact.tool_spec.clone();
                if spec_old == spec_new {
                    None
                } else {
                    Some((alias.clone(), spec_old, spec_new))
                }
            })
            .collect::<Vec<_>>();
        if self.check {
            let bullet = style("•").dim();
            let arrow = style("→").dim();

            let updated_tool_lines = tools_changed
                .iter()
                .map(|(alias, spec_old, spec_new)| {
                    format!(
                        "{bullet} {} {} {arrow} {}",
                        style(alias.to_string()).bold().cyan(),
                        style(spec_old.version()).yellow(),
                        style(spec_new.version()).bold().yellow()
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            pt.update_message("Checking for updates");

            if tools_changed.is_empty() {
                pt.finish_with_message(format!(
                    "All tools are already up-to-date! {}",
                    pt.formatted_elapsed(),
                ));
            } else {
                pt.finish_with_message(format!(
                    "New versions are available for {} tool{} {}\
                     \n\n{updated_tool_lines}\n\n\
                    Run `{}` to update the tools.",
                    style(tools_changed.len()).bold().magenta(),
                    if tools_changed.len() == 1 { "" } else { "s" },
                    pt.formatted_elapsed(),
                    style("rokit update").bold().green(),
                ));
            }
            pt.subtask_completed();
            return Ok(());
        }

        // 5. Modify the manifest with the desired new tools, save
        pt.update_message("Modifying");

        for (alias, _, spec_new) in &tools_changed {
            manifest.update_tool(alias, spec_new);
            pt.subtask_completed();
        }
        manifest.save(&manifest_path).await?;

        // 6. Finally, display a nice message to the user
        let tools_changed = tool_releases
            .iter()
            .filter_map(|(alias, _, artifact)| {
                let spec_old = manifest.get_tool(alias).unwrap();
                let spec_new = artifact.tool_spec.clone();
                if spec_old == spec_new {
                    None
                } else {
                    Some((alias.clone(), spec_old, spec_new))
                }
            })
            .collect::<Vec<_>>();
        let bullet = style("•").dim();
        let arrow = style("→").dim();

        let updated_tool_lines = tools_changed
            .iter()
            .map(|(alias, spec_old, spec_new)| {
                format!(
                    "{bullet} {} {} {arrow} {}",
                    style(alias.to_string()).bold().cyan(),
                    style(spec_old.version()).yellow(),
                    style(spec_new.version()).bold().yellow()
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        if tools_changed.is_empty() {
            pt.finish_with_message(format!(
                "All tools are already up-to-date! {}",
                pt.formatted_elapsed(),
            ));
        } else {
            pt.finish_with_message(format!(
                "Updated versions for {} tool{} {}\
                \n\n{updated_tool_lines}\n\n\
                Run `{}` to install the updated tools.",
                style(tools_changed.len()).bold().magenta(),
                if tools_changed.len() == 1 { "" } else { "s" },
                pt.formatted_elapsed(),
                style("rokit install").bold().green(),
            ));
        }

        // FUTURE: Install the newly updated tools automatically

        Ok(())
    }
}
