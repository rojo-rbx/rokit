use anyhow::{Context, Result};

use rokit::{
    descriptor::{Arch, OS},
    sources::Artifact,
    tool::ToolId,
};

pub fn find_most_compatible_artifact(artifacts: &[Artifact], tool_id: &ToolId) -> Result<Artifact> {
    let mut artifact_opt = Artifact::sort_by_system_compatibility(artifacts)
        .first()
        .cloned();

    if artifact_opt.is_none() {
        if let Some(artifact) = Artifact::find_partially_compatible_fallback(artifacts) {
            tracing::debug!(
                %tool_id,
                name = %artifact.name.as_deref().unwrap_or("N/A"),
                "found fallback artifact for tool",
            );
            artifact_opt.replace(artifact);
        } else {
            // If we failed to find an artifact compatible with the current system,
            // we may be able to give additional information to Rokit's users, or tool
            // maintainers who want to be Rokit-compatible, by examining the artifacts
            let artifact_names = artifacts
                .iter()
                .filter_map(|artifact| artifact.name.as_deref())
                .collect::<Vec<_>>();
            tracing::debug!(
                %tool_id,
                missing_os_all = %artifact_names.iter().all(|s| OS::detect(s).is_none()),
                missing_arch_all = %artifact_names.iter().all(|s| Arch::detect(s).is_none()),
                "missing compatible artifact or fallback for tool"
            );
        }
    }

    // If we did not find a compatible artifact, either directly
    // or through a fallback mechanism, this should be a hard error
    artifact_opt.with_context(|| format!("No compatible artifact found for {tool_id}"))
}
