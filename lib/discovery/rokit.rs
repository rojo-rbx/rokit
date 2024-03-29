use std::collections::HashMap;

use crate::{
    manifests::RokitManifest,
    tool::{ToolAlias, ToolSpec},
};

use super::Manifest;

impl Manifest for RokitManifest {
    fn home_dir() -> &'static str {
        ".rokit"
    }

    fn manifest_file_name() -> &'static str {
        "rokit.toml"
    }

    fn parse_manifest(contents: &str) -> Option<Self>
    where
        Self: Sized,
    {
        contents.parse().ok()
    }

    fn into_tools(self) -> HashMap<ToolAlias, ToolSpec> {
        self.tool_specs().into_iter().collect()
    }
}
