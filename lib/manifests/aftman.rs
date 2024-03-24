use std::{path::Path, str::FromStr};

use toml_edit::{DocumentMut, Formatted, Item, Value};

use crate::{
    result::{AftmanError, AftmanResult},
    tool::{ToolAlias, ToolSpec},
    util::{load_from_file_fallible, save_to_file},
};

pub const MANIFEST_FILE_NAME: &str = "aftman.toml";
const MANIFEST_DEFAULT_CONTENTS: &str = r#"
# This file lists tools managed by Aftman, a cross-platform toolchain manager.
# For more information, see <|REPOSITORY_URL|>

# New tools can be added by running `aftman add <tool>` in a terminal.
[tools]
"#;

/**
    Aftman manifest file.

    Lists tools managed by Aftman.
*/
#[derive(Debug, Clone)]
pub struct AftmanManifest {
    document: DocumentMut,
}

impl AftmanManifest {
    /**
        Loads the manifest from the given directory, or creates a new one if it doesn't exist.

        If the manifest doesn't exist, a new one will be created with default contents and saved.

        See [`AftmanManifest::load`] and [`AftmanManifest::save`] for more information.
    */
    pub async fn load_or_create(dir: impl AsRef<Path>) -> AftmanResult<Self> {
        let path = dir.as_ref().join(MANIFEST_FILE_NAME);
        match load_from_file_fallible(path).await {
            Ok(manifest) => Ok(manifest),
            Err(AftmanError::FileNotFound(_)) => {
                let new = Self::default();
                new.save(dir).await?;
                Ok(new)
            }
            Err(e) => Err(e),
        }
    }

    /**
        Loads the manifest from the given directory.

        This will search for a file named `aftman.toml` in the given directory.
    */
    #[tracing::instrument(skip(dir), level = "trace")]
    pub async fn load(dir: impl AsRef<Path>) -> AftmanResult<Self> {
        let path = dir.as_ref().join(MANIFEST_FILE_NAME);
        tracing::trace!(?path, "Loading manifest");
        load_from_file_fallible(path).await
    }

    /**
        Saves the manifest to the given directory.

        This will write the manifest to a file named `aftman.toml` in the given directory.
    */
    #[tracing::instrument(skip(self, dir), level = "trace")]
    pub async fn save(&self, dir: impl AsRef<Path>) -> AftmanResult<()> {
        let path = dir.as_ref().join(MANIFEST_FILE_NAME);
        tracing::trace!(?path, "Saving manifest");
        save_to_file(path, self.clone()).await
    }

    /**
        Checks if the manifest has a tool with the given alias.
    */
    pub fn has_tool(&self, alias: &ToolAlias) -> bool {
        let tools = self.document["tools"].as_table();
        tools.is_some_and(|t| t.contains_key(alias.name()))
    }

    /**
        Gets a tool specification from the manifest by its alias, if it exists.
    */
    pub fn get_tool(&self, alias: &ToolAlias) -> Option<ToolSpec> {
        let tools = self.document["tools"].as_table()?;
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
        if !tools.contains_value(alias.name()) {
            tools.insert(
                alias.name(),
                Item::Value(Value::String(Formatted::new(spec.to_string()))),
            );
            true
        } else {
            false
        }
    }
}

impl FromStr for AftmanManifest {
    type Err = toml_edit::TomlError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let document = s.parse::<DocumentMut>()?;
        Ok(Self { document })
    }
}

impl ToString for AftmanManifest {
    fn to_string(&self) -> String {
        self.document.to_string()
    }
}

impl Default for AftmanManifest {
    fn default() -> Self {
        let document = MANIFEST_DEFAULT_CONTENTS
            .replace("<|REPOSITORY_URL|>", env!("CARGO_PKG_REPOSITORY"))
            .parse::<DocumentMut>()
            .unwrap();
        Self { document }
    }
}
