use std::{collections::HashMap, str::FromStr};

use semver::Version;
use toml_edit::{DocumentMut, InlineTable, Table};
use tracing::warn;

use crate::tool::{ToolAlias, ToolId, ToolSpec, util::to_xyz_version};

use super::Manifest;

enum SpecType {
    InlineTable(InlineTable),
    Table(Table),
}

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
            .inspect_err(|e| {
                warn!(
                    "A Foreman manifest could not be parsed!\
                    \nThe manifest will be ignored and its tools may not be available.\
                    \nError: {e}",
                );
            })
            .ok()
    }

    fn into_tools(self) -> HashMap<ToolAlias, ToolSpec> {
        let mut tools = HashMap::new();
        if let Some(map) = self.document.get("tools").and_then(|t| t.as_table()) {
            for (alias, tool_def) in map {
                let Ok(tool_alias) = alias.parse::<ToolAlias>().inspect_err(|e| {
                    warn!(
                        "A Foreman tool alias could not be parsed!\
                        \nThe tool will be ignored and may not be available.\
                        \nAlias: {alias}\
                        \nError: {e}",
                    );
                }) else {
                    continue;
                };

                let Some(spec) = tool_def
                    .as_inline_table()
                    .cloned()
                    .map(SpecType::InlineTable)
                    .or_else(|| tool_def.as_table().cloned().map(SpecType::Table))
                    .and_then(parse_foreman_tool_definition)
                else {
                    warn!(
                        "A Foreman tool spec with alias '{tool_alias}' could not be parsed!\
                        \nThe tool will be ignored and may not be available.",
                    );
                    continue;
                };

                tools.insert(tool_alias, spec);
            }
        }
        tools
    }
}

fn parse_foreman_tool_definition(map: SpecType) -> Option<ToolSpec> {
    let map = match map {
        SpecType::InlineTable(table) => table,
        SpecType::Table(table) => table.into_inline_table(),
    };

    let version = map.get("version").and_then(|t| t.as_str()).and_then(|v| {
        // TODO: Support real version requirements instead of just exact/min versions
        let without_prefix = v.trim_start_matches('=').trim_start_matches('^');
        let version_str = to_xyz_version(without_prefix);
        version_str.parse::<Version>().ok()
    })?;
    // TODO: Support gitlab tool ids
    let github_tool_id = map
        .get("github")
        .or(map.get("source"))
        .and_then(|t| t.as_str())
        .and_then(|s| s.parse::<ToolId>().ok());
    github_tool_id.map(|id| (id, version).into())
}
