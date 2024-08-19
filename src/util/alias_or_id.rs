use std::str::FromStr;

use serde_with::DeserializeFromStr;

use rokit::tool::{ToolAlias, ToolId};

/**
    A tool alias *or* identifier.

    See [`ToolAlias`] and [`ToolId`] for more information.
*/
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, DeserializeFromStr)]
pub enum ToolAliasOrId {
    Alias(ToolAlias),
    Id(ToolId),
}

impl FromStr for ToolAliasOrId {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('/') {
            Ok(Self::Id(s.parse()?))
        } else {
            Ok(Self::Alias(s.parse()?))
        }
    }
}

impl From<ToolAlias> for ToolAliasOrId {
    fn from(alias: ToolAlias) -> Self {
        Self::Alias(alias)
    }
}

impl From<ToolId> for ToolAliasOrId {
    fn from(id: ToolId) -> Self {
        Self::Id(id)
    }
}

impl From<ToolAliasOrId> for ToolAlias {
    fn from(id_or_spec: ToolAliasOrId) -> Self {
        let name = match id_or_spec {
            ToolAliasOrId::Alias(alias) => alias.name().to_string(),
            ToolAliasOrId::Id(id) => id.name().to_string(),
        };
        Self::from_str(&name).expect("Derived alias is always valid")
    }
}
