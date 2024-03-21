#![allow(clippy::should_implement_trait)]
#![allow(clippy::inherent_to_string)]

use std::{collections::BTreeSet, convert::Infallible, str::FromStr};

use crate::tool::ToolId;

/**
    Storage for trusted tool identifiers.
*/
#[derive(Debug, Default)]
pub struct TrustStorage {
    tools: BTreeSet<ToolId>,
}

impl TrustStorage {
    /**
        Create a new, **empty** `TrustStorage`.
    */
    pub fn new() -> Self {
        Self::default()
    }

    /**
        Parse the contents of a string into a `TrustStorage`.

        Note that this is not fallible - any invalid
        lines or tool identifiers will simply be ignored.

        This means that, worst case, if the trust storage file is corrupted,
        the user will simply have to re-trust the tools they want to use.
    */
    pub fn from_str(s: impl AsRef<str>) -> Self {
        let tools = s
            .as_ref()
            .lines()
            .filter_map(|line| line.parse::<ToolId>().ok())
            .collect();
        Self { tools }
    }

    /**
        Add a tool to this `TrustStorage`.

        Returns `true` if the tool was added and not already trusted.
    */
    pub fn add_tool(&mut self, tool: ToolId) -> bool {
        self.tools.insert(tool)
    }

    /**
        Remove a tool from this `TrustStorage`.

        Returns `true` if the tool was previously trusted and has now been removed.
    */
    pub fn remove_tool(&mut self, tool: &ToolId) -> bool {
        self.tools.remove(tool)
    }

    /**
        Check if a tool is trusted by this `TrustStorage`.
    */
    pub fn is_trusted(&self, tool: &ToolId) -> bool {
        self.tools.contains(tool)
    }

    /**
        Get an iterator over the tools in this `TrustStorage`.
    */
    pub fn iter_tools(&self) -> impl Iterator<Item = &ToolId> {
        self.tools.iter()
    }

    /**
        Render the contents of this `TrustStorage` to a string.

        This will be a sorted list of all tool ids, separated by newlines.
    */
    pub fn to_string(&self) -> String {
        let mut contents = self
            .tools
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        contents.push('\n');
        contents
    }
}

impl FromStr for TrustStorage {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(TrustStorage::from_str(s))
    }
}

impl ToString for TrustStorage {
    fn to_string(&self) -> String {
        self.to_string()
    }
}
