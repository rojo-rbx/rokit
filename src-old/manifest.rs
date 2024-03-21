use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{bail, format_err, Context};
use serde::{Deserialize, Serialize};
use toml_edit::DocumentMut;

use crate::config::{write_if_not_exists, write_only_new};
use crate::home::Home;
use crate::tool_alias::ToolAlias;
use crate::tool_id::ToolId;

pub static MANIFEST_FILE_NAME: &str = "aftman.toml";

static DEFAULT_MANIFEST: &str = r#"
# This file lists tools managed by Aftman, a cross-platform toolchain manager.
# For more information, see https://github.com/LPGhatguy/aftman

# To add a new tool, add an entry to this table.
[tools]
# rojo = "rojo-rbx/rojo@6.2.0"
"#;

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub tools: BTreeMap<ToolAlias, ToolId>,

    /// The path that this manifest was loaded from if it was loaded from a file.
    #[serde(skip)]
    pub path: Option<PathBuf>,
}

impl Manifest {
    /// Create an empty global Aftman manifest if there isn't one already.
    pub fn init_global(home: &Home) -> anyhow::Result<()> {
        let base_dir = home.path();
        fs::create_dir_all(base_dir)?;

        let manifest_path = base_dir.join(MANIFEST_FILE_NAME);
        write_if_not_exists(&manifest_path, DEFAULT_MANIFEST.trim())?;

        Ok(())
    }

    pub fn init_local(base_dir: &Path) -> anyhow::Result<()> {
        let manifest_path = base_dir.join(MANIFEST_FILE_NAME);
        write_only_new(&manifest_path, DEFAULT_MANIFEST.trim())?;

        Ok(())
    }

    /// Find and load all manifests from the current directory, sorted in
    /// priority order.
    pub fn discover(home: &Home, mut current_dir: &Path) -> anyhow::Result<Vec<Manifest>> {
        let mut manifests = Vec::new();

        // Starting in the current directory, find every manifest file,
        // prioritizing manifests that are closer to our current dir.
        loop {
            if let Some(manifest) = Self::load_from_dir(current_dir)? {
                manifests.push(manifest);
            }

            match current_dir.parent() {
                Some(parent) => current_dir = parent,
                None => break,
            }
        }

        // We'll also load the user's global config, usually from
        // ~/.aftman/aftman.toml.
        if let Some(manifest) = Self::load_from_dir(home.path())? {
            manifests.push(manifest);
        }

        Ok(manifests)
    }

    /// Try to load an Aftman manifest from a directory containing an
    /// aftman.toml file.
    pub fn load_from_dir(path: &Path) -> anyhow::Result<Option<Manifest>> {
        let file_path = path.join(MANIFEST_FILE_NAME);

        let contents = match fs::read_to_string(&file_path) {
            Ok(contents) => contents,
            Err(err) => {
                if err.kind() == io::ErrorKind::NotFound {
                    return Ok(None);
                }

                bail!(err);
            }
        };

        let mut manifest: Manifest = toml::from_str(&contents)
            .with_context(|| format_err!("Invalid manifest at {}", file_path.display()))?;

        manifest.path = Some(file_path);

        Ok(Some(manifest))
    }

    /// Add the given alias and tool ID to the nearest manifest file.
    pub fn add_local_tool(
        home: &Home,
        mut current_dir: &Path,
        alias: &ToolAlias,
        id: &ToolId,
    ) -> anyhow::Result<()> {
        // Starting in the current directory, find every manifest file,
        // prioritizing manifests that are closer to our current dir.
        let mut manifest_path = None;
        loop {
            let file_path = current_dir.join(MANIFEST_FILE_NAME);
            if file_path.is_file() {
                manifest_path = Some(file_path);
                break;
            }

            match current_dir.parent() {
                Some(parent) => current_dir = parent,
                None => break,
            }
        }

        let manifest_path = match manifest_path {
            Some(v) => v,
            None => home.path().join(MANIFEST_FILE_NAME),
        };

        Self::add_tool(&manifest_path, alias, id)?;

        Ok(())
    }

    pub fn add_global_tool(home: &Home, alias: &ToolAlias, id: &ToolId) -> anyhow::Result<()> {
        let manifest_path = home.path().join(MANIFEST_FILE_NAME);
        Self::add_tool(&manifest_path, alias, id)?;

        Ok(())
    }

    fn add_tool(manifest_path: &Path, alias: &ToolAlias, id: &ToolId) -> anyhow::Result<()> {
        let content = fs::read_to_string(manifest_path)?;
        let mut document: DocumentMut = content.parse()?;
        document["tools"][alias.as_ref()] = toml_edit::value(id.to_string());

        fs::write(manifest_path, document.to_string())?;

        tracing::info!(
            "Tool {alias} = {id} has been added to {}",
            manifest_path.display()
        );

        Ok(())
    }
}
