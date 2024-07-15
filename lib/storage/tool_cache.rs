#![allow(clippy::should_implement_trait)]
#![allow(clippy::inherent_to_string)]

use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use dashmap::DashSet;
use semver::Version;
use serde::Deserialize;
use tokio::{fs::create_dir_all, task::spawn_blocking, time::Instant};
use tracing::{instrument, trace};

use crate::{
    result::RokitResult,
    tool::{ToolId, ToolSpec},
};

/**
    Cache for trusted tool identifiers and installed tool specifications.

    Can be cheaply cloned while still referring to the same underlying data.
*/
#[derive(Debug, Default, Clone, Deserialize)]
pub struct ToolCache {
    trusted: Arc<DashSet<ToolId>>,
    installed: Arc<DashSet<ToolSpec>>,
    #[serde(default, skip)]
    needs_saving: Arc<AtomicBool>,
}

impl ToolCache {
    /**
        Create a new, **empty** `ToolCache`.
    */
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /**
        Add trust for a tool to this `ToolCache`.

        Returns `true` if the tool was added and not already trusted.
    */
    #[must_use]
    pub fn add_trust(&self, tool: ToolId) -> bool {
        self.needs_saving.store(true, Ordering::SeqCst);
        self.trusted.insert(tool)
    }

    /**
        Remove trust for a tool from this `ToolCache`.

        Returns `true` if the tool was previously trusted and has now been removed.
    */
    #[must_use]
    pub fn remove_trust(&self, tool: &ToolId) -> bool {
        self.needs_saving.store(true, Ordering::SeqCst);
        self.trusted.remove(tool).is_some()
    }

    /**
        Check if a tool is trusted by this `ToolCache`.
    */
    #[must_use]
    pub fn is_trusted(&self, tool: &ToolId) -> bool {
        self.trusted.contains(tool)
    }

    /**
        Get a sorted copy of the trusted tools in this `ToolCache`.
    */
    #[must_use]
    pub fn all_trusted(&self) -> Vec<ToolId> {
        let mut sorted_tools = self.trusted.iter().map(|id| id.clone()).collect::<Vec<_>>();
        sorted_tools.sort();
        sorted_tools
    }

    /**
        Add a tool to this `ToolCache`.

        Returns `true` if the tool was added and not already cached.
    */
    #[must_use]
    pub fn add_installed(&self, tool: ToolSpec) -> bool {
        self.needs_saving.store(true, Ordering::SeqCst);
        self.installed.insert(tool)
    }

    /**
        Remove a tool from this `ToolCache`.

        Returns `true` if the tool was previously cached and has now been removed.
    */
    #[must_use]
    pub fn remove_installed(&self, tool: &ToolSpec) -> bool {
        self.needs_saving.store(true, Ordering::SeqCst);
        self.installed.remove(tool).is_some()
    }

    /**
        Check if a tool is cached in this `ToolCache`.
    */
    #[must_use]
    pub fn is_installed(&self, tool: &ToolSpec) -> bool {
        self.installed.contains(tool)
    }

    /**
        Get a sorted copy of the cached tools in this `ToolCache`.
    */
    #[must_use]
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
    #[must_use]
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
    pub(crate) async fn load(home_path: impl AsRef<Path>) -> RokitResult<Self> {
        let start = Instant::now();
        let path = Self::path(home_path);
        let this = load_impl(path.clone()).await?;
        trace!(?path, elapsed = ?start.elapsed(), "Loading tool cache");
        Ok(this)
    }

    #[instrument(skip(self, home_path), level = "trace")]
    pub(crate) async fn save(&self, home_path: impl AsRef<Path>) -> RokitResult<()> {
        self.needs_saving.store(false, Ordering::SeqCst);
        let start = Instant::now();
        let path = Self::path(home_path);
        save_impl(path.clone(), self).await?;
        trace!(?path, elapsed = ?start.elapsed(), "Saved tool cache");
        Ok(())
    }

    pub(crate) fn needs_saving(&self) -> bool {
        self.needs_saving.load(Ordering::SeqCst)
    }
}

async fn load_impl(path: PathBuf) -> RokitResult<ToolCache> {
    // Make sure we have created the directory for the cache file, since
    // OpenOptions::create will only create the file and not the directory.
    let dir = path
        .parent()
        .expect("should not be given empty or root path");
    create_dir_all(dir).await?;

    // NOTE: Using std::fs here and passing a reader to serde_json lets us
    // deserialize the cache faster and without storing the file in memory.
    let result = spawn_blocking(move || {
        use std::{
            fs::OpenOptions,
            io::{BufReader, Error},
        };

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;
        let reader = BufReader::new(file);
        let this: ToolCache = serde_json::from_reader(reader)?;

        Ok::<_, Error>(this)
    });

    let read_result = result
        .await
        .expect("blocking reader task panicked unexpectedly");
    Ok(read_result.unwrap_or_default())
}

async fn save_impl(path: PathBuf, cache: &ToolCache) -> RokitResult<()> {
    // NOTE: We save using sorted json arrays here, which is
    // compatible with the deserialize implementation for DashSet,
    // while also being easier to read for any human inspectors.
    let json = serde_json::json!({
        "trusted": cache.all_trusted(),
        "installed": cache.all_installed(),
    });

    // Same as in our load implementation, see notes there.
    let result = spawn_blocking(move || {
        use std::{
            fs::{create_dir_all, File},
            io::{BufWriter, Error},
        };
        create_dir_all(path.parent().unwrap())?;
        let writer = BufWriter::new(File::create(path)?);
        serde_json::to_writer(writer, &json)?;
        Ok::<_, Error>(())
    });

    result.await??;
    Ok(())
}
