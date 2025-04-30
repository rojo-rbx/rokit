#![allow(clippy::to_string_trait_impl)]
// NOTE: We don't want to implement Display here since it may
// make library consumers think that Rokit manifests are meant
// to be displayed - they are only meant to be stringified.

use std::{path::Path, str::FromStr};

use toml_edit::{DocumentMut, Formatted, Item, Value};
use tracing::warn;

use crate::{
    result::{RokitError, RokitResult},
    tool::{ToolAlias, ToolSpec},
    util::fs::{load_from_file, save_to_file},
};

pub const MANIFEST_FILE_NAME: &str = "rokit.toml";
pub(super) const MANIFEST_DEFAULT_CONTENTS: &str = "
# This file lists tools managed by Rokit, a toolchain manager for Roblox projects.
# For more information, see <|REPOSITORY_URL|>

# New tools can be added by running `rokit add <tool>` in a terminal.

[tools]
";

/**
    Rokit manifest file.

    Lists tools managed by Rokit.
*/
#[derive(Debug, Clone)]
pub struct RokitManifest {
    document: DocumentMut,
}

impl RokitManifest {
    /**
        Loads the manifest from the given directory, or creates a new one if it doesn't exist.

        If the manifest doesn't exist, a new one will be created with default contents and saved.

        See [`RokitManifest::load`] and [`RokitManifest::save`] for more information.

        # Errors

        - If the manifest could not be loaded or created.
    */
    pub async fn load_or_create(dir: impl AsRef<Path>) -> RokitResult<Self> {
        let path = dir.as_ref().join(MANIFEST_FILE_NAME);
        match load_from_file(path).await {
            Ok(manifest) => Ok(manifest),
            Err(RokitError::FileNotFound(_)) => {
                let new = Self::default();
                new.save(dir).await?;
                Ok(new)
            }
            Err(e) => Err(e),
        }
    }

    /**
        Loads the manifest from the given directory.

        This will search for a file named `rokit.toml` in the given directory.

        # Errors

        - If the manifest file could not be loaded.
    */
    #[tracing::instrument(skip(dir), level = "trace")]
    pub async fn load(dir: impl AsRef<Path>) -> RokitResult<Self> {
        let path = dir.as_ref().join(MANIFEST_FILE_NAME);
        tracing::trace!(?path, "Loading manifest");
        load_from_file(path).await
    }

    /**
        Saves the manifest to the given directory.

        This will write the manifest to a file named `rokit.toml` in the given directory.

        # Errors

        - If the manifest could not be saved.
    */
    #[tracing::instrument(skip(self, dir), level = "trace")]
    pub async fn save(&self, dir: impl AsRef<Path>) -> RokitResult<()> {
        let path = dir.as_ref().join(MANIFEST_FILE_NAME);
        tracing::trace!(?path, "Saving manifest");
        save_to_file(path, self.clone()).await
    }

    /**
        Checks if the manifest has a tool with the given alias.
    */
    #[must_use]
    pub fn has_tool(&self, alias: &ToolAlias) -> bool {
        let tools = self.document.get("tools").and_then(|v| v.as_table());
        tools.is_some_and(|t| t.contains_key(alias.name()))
    }

    /**
        Gets a tool specification from the manifest by its alias, if it exists.
    */
    #[must_use]
    pub fn get_tool(&self, alias: &ToolAlias) -> Option<ToolSpec> {
        let tools = self.document.get("tools")?.as_table()?;
        let tool_str = tools.get(alias.name())?.as_str()?;
        tool_str.parse::<ToolSpec>().ok()
    }

    /**
        Adds a tool to the manifest.

        If the tool already exists, this will return `false` and do nothing.
    */
    pub fn add_tool(&mut self, alias: &ToolAlias, spec: &ToolSpec) -> bool {
        let doc = self.document.as_table_mut();
        if !doc.contains_table("tools") {
            doc.insert("tools", toml_edit::table());
        }
        let tools = doc["tools"].as_table_mut().unwrap();
        if tools.contains_value(alias.name()) {
            false
        } else {
            tools.insert(
                alias.name(),
                Item::Value(Value::String(Formatted::new(spec.to_string()))),
            );
            true
        }
    }

    /**
        Updates a tool in the manifest with a new tool specification.

        If the tool doesn't exist, this will return `false` and do nothing.
    */
    pub fn update_tool(&mut self, alias: &ToolAlias, spec: &ToolSpec) -> bool {
        let doc = self.document.as_table_mut();
        if !doc.contains_table("tools") {
            return false;
        }
        let tools = doc["tools"].as_table_mut().unwrap();
        if tools.contains_value(alias.name()) {
            tools.insert(
                alias.name(),
                Item::Value(Value::String(Formatted::new(spec.to_string()))),
            );
            true
        } else {
            false
        }
    }

    /**
        Returns all valid tool specifications in the manifest.

        This will ignore any tools that are not valid tool specifications.
    */
    #[must_use]
    pub fn tool_specs(&self) -> Vec<(ToolAlias, ToolSpec)> {
        let tools = self.document.get("tools").and_then(|v| v.as_table());
        let tool_kv_pairs = tools.map(|t| t.get_values()).unwrap_or_default();
        tool_kv_pairs
            .into_iter()
            .filter_map(|(keys, value)| {
                let alias = keys.last()?.parse::<ToolAlias>().ok()?;
                let spec = value.as_str()?.parse::<ToolSpec>().ok()?;
                Some((alias, spec))
            })
            .collect()
    }
}

impl FromStr for RokitManifest {
    type Err = toml_edit::TomlError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut document = s.parse::<DocumentMut>()?;

        /*
            Check for invalid tool aliases and specs and warn the user about them
            as a preprocessing step. We do this here instead of when accessed in
            manifest methods to avoid duplicate warnings being emitted.

            Note that we do not check if the 'tools' table is missing here,
            since that should be handled gracefully and created if necessary.
            We do still check that it is of the correct type, and fix it if it isn't.
        */
        let tools = match document.get("tools") {
            None => None,
            Some(t) => {
                if let Some(t) = t.as_table() {
                    Some(t)
                } else {
                    warn!(
                        "Encountered an invalid 'tools' value in a Rokit manifest!\
                        The value will be replaced with an empty table.\
                        Any existing value has been overwritten."
                    );
                    document.insert("tools", toml_edit::table());
                    Some(
                        document
                            .get("tools")
                            .expect("table was inserted")
                            .as_table()
                            .expect("inserted table is a table"),
                    )
                }
            }
        };

        // Check all of the tools.
        let tool_kv_pairs = tools.map(|t| t.get_values()).unwrap_or_default();
        for (keys, value) in tool_kv_pairs {
            if let Err(e) = keys.last().unwrap().parse::<ToolAlias>() {
                warn!(
                    "A tool alias could not be parsed!\
                    \nThe tool will be ignored and may not be available.\
                    \nError: {e}",
                );
            }
            let Some(spec_str) = value.as_str() else {
                warn!(
                    "A tool spec with alias '{}' could not be parsed!\
                    \nThe tool will be ignored and may not be available.\
                    \nExpected: String\
                    \nActual: {}",
                    keys.into_iter().next_back().unwrap(),
                    value.type_name()
                );
                continue;
            };
            if let Err(e) = spec_str.parse::<ToolSpec>() {
                warn!(
                    "A tool spec with alias '{}' could not be parsed!\
                    \nThe tool will be ignored and may not be available.\
                    \nError: {e}",
                    keys.into_iter().next_back().unwrap(),
                );
            }
        }

        Ok(Self { document })
    }
}

impl ToString for RokitManifest {
    fn to_string(&self) -> String {
        self.document.to_string()
    }
}

impl Default for RokitManifest {
    fn default() -> Self {
        let document = super::make_manifest_template(MANIFEST_DEFAULT_CONTENTS)
            .parse::<DocumentMut>()
            .expect("default manifest template should be valid");
        Self { document }
    }
}
