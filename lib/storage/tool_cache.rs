#![allow(clippy::should_implement_trait)]
#![allow(clippy::inherent_to_string)]

use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::Arc,
};

use dashmap::DashSet;
use semver::Version;
use serde::{Deserialize, Serialize};
use tokio::{task::spawn_blocking, time::Instant};
use tracing::{instrument, trace};

use crate::{
    result::AftmanResult,
    tool::{ToolId, ToolSpec},
};

/**
    Cache for trusted tool identifiers and installed tool specifications.

    Can be cheaply cloned while still referring to the same underlying data.
*/
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ToolCache {
    trusted: Arc<DashSet<ToolId>>,
    installed: Arc<DashSet<ToolSpec>>,
}

impl ToolCache {
    /**
        Create a new, **empty** `ToolCache`.
    */
    pub fn new() -> Self {
        Self::default()
    }

    /**
        Add trust for a tool to this `ToolCache`.

        Returns `true` if the tool was added and not already trusted.
    */
    pub fn add_trust(&self, tool: ToolId) -> bool {
        self.trusted.insert(tool)
    }

    /**
        Remove trust for a tool from this `ToolCache`.

        Returns `true` if the tool was previously trusted and has now been removed.
    */
    pub fn remove_trust(&self, tool: &ToolId) -> bool {
        self.trusted.remove(tool).is_some()
    }

    /**
        Check if a tool is trusted by this `ToolCache`.
    */
    pub fn is_trusted(&self, tool: &ToolId) -> bool {
        self.trusted.contains(tool)
    }

    /**
        Get a sorted copy of the trusted tools in this `ToolCache`.
    */
    pub fn all_trusted(&self) -> Vec<ToolId> {
        let mut sorted_tools = self.trusted.iter().map(|id| id.clone()).collect::<Vec<_>>();
        sorted_tools.sort();
        sorted_tools
    }

    /**
        Add a tool to this `ToolCache`.

        Returns `true` if the tool was added and not already cached.
    */
    pub fn add_installed(&self, tool: ToolSpec) -> bool {
        self.installed.insert(tool)
    }

    /**
        Remove a tool from this `ToolCache`.

        Returns `true` if the tool was previously cached and has now been removed.
    */
    pub fn remove_installed(&self, tool: &ToolSpec) -> bool {
        self.installed.remove(tool).is_some()
    }

    /**
        Check if a tool is cached in this `ToolCache`.
    */
    pub fn is_installed(&self, tool: &ToolSpec) -> bool {
        self.installed.contains(tool)
    }

    /**
        Get a sorted copy of the cached tools in this `ToolCache`.
    */
    pub fn all_installed(&self) -> Vec<ToolSpec> {
        let mut sorted_tools = self
            .installed
            .iter()
            .map(|id| id.clone())
            .collect::<Vec<_>>();
        sorted_tools.sort();
        sorted_tools
    }

    /**
        Get a sorted list of all unique tool identifiers in this `ToolCache`.

        Note that this will deduplicate any tools with the same identifier,
        and only one identifier will be returned for each unique tool.
    */
    pub fn all_installed_ids(&self) -> Vec<ToolId> {
        let sorted_set = self
            .all_installed()
            .into_iter()
            .map(ToolId::from)
            .collect::<BTreeSet<_>>();
        sorted_set.into_iter().collect()
    }

    /**
        Get a sorted list of all unique versions for
        a given tool identifier in this `ToolCache`.
    */
    pub fn all_installed_versions_for_id(&self, id: &ToolId) -> Vec<Version> {
        let sorted_set = self
            .all_installed()
            .into_iter()
            .filter_map(|spec| {
                if spec.matches_id(id) {
                    Some(spec.version().clone())
                } else {
                    None
                }
            })
            .collect::<BTreeSet<_>>();
        sorted_set.into_iter().collect()
    }

    fn path(home_path: impl AsRef<Path>) -> PathBuf {
        home_path.as_ref().join("tool-storage").join("cache.json")
    }

    #[instrument(skip(home_path), level = "trace")]
    pub(crate) async fn load(home_path: impl AsRef<Path>) -> AftmanResult<Self> {
        let start = Instant::now();
        let path = Self::path(home_path);
        let this = load_impl(path.clone()).await?;
        trace!(?path, elapsed = ?start.elapsed(), "Loading tool cache");
        Ok(this)
    }

    #[instrument(skip(self, home_path), level = "trace")]
    pub(crate) async fn save(&self, home_path: impl AsRef<Path>) -> AftmanResult<()> {
        let start = Instant::now();
        let path = Self::path(home_path);
        save_impl(path.clone(), self).await?;
        trace!(?path, elapsed = ?start.elapsed(), "Saved tool cache");
        Ok(())
    }
}

// NOTE: Using std::fs here and passing a reader to serde_json lets us
// deserialize the cache faster and without storing the file in memory.
async fn load_impl(path: PathBuf) -> AftmanResult<ToolCache> {
    Ok(spawn_blocking(move || {
        use std::{
            fs::File,
            io::{BufReader, Error},
        };
        let reader = BufReader::new(File::open(path)?);
        let this: ToolCache = serde_json::from_reader(reader)?;
        Ok::<_, Error>(this)
    })
    .await?
    .unwrap_or_default())
}

// Same as in our load implementation, see notes above.
async fn save_impl(path: PathBuf, cache: &ToolCache) -> AftmanResult<()> {
    // NOTE: We save using sorted json arrays here, which is
    // compatible with the deserialize implementation for DashSet,
    // while also being easier to read for any human inspectors.
    let json = serde_json::json!({
        "trusted": cache.all_trusted(),
        "installed": cache.all_installed(),
    });

    spawn_blocking(move || {
        use std::{
            fs::{create_dir_all, File},
            io::{BufWriter, Error},
        };
        create_dir_all(path.parent().unwrap())?;
        let writer = BufWriter::new(File::create(path)?);
        serde_json::to_writer(writer, &json)?;
        Ok::<_, Error>(())
    })
    .await??;

    Ok(())
}
