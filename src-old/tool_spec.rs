use std::fmt;
use std::str::FromStr;

use anyhow::{format_err, Context};
use semver::Version;
use serde::de::{Deserialize, Deserializer, Error, Visitor};
use serde::ser::{Serialize, Serializer};

use crate::tool_id::ToolId;
use crate::tool_name::ToolName;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolSpec {
    name: ToolName,
    version: Option<Version>,
}

impl ToolSpec {
    pub fn new(name: ToolName, version: Option<Version>) -> Self {
        Self { name, version }
    }

    pub fn name(&self) -> &ToolName {
        &self.name
    }

    pub fn version(&self) -> Option<&Version> {
        self.version.as_ref()
    }

    #[allow(unused)]
    pub fn matches(&self, id: &ToolId) -> bool {
        if self.name() != id.name() {
            return false;
        }

        if let Some(version) = self.version() {
            version == id.version()
        } else {
            true
        }
    }
}

impl fmt::Display for ToolSpec {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match &self.version {
            Some(version) => write!(formatter, "{}@{}", self.name, version),
            None => write!(formatter, "{}", self.name),
        }
    }
}

impl FromStr for ToolSpec {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> anyhow::Result<Self> {
        let context = || {
            format_err!("Invalid Tool Spec \"{}\". It must be of the form SCOPE/NAME or SCOPE/NAME@VERSION.", value)
        };

        let mut name_version = value.splitn(2, '@');
        let name = name_version.next().unwrap();
        let name = ToolName::from_str(name).with_context(context)?;

        let version = match name_version.next() {
            None => None,
            Some(version_str) => {
                if version_str.is_empty() || version_str.chars().all(char::is_whitespace) {
                    return Err(format_err!("VERSION must be non-empty.")).with_context(context);
                }

                let version = version_str
                    .parse()
                    .context("Invalid version")
                    .with_context(context)?;
                Some(version)
            }
        };

        Ok(Self::new(name, version))
    }
}

impl Serialize for ToolSpec {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ToolSpec {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(ToolSpecVisitor)
    }
}

struct ToolSpecVisitor;

impl<'de> Visitor<'de> for ToolSpecVisitor {
    type Value = ToolSpec;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a Tool Spec of the form SCOPE/NAME or SCOPE/NAME@VERSION"
        )
    }

    fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
        value.parse().map_err(|err| E::custom(err))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Utility to create a ToolSpec for creating quick test cases.
    fn spec(scope: &str, name: &str, version: Option<&str>) -> ToolSpec {
        let name = ToolName::new(scope, name).expect("failed to create test ToolName");
        let version = version.map(|v| Version::parse(v).expect("failed to create test Version"));
        ToolSpec::new(name, version)
    }

    #[test]
    fn parse_success() {
        fn test(input: &str, expected: ToolSpec) {
            let parsed: ToolSpec = input.parse().expect("failed to parse ToolSpec");
            assert_eq!(parsed, expected);
        }

        test("a/b", spec("a", "b", None));
        test("hello/world", spec("hello", "world", None));
        test("a/b@1.0.0", spec("a", "b", Some("1.0.0")));
    }

    #[test]
    fn parse_failure() {
        fn test(input: &str, fragments: &[&str]) {
            let result: Result<ToolSpec, _> = input.parse();
            let err = format!("{:?}", result.expect_err("succeeded parsing bad ToolSpec"));
            let err_lowercase = err.to_lowercase();

            if fragments.is_empty() {
                panic!(
                    "Debug output, no fragments specified. Error message:\n{}",
                    err
                );
            }

            for fragment in fragments {
                if !err_lowercase.contains(fragment) {
                    panic!(
                        "Expected error to contain '{}' but it did not. Error:\n{}",
                        fragment, err
                    );
                }
            }
        }

        test("", &["name is missing"]);
        test("abc", &["name is missing", "abc"]);

        test("/", &["scope must be non-empty"]);
        test("/abc", &["scope must be non-empty"]);

        test("abc/", &["name must be non-empty"]);
        test("abc/ ", &["name must be non-empty"]);

        test("abc/abc@", &["version must be non-empty"]);
        test("abc/abc@1", &["invalid version"]);
    }

    #[test]
    fn parse_json() {
        fn test(input: &str, expected: ToolSpec) {
            let parsed: ToolSpec = serde_json::from_str(input).expect("failed to parse ToolSpec");
            assert_eq!(parsed, expected);
        }

        test(r#""abc/abc@1.0.0""#, spec("abc", "abc", Some("1.0.0")));
    }
}
