use std::collections::HashMap;

use serde::Deserialize;

use crate::tool::{ToolAlias, ToolSpec};

use super::Manifest;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
struct AftmanAlias(ToolAlias);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
struct AftmanTool(ToolSpec);

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct AftmanManifest {
    tools: HashMap<AftmanAlias, AftmanTool>,
}

impl Manifest for AftmanManifest {
    fn home_dir() -> &'static str {
        ".aftman"
    }

    fn manifest_file_name() -> &'static str {
        "aftman.toml"
    }

    fn parse_manifest(contents: &str) -> Option<Self>
    where
        Self: Sized,
    {
        toml::from_str(contents).ok()
    }

    fn into_tools(self) -> HashMap<ToolAlias, ToolSpec> {
        self.tools
            .into_iter()
            .map(|(alias, tool)| (alias.0, tool.0))
            .collect()
    }
}
