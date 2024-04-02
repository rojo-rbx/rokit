use std::str::FromStr;

use serde_with::DeserializeFromStr;

use rokit::tool::{ToolAlias, ToolId, ToolSpec};

use super::constants::get_known_tool;

/**
    A tool identifier *or* specification, which includes
    the author, name, and *maybe* a version of a tool.

    Can also be parsed from common Roblox tool aliases, meaning:

    - `rojo` will become `rojo-rbx/rojo`
    - `lune` will become `lune-org/lune`

    As well as a bunch of other tools that are
    well-known in the greater Roblox community.

    See [`ToolId`] and [`ToolSpec`] for more information.
*/
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, DeserializeFromStr)]
pub enum ToolIdOrSpec {
    Id(ToolId),
    Spec(ToolSpec),
}

impl FromStr for ToolIdOrSpec {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('@') {
            Ok(Self::Spec(s.parse()?))
        } else if let Some(id) = get_known_tool(s) {
            Ok(Self::Id(id.clone()))
        } else {
            Ok(Self::Id(s.parse()?))
        }
    }
}

impl From<ToolId> for ToolIdOrSpec {
    fn from(id: ToolId) -> Self {
        Self::Id(id)
    }
}

impl From<ToolSpec> for ToolIdOrSpec {
    fn from(spec: ToolSpec) -> Self {
        Self::Spec(spec)
    }
}

impl From<ToolIdOrSpec> for ToolId {
    fn from(id_or_spec: ToolIdOrSpec) -> Self {
        match id_or_spec {
            ToolIdOrSpec::Id(id) => id,
            ToolIdOrSpec::Spec(spec) => spec.into(),
        }
    }
}

impl From<ToolIdOrSpec> for ToolAlias {
    fn from(id_or_spec: ToolIdOrSpec) -> Self {
        let name = match id_or_spec {
            ToolIdOrSpec::Id(id) => id.name().to_string(),
            ToolIdOrSpec::Spec(spec) => spec.name().to_string(),
        };
        Self::from_str(&name).expect("Derived alias is always valid")
    }
}
