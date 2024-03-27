use std::str::FromStr;

use serde_with::DeserializeFromStr;

use rokit::tool::{ToolAlias, ToolId, ToolSpec};

/**
    A tool alias *or* identifier *or* specification.

    See [`ToolAlias`], [`ToolId`] and [`ToolSpec`] for more information.
*/
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, DeserializeFromStr)]
pub enum ToolAliasOrIdOrSpec {
    Alias(ToolAlias),
    Id(ToolId),
    Spec(ToolSpec),
}

impl FromStr for ToolAliasOrIdOrSpec {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('@') {
            Ok(Self::Spec(s.parse()?))
        } else if s.contains('/') {
            Ok(Self::Id(s.parse()?))
        } else {
            Ok(Self::Alias(s.parse()?))
        }
    }
}

impl From<ToolAlias> for ToolAliasOrIdOrSpec {
    fn from(alias: ToolAlias) -> Self {
        Self::Alias(alias)
    }
}

impl From<ToolId> for ToolAliasOrIdOrSpec {
    fn from(id: ToolId) -> Self {
        Self::Id(id)
    }
}

impl From<ToolSpec> for ToolAliasOrIdOrSpec {
    fn from(spec: ToolSpec) -> Self {
        Self::Spec(spec)
    }
}

impl From<ToolAliasOrIdOrSpec> for ToolAlias {
    fn from(id_or_spec: ToolAliasOrIdOrSpec) -> Self {
        let name = match id_or_spec {
            ToolAliasOrIdOrSpec::Alias(alias) => alias.name().to_string(),
            ToolAliasOrIdOrSpec::Id(id) => id.name().to_string(),
            ToolAliasOrIdOrSpec::Spec(spec) => spec.name().to_string(),
        };
        Self::from_str(&name).expect("Derived alias is always valid")
    }
}
