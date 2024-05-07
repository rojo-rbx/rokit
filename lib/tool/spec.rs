use std::{fmt, str::FromStr};

use semver::{Version, VersionReq};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use thiserror::Error;

use crate::sources::ArtifactProvider;

use super::{util::is_invalid_identifier, ToolId, ToolIdParseError};

/**
    Error type representing the possible errors that can occur when parsing a `ToolSpec`.
*/
#[derive(Debug, Error)]
pub enum ToolSpecParseError {
    #[error("tool spec is empty")]
    Empty,
    #[error("missing '@' separator")]
    MissingVersionSeparator,
    #[error(transparent)]
    IdParseError(#[from] ToolIdParseError),
    #[error("version '{0}' is invalid")]
    InvalidVersion(String),
    #[error(transparent)]
    VersionParseError(#[from] semver::Error),
    #[error(
        "{0}\nNote: It seems like you may be trying to use a version \
        requirement, which is not supported in Rokit. To use this tool, \
        specify an exact version instead."
    )]
    VersionParseErrorSuspectedVersionReq(String),
}

/**
    A tool specification, which includes the author, name, and version of a tool.

    This is an extension of [`ToolId`] used to uniquely identify
    a *specific version requirement* of a given tool.
*/
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, DeserializeFromStr, SerializeDisplay,
)]
pub struct ToolSpec {
    pub(crate) id: ToolId,
    pub(crate) version: Version,
}

impl ToolSpec {
    #[must_use]
    pub fn provider(&self) -> ArtifactProvider {
        self.id.provider()
    }

    #[must_use]
    pub fn author(&self) -> &str {
        self.id.author()
    }

    #[must_use]
    pub fn name(&self) -> &str {
        self.id.name()
    }

    #[must_use]
    pub fn id(&self) -> &ToolId {
        &self.id
    }

    #[must_use]
    pub fn version(&self) -> &Version {
        &self.version
    }

    #[must_use]
    pub fn matches_id(&self, id: &ToolId) -> bool {
        self.id == *id
    }
}

impl FromStr for ToolSpec {
    type Err = ToolSpecParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ToolSpecParseError::Empty);
        }

        let Some((before, after)) = s.split_once('@') else {
            return Err(ToolSpecParseError::MissingVersionSeparator);
        };

        let before = before.trim();
        let after = after.trim();

        let id = before.parse::<ToolId>()?;

        if is_invalid_identifier(after) {
            return Err(ToolSpecParseError::InvalidVersion(after.to_string()));
        }

        let version = match after.parse::<Version>() {
            Ok(version) => version,
            Err(e) => {
                return match after.parse::<VersionReq>() {
                    Ok(_) => Err(ToolSpecParseError::VersionParseErrorSuspectedVersionReq(
                        e.to_string(),
                    )),
                    Err(_) => Err(ToolSpecParseError::VersionParseError(e)),
                }
            }
        };

        Ok(ToolSpec { id, version })
    }
}

impl fmt::Display for ToolSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.id, self.version)
    }
}

impl From<(ToolId, Version)> for ToolSpec {
    fn from((id, version): (ToolId, Version)) -> Self {
        ToolSpec { id, version }
    }
}

impl From<ToolSpec> for ToolId {
    fn from(spec: ToolSpec) -> Self {
        spec.id.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_spec(author: &str, name: &str, version: &str) -> ToolSpec {
        ToolSpec {
            id: ToolId::from_str(&format!("{author}/{name}")).unwrap(),
            version: version.parse().unwrap(),
        }
    }

    #[test]
    fn parse_valid_basic() {
        // Basic strings should parse ok
        assert!("a/b@0.0.0".parse::<ToolSpec>().is_ok());
        assert!("author/name@1.2.3".parse::<ToolSpec>().is_ok());
        assert!("123abc456/78de90@11.22.33".parse::<ToolSpec>().is_ok());
        // The parsed ToolSpec should match the input
        assert_eq!(
            "a/b@0.0.0".parse::<ToolSpec>().unwrap(),
            new_spec("a", "b", "0.0.0"),
        );
        assert_eq!(
            "author/name@1.2.3".parse::<ToolSpec>().unwrap(),
            new_spec("author", "name", "1.2.3"),
        );
        assert_eq!(
            "123abc456/78de90@11.22.33".parse::<ToolSpec>().unwrap(),
            new_spec("123abc456", "78de90", "11.22.33"),
        );
    }

    #[test]
    fn parse_valid_extra_whitespace() {
        // Leading and trailing whitespace should be ignored
        assert!(" author/name@1.2.3 ".parse::<ToolSpec>().is_ok());
        assert!(" author / name @ 1.2.3 ".parse::<ToolSpec>().is_ok());
        // The trimmed whitespace should not be in the resulting ToolSpec
        let spec = new_spec("author", "name", "1.2.3");
        assert_eq!(" author/name@1.2.3 ".parse::<ToolSpec>().unwrap(), spec);
        assert_eq!(" author / name @ 1.2.3 ".parse::<ToolSpec>().unwrap(), spec);
    }

    #[test]
    fn parse_invalid_missing() {
        // Empty strings or parts should not be allowed
        assert!("".parse::<ToolSpec>().is_err());
        assert!("/".parse::<ToolSpec>().is_err());
        assert!("a/@".parse::<ToolSpec>().is_err());
        assert!("/b@".parse::<ToolSpec>().is_err());
        assert!("/@".parse::<ToolSpec>().is_err());
    }

    #[test]
    fn parse_invalid_extra_separator() {
        // Superfluous separators should not be allowed
        assert!("a/b@c@".parse::<ToolSpec>().is_err());
        assert!("a/b@c@d".parse::<ToolSpec>().is_err());
        assert!("a/b@c@d@e".parse::<ToolSpec>().is_err());
    }
}
