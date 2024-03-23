#![allow(clippy::should_implement_trait)]
#![allow(clippy::inherent_to_string)]

use std::{convert::Infallible, path::Path, str::FromStr, sync::Arc};

use dashmap::DashSet;

use crate::tool::ToolId;

use super::{
    util::{load_from_file, save_to_file},
    StorageResult,
};

const FILE_PATH_TRUST: &str = "trusted.txt";

/**
    Cache for trusted tool identifiers.

    Can be cheaply cloned while still
    referring to the same underlying data.
*/
#[derive(Debug, Default, Clone)]
pub struct TrustCache {
    tools: Arc<DashSet<ToolId>>,
}

impl TrustCache {
    /**
        Create a new, **empty** `TrustCache`.
    */
    pub fn new() -> Self {
        Self::default()
    }

    /**
        Parse the contents of a string into a `TrustCache`.

        Note that this is not fallible - any invalid
        lines or tool identifiers will simply be ignored.

        This means that, worst case, if the trust cache file is corrupted,
        the user will simply have to re-trust the tools they want to use.
    */
    pub fn from_str(s: impl AsRef<str>) -> Self {
        let tools = s
            .as_ref()
            .lines()
            .filter_map(|line| line.parse::<ToolId>().ok())
            .collect::<DashSet<_>>();
        Self {
            tools: Arc::new(tools),
        }
    }

    /**
        Add a tool to this `TrustCache`.

        Returns `true` if the tool was added and not already trusted.
    */
    pub fn add_tool(&self, tool: ToolId) -> bool {
        self.tools.insert(tool)
    }

    /**
        Remove a tool from this `TrustCache`.

        Returns `true` if the tool was previously trusted and has now been removed.
    */
    pub fn remove_tool(&self, tool: &ToolId) -> bool {
        self.tools.remove(tool).is_some()
    }

    /**
        Check if a tool is trusted by this `TrustCache`.
    */
    pub fn is_trusted(&self, tool: &ToolId) -> bool {
        self.tools.contains(tool)
    }

    /**
        Get a sorted copy of the trusted tools in this `TrustCache`.
    */
    pub fn all_tools(&self) -> Vec<ToolId> {
        let mut sorted_tools = self.tools.iter().map(|id| id.clone()).collect::<Vec<_>>();
        sorted_tools.sort();
        sorted_tools
    }

    /**
        Render the contents of this `TrustCache` to a string.

        This will be a sorted list of all tool ids, separated by newlines.
    */
    pub fn to_string(&self) -> String {
        let mut contents = self
            .all_tools()
            .into_iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        contents.push('\n');
        contents
    }

    pub(crate) async fn load(home_path: impl AsRef<Path>) -> StorageResult<Self> {
        let path = home_path.as_ref().join(FILE_PATH_TRUST);
        load_from_file(path).await
    }

    pub(crate) async fn save(&self, home_path: impl AsRef<Path>) -> StorageResult<()> {
        let path = home_path.as_ref().join(FILE_PATH_TRUST);
        save_to_file(path, self.clone()).await
    }
}

impl FromStr for TrustCache {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(TrustCache::from_str(s))
    }
}

impl ToString for TrustCache {
    fn to_string(&self) -> String {
        self.to_string()
    }
}
