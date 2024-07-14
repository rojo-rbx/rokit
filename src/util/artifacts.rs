use anyhow::{Context, Result};

use rokit::{
    descriptor::Descriptor,
    sources::{Artifact, ArtifactProvider},
    tool::ToolId,
};

pub fn find_most_compatible_artifact(artifacts: &[Artifact], tool_id: &ToolId) -> Result<Artifact> {
    let mut artifact_opt = Artifact::sort_by_system_compatibility(artifacts)
        .first()
        .cloned();

    if artifact_opt.is_none() {
        let current_desc = Descriptor::current_system();

        // If we failed to find an artifact compatible with the current system,
        // we may be able to give additional information to Rokit's users, or tool
        // maintainers who want to be Rokit-compatible, by examining the artifacts
        let no_artifacts_with_arch = artifacts.iter().all(|artifact| {
            artifact
                .name
                .as_deref()
                .and_then(Descriptor::detect)
                .map_or(false, |desc| desc.arch().is_none())
        });
        let additional_information = if no_artifacts_with_arch {
            let source_is_github = artifacts
                .iter()
                .all(|artifact| matches!(artifact.provider, ArtifactProvider::GitHub));
            let source_name = if source_is_github {
                "GitHub release files"
            } else {
                "tool release files"
            };
            Some(format!(
                "This seems to have been caused by {0} not \
                specifying an architecture in any of its artifacts.\
                \nIf you are the maintainer of this tool, you can resolve \
                this issue by specifying an architecture in {source_name}:\
                \n    {0}-{1}-{2}.zip",
                tool_id.name(),
                current_desc.os().as_str(),
                current_desc.arch().expect("no current arch (??)").as_str(),
            ))
        } else {
            None
        };

        // Let the user know about failing to find an artifact,
        // potentially with additional information generated above
        tracing::warn!(
            "Failed to find a fully compatible artifact for {tool_id}!{}\
            \nSearching for a fallback...",
            match additional_information {
                Some(info) => format!("\n{info}"),
                None => String::new(),
            }
        );

        if let Some(artifact) = Artifact::find_partially_compatible_fallback(artifacts) {
            tracing::info!(
                "Found fallback artifact '{}' for tool {tool_id}",
                artifact.name.as_deref().unwrap_or("N/A")
            );
            artifact_opt.replace(artifact);
        }
    }

    // If we did not find a compatible artifact, either directly
    // or through a fallback mechanism, this should be a hard error
    artifact_opt.with_context(|| format!("No compatible artifact found for {tool_id}"))
}
