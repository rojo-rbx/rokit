use std::{fmt, str::FromStr};

use semver::Version;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use thiserror::Error;

use crate::sources::ArtifactProvider;

use super::{util::is_invalid_identifier, ToolAlias, ToolSpec};

/**
    Error type representing the possible errors that can occur when parsing a `ToolId`.
*/
#[derive(Debug, Error)]
pub enum ToolIdParseError {
    #[error("tool id is empty")]
    Empty,
    #[error("missing '/' separator")]
    MissingSeparator,
    #[error("artifact provider '{0}' is invalid")]
    InvalidProvider(String),
    #[error("author '{0}' is empty or invalid")]
    InvalidAuthor(String),
    #[error("name '{0}' is empty or invalid")]
    InvalidName(String),
}

/**
    A tool identifier, which includes the author and name of a tool.

    Also includes the provider of the artifact, which by default is `GitHub`.

    Used to uniquely identify a tool, but not its version.
*/
#[derive(Debug, Clone, PartialEq, Eq, Hash, DeserializeFromStr, SerializeDisplay)]
pub struct ToolId {
    pub(super) provider: ArtifactProvider,
    pub(super) author: String,
    pub(super) name: String,
}

impl ToolId {
    #[must_use]
    pub fn provider(&self) -> ArtifactProvider {
        self.provider
    }

    #[must_use]
    pub fn author(&self) -> &str {
        &self.author
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn into_spec(self, version: Version) -> ToolSpec {
        ToolSpec::from((self, version))
    }

    #[must_use]
    pub fn into_alias(self) -> ToolAlias {
        ToolAlias::from(self)
    }
}

impl Ord for ToolId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.author
            .cmp(&other.author)
            .then_with(|| self.name.cmp(&other.name))
    }
}

impl PartialOrd for ToolId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl FromStr for ToolId {
    type Err = ToolIdParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ToolIdParseError::Empty);
        }

        let (provider, after_provider) = match s.split_once(':') {
            None => (ArtifactProvider::default(), s),
            Some((left, right)) => {
                let provider = ArtifactProvider::from_str(left)
                    .map_err(|e| ToolIdParseError::InvalidProvider(e.to_string()))?;
                (provider, right)
            }
        };

        let Some((before, after)) = after_provider.split_once('/') else {
            return Err(ToolIdParseError::MissingSeparator);
        };

        let before = before.trim();
        let after = after.trim();

        if is_invalid_identifier(before) {
            return Err(ToolIdParseError::InvalidAuthor(before.to_string()));
        }
        if is_invalid_identifier(after) {
            return Err(ToolIdParseError::InvalidName(after.to_string()));
        }

        Ok(Self {
            provider,
            author: before.to_string(),
            name: after.to_string(),
        })
    }
}

impl fmt::Display for ToolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.author, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_id(author: &str, name: &str) -> ToolId {
        ToolId {
            provider: ArtifactProvider::default(),
            author: author.to_string(),
            name: name.to_string(),
        }
    }

    fn new_id_with_provider(provider: ArtifactProvider, author: &str, name: &str) -> ToolId {
        ToolId {
            provider,
            author: author.to_string(),
            name: name.to_string(),
        }
    }

    #[test]
    fn parse_valid_basic() {
        // Basic strings should parse ok
        assert!("a/b".parse::<ToolId>().is_ok());
        assert!("author/name".parse::<ToolId>().is_ok());
        assert!("123abc456/78de90".parse::<ToolId>().is_ok());
        // The parsed ToolId should match the input
        assert_eq!("a/b".parse::<ToolId>().unwrap(), new_id("a", "b"));
        assert_eq!(
            "author/name".parse::<ToolId>().unwrap(),
            new_id("author", "name")
        );
        assert_eq!(
            "123abc456/78de90".parse::<ToolId>().unwrap(),
            new_id("123abc456", "78de90")
        );
    }

    #[test]
    fn parse_valid_extra_whitespace() {
        // Leading and trailing whitespace should be trimmed and ok
        assert!("a/ b".parse::<ToolId>().is_ok());
        assert!("a/b ".parse::<ToolId>().is_ok());
        assert!("a /b".parse::<ToolId>().is_ok());
        assert!("a/ b".parse::<ToolId>().is_ok());
        assert!("a/b ".parse::<ToolId>().is_ok());
        // The trimmed whitespace should not be in the resulting ToolId
        let id = new_id("a", "b");
        assert_eq!("a/ b".parse::<ToolId>().unwrap(), id);
        assert_eq!("a/b ".parse::<ToolId>().unwrap(), id);
        assert_eq!("a /b".parse::<ToolId>().unwrap(), id);
        assert_eq!("a/ b".parse::<ToolId>().unwrap(), id);
        assert_eq!("a/b ".parse::<ToolId>().unwrap(), id);
    }

    #[test]
    fn parse_valid_provider() {
        // Known provider strings should parse ok
        assert!("github:a/b".parse::<ToolId>().is_ok());
        // The parsed ToolId should match the input
        assert_eq!(
            "github:a/b".parse::<ToolId>().unwrap(),
            new_id_with_provider(ArtifactProvider::GitHub, "a", "b")
        );
    }

    #[test]
    fn parse_invalid_missing() {
        // Empty strings or parts should not be allowed
        assert!("".parse::<ToolId>().is_err());
        assert!("/".parse::<ToolId>().is_err());
        assert!("a/".parse::<ToolId>().is_err());
        assert!("/b".parse::<ToolId>().is_err());
    }

    #[test]
    fn parse_invalid_extra_separator() {
        // Superfluous separators should not be allowed
        assert!("a/b/".parse::<ToolId>().is_err());
        assert!("a/b/c".parse::<ToolId>().is_err());
    }

    #[test]
    fn parse_invalid_provider() {
        // Empty provider should not be allowed
        assert!(":a/b".parse::<ToolId>().is_err());
        assert!(":a/b".parse::<ToolId>().is_err());
        assert!(":a/b".parse::<ToolId>().is_err());
        // Unrecognized provider should not be allowed
        assert!("unknown:a/b".parse::<ToolId>().is_err());
        assert!("hubgit:a/b".parse::<ToolId>().is_err());
        assert!("bitbab:a/b".parse::<ToolId>().is_err());
    }
}
