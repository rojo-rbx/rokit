#![allow(clippy::should_implement_trait)]
#![allow(clippy::inherent_to_string)]

use std::{collections::BTreeSet, convert::Infallible, str::FromStr, sync::Arc};

use dashmap::DashSet;
use semver::Version;

use crate::tool::{ToolId, ToolSpec};

/**
    Storage for installed tool specifications.

    Can be cheaply cloned while still
    referring to the same underlying data.
*/
#[derive(Debug, Default, Clone)]
pub struct InstalledStorage {
    tools: Arc<DashSet<ToolSpec>>,
}

impl InstalledStorage {
    /**
        Create a new, **empty** `InstalledStorage`.
    */
    pub fn new() -> Self {
        Self::default()
    }

    /**
        Parse the contents of a string into a `InstalledStorage`.

        Note that this is not fallible - any invalid
        lines or tool specifications will simply be ignored.

        This means that, worst case, if the installed storage file is corrupted,
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
        Add a tool to this `InstalledStorage`.

        Returns `true` if the tool was added and not already trusted.
    */
    pub fn add_spec(&self, tool: ToolSpec) -> bool {
        self.tools.insert(tool)
    }

    /**
        Remove a tool from this `InstalledStorage`.

        Returns `true` if the tool was previously trusted and has now been removed.
    */
    pub fn remove_spec(&self, tool: &ToolSpec) -> bool {
        self.tools.remove(tool).is_some()
    }

    /**
        Check if a tool is cached in this `InstalledStorage`.
    */
    pub fn is_installed(&self, tool: &ToolSpec) -> bool {
        self.tools.contains(tool)
    }

    /**
        Get a sorted copy of the installed tools in this `InstalledStorage`.
    */
    pub fn all_specs(&self) -> Vec<ToolSpec> {
        let mut sorted_tools = self.tools.iter().map(|id| id.clone()).collect::<Vec<_>>();
        sorted_tools.sort();
        sorted_tools
    }

    /**
        Get a sorted list of all unique tool identifiers in this `InstalledStorage`.
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
        Get a sorted list of all unique versions for a
        given tool identifier in this `InstalledStorage`.
    */
    pub fn all_versions_for_id(&self, id: &ToolId) -> Vec<Version> {
        let sorted_set = self
            .all_specs()
            .into_iter()
            .filter_map(|spec| {
                if ToolId::from(spec.clone()) == *id {
                    Some(spec.version().clone())
                } else {
                    None
                }
            })
            .collect::<BTreeSet<_>>();
        sorted_set.into_iter().collect()
    }

    /**
        Render the contents of this `InstalledStorage` to a string.

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
}

impl FromStr for InstalledStorage {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(InstalledStorage::from_str(s))
    }
}

impl ToString for InstalledStorage {
    fn to_string(&self) -> String {
        self.to_string()
    }
}
