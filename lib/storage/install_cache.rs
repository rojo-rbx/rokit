#![allow(clippy::should_implement_trait)]
#![allow(clippy::inherent_to_string)]

use std::{
    collections::BTreeSet,
    convert::Infallible,
    path::{Path, MAIN_SEPARATOR},
    str::FromStr,
    sync::Arc,
};

use const_format::concatcp;
use dashmap::DashSet;
use semver::Version;

use crate::tool::{ToolId, ToolSpec};

use super::{
    util::{load_from_file, save_to_file},
    StorageResult,
};

const FILE_PATH_INSTALLED: &str = concatcp!("tool-storage", MAIN_SEPARATOR, "installed.txt");

/**
    Storage for installed tool specifications.

    Can be cheaply cloned while still
    referring to the same underlying data.
*/
#[derive(Debug, Default, Clone)]
pub struct InstallCache {
    tools: Arc<DashSet<ToolSpec>>,
}

impl InstallCache {
    /**
        Create a new, **empty** `InstallCache`.
    */
    pub fn new() -> Self {
        Self::default()
    }

    /**
        Parse the contents of a string into a `InstallCache`.

        Note that this is not fallible - any invalid
        lines or tool specifications will simply be ignored.

        This means that, worst case, if the installed cache file is corrupted,
        the user will simply have to re-install the tools they want to use.
    */
    pub fn from_str(s: impl AsRef<str>) -> Self {
        let tools = s
            .as_ref()
            .lines()
            .filter_map(|line| line.parse::<ToolSpec>().ok())
            .collect::<DashSet<_>>();
        Self {
            tools: Arc::new(tools),
        }
    }

    /**
        Add a tool to this `InstallCache`.

        Returns `true` if the tool was added and not already cached.
    */
    pub fn add_spec(&self, tool: ToolSpec) -> bool {
        self.tools.insert(tool)
    }

    /**
        Remove a tool from this `InstallCache`.

        Returns `true` if the tool was previously cached and has now been removed.
    */
    pub fn remove_spec(&self, tool: &ToolSpec) -> bool {
        self.tools.remove(tool).is_some()
    }

    /**
        Check if a tool is cached in this `InstallCache`.
    */
    pub fn is_installed(&self, tool: &ToolSpec) -> bool {
        self.tools.contains(tool)
    }

    /**
        Get a sorted copy of the cached tools in this `InstallCache`.
    */
    pub fn all_specs(&self) -> Vec<ToolSpec> {
        let mut sorted_tools = self.tools.iter().map(|id| id.clone()).collect::<Vec<_>>();
        sorted_tools.sort();
        sorted_tools
    }

    /**
        Get a sorted list of all unique tool identifiers in this `InstallCache`.

        Note that this will deduplicate any tools with the same identifier,
        and only one identifier will be returned for each unique tool.
    */
    pub fn all_ids(&self) -> Vec<ToolId> {
        let sorted_set = self
            .all_specs()
            .into_iter()
            .map(ToolId::from)
            .collect::<BTreeSet<_>>();
        sorted_set.into_iter().collect()
    }

    /**
        Get a sorted list of all unique versions for
        a given tool identifier in this `InstallCache`.
    */
    pub fn all_versions_for_id(&self, id: &ToolId) -> Vec<Version> {
        let sorted_set = self
            .all_specs()
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

    /**
        Render the contents of this `InstallCache` to a string.

        This will be a sorted list of all tool specifications, separated by newlines.
    */
    pub fn to_string(&self) -> String {
        let mut contents = self
            .all_specs()
            .into_iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        contents.push('\n');
        contents
    }

    pub(crate) async fn load(home_path: impl AsRef<Path>) -> StorageResult<Self> {
        let path = home_path.as_ref().join(FILE_PATH_INSTALLED);
        load_from_file(path).await
    }

    pub(crate) async fn save(&self, home_path: impl AsRef<Path>) -> StorageResult<()> {
        let path = home_path.as_ref().join(FILE_PATH_INSTALLED);
        save_to_file(path, self.clone()).await
    }
}

impl FromStr for InstallCache {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(InstallCache::from_str(s))
    }
}

impl ToString for InstallCache {
    fn to_string(&self) -> String {
        self.to_string()
    }
}
