use std::{collections::HashMap, str::FromStr};

use semver::Version;
use toml_edit::DocumentMut;

use crate::tool::{ToolAlias, ToolId, ToolSpec};

use super::Manifest;

#[derive(Debug, Clone)]
pub(crate) struct ForemanManifest {
    document: DocumentMut,
}

impl Manifest for ForemanManifest {
    fn home_dir() -> &'static str {
        ".foreman"
    }

    fn manifest_file_name() -> &'static str {
        "foreman.toml"
    }

    fn parse_manifest(contents: &str) -> Option<Self>
    where
        Self: Sized,
    {
        toml_edit::DocumentMut::from_str(contents)
            .map(|document| Self { document })
            .ok()
    }

    fn into_tools(self) -> HashMap<ToolAlias, ToolSpec> {
        let mut tools = HashMap::new();
        if let Some(map) = self.document.get("tools").and_then(|t| t.as_table()) {
            for (alias, tool_def) in map {
                let tool_alias = alias.parse::<ToolAlias>().ok();
                let tool_spec = tool_def.as_table().and_then(parse_foreman_tool_definition);
                if let (Some(alias), Some(spec)) = (tool_alias, tool_spec) {
                    tools.insert(alias, spec);
                }
            }
        }
        tools
    }
}

fn parse_foreman_tool_definition(map: &toml_edit::Table) -> Option<ToolSpec> {
    let version = map.get("version").and_then(|t| t.as_str()).and_then(|v| {
        // TODO: Support real version requirements instead of just exact/min versions
        let without_prefix = v.trim_start_matches('=').trim_start_matches('^');
        without_prefix.parse::<Version>().ok()
    })?;
    // TODO: Support gitlab tool ids
    let github_tool_id = map
        .get("github")
        .or(map.get("source"))
        .and_then(|t| t.as_str())
        .and_then(|s| s.parse::<ToolId>().ok());
    github_tool_id.map(|id| (id, version).into())
}
