use std::cmp::Ordering;

use semver::Version;

use crate::{
    descriptor::{Arch, OS},
    tool::ToolId,
    util::str::char_is_word_separator,
};

use super::Artifact;

/**
    Helper function to sort which artifact is preferred, based on
    heuristics such as which tool name mentions the artifact name
    more closely, and is probably more desirable for a user.

    This **currently** means that, if a tool is named `tool`,
    and we have these two artifacts:

    - Artifact A: `tool-v1.0.0-x86_64-linux`
    - Artifact B: `tool-with-extras-in-name-v1.0.0-x86_64-linux`

    Then A would be preferred, because it mentions
    the tool name more precisely, and nothing else.

    Note that this sorting method is subject to change
    and should not be directly exposed in a public API.
*/
pub(super) fn sort_preferred_artifact(artifact_a: &Artifact, artifact_b: &Artifact) -> Ordering {
    let count_a = count_non_tool_mentions(
        artifact_a.name.as_deref().unwrap_or_default(),
        artifact_a.tool_spec.id(),
    );
    let count_b = count_non_tool_mentions(
        artifact_b.name.as_deref().unwrap_or_default(),
        artifact_b.tool_spec.id(),
    );
    count_a.cmp(&count_b)
}

fn count_non_tool_mentions(name: impl AsRef<str>, tool_id: &ToolId) -> usize {
    let name = name.as_ref();
    if name.trim().is_empty() {
        return 0;
    }

    let name_words = name
        .split(char_is_word_separator)
        .filter(|s| word_is_not_arch_or_os_or_version_or_numeric(s))
        .collect::<Vec<_>>();
    let tool_words = tool_id
        .name()
        .split(char_is_word_separator)
        .collect::<Vec<_>>();

    #[allow(clippy::cast_possible_wrap)]
    let len_diff = ((name_words.len() as isize) - (tool_words.len() as isize)).unsigned_abs();

    let mut word_diff = 0;
    for (name_word, tool_word) in name_words.into_iter().zip(tool_words) {
        if !name_word.eq_ignore_ascii_case(tool_word) {
            word_diff += 1;
        }
    }

    len_diff + word_diff
}

fn word_is_not_arch_or_os_or_version_or_numeric(word: impl AsRef<str>) -> bool {
    let word = word.as_ref();
    Arch::detect(word).is_none()
        && OS::detect(word).is_none()
        && word.trim_start_matches('v').parse::<Version>().is_err()
        && !word.chars().all(char::is_numeric)
}

pub(super) fn sort_preferred_formats(artifact_a: &Artifact, artifact_b: &Artifact) -> Ordering {
    match (artifact_a.format, artifact_b.format) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, _) => std::cmp::Ordering::Greater,
        (_, None) => std::cmp::Ordering::Less,
        (Some(format_a), Some(format_b)) => format_a.cmp(&format_b),
    }
}

#[cfg(test)]
mod tests {
    use crate::sources::{ArtifactFormat, ArtifactProvider};

    use super::*;

    fn new_id(author: &str, name: &str) -> ToolId {
        format!("{author}/{name}").parse().unwrap()
    }

    fn test_no_mentions(name: &str, tool_name: &str) {
        let tool_id = new_id("author", tool_name);
        assert_eq!(
            count_non_tool_mentions(name, &tool_id),
            0,
            "Expected no non-tool mentions in name: {name}"
        );
    }

    fn test_some_mentions(name: &str, tool_name: &str) {
        let tool_id = new_id("author", tool_name);
        assert_ne!(
            count_non_tool_mentions(name, &tool_id),
            0,
            "Expected non-tool mentions in name: {name}"
        );
    }

    #[test]
    fn name_mention_check_tool_valid() {
        // Single word tools
        test_no_mentions("tool", "tool");
        test_no_mentions("tool-linux", "tool");
        test_no_mentions("tool-v1.0.0", "tool");
        test_no_mentions("tool-1.0.0-x86_64-linux", "tool");
        test_no_mentions("tool-x86_64-linux", "tool");
        test_no_mentions("tool-x86_64-linux-v1.0.0", "tool");
        // Multiple word tools
        test_no_mentions("super-tool", "super-tool");
        test_no_mentions("super-tool-linux", "super-tool");
        test_no_mentions("super-tool-v1.0.0", "super-tool");
        test_no_mentions("super-tool-1.0.0-x86_64-linux", "super-tool");
        test_no_mentions("super-mega-tool", "super-mega-tool");
        test_no_mentions("super-mega-tool-linux", "super-mega-tool");
        test_no_mentions("super-mega-tool-v1.0.0", "super-mega-tool");
        test_no_mentions("super-mega-tool-1.0.0-x86_64-linux", "super-mega-tool");
    }

    #[test]
    fn name_mention_check_tool_invalid() {
        // Contains similar but not exact word
        test_some_mentions("tooling", "tool");
        test_some_mentions("tooling-linux", "tool");
        test_some_mentions("tooling-v1.0.0", "tool");
        test_some_mentions("tooling-1.0.0-x86_64-linux", "tool");
        test_some_mentions("tooling-x86_64-linux", "tool");
        test_some_mentions("tooling-x86_64-linux-v1.0.0", "tool");
        // Contains the exact word, but also others
        test_some_mentions("super-tool", "tool");
        test_some_mentions("super-tool-linux", "tool");
        test_some_mentions("super-tool-v1.0.0", "tool");
        test_some_mentions("super-tool-1.0.0-x86_64-linux", "tool");
        test_some_mentions("super-mega-tool", "tool");
        test_some_mentions("super-mega-tool-linux", "tool");
        test_some_mentions("super-mega-tool-v1.0.0", "tool");
        test_some_mentions("super-mega-tool-1.0.0-x86_64-linux", "tool");
    }

    #[test]
    fn name_mention_check_case_insensitive() {
        test_no_mentions("Tool-x86_64-linux", "tool");
        test_no_mentions("tOOl-x86_64-linux", "tool");
        test_no_mentions("TOOL-x86_64-linux", "tool");
        test_some_mentions("Tooling-x86_64-linux", "tool");
        test_some_mentions("tOOling-x86_64-linux", "tool");
        test_some_mentions("TOOLING-x86_64-linux", "tool");
    }

    #[test]
    fn name_mention_check_real_tools() {
        // Valid
        test_no_mentions("wally-v0.3.2-linux.zip", "wally");
        test_no_mentions("lune-0.8.6-macos-aarch64.zip", "lune");
        test_no_mentions("selene-0.27.1-linux.zip", "selene");
        // Invalid
        test_some_mentions("selene-light-0.27.1-linux.zip", "selene");
        // Valid - but multiple words
        test_no_mentions("sentry-cli-linux-i686-2.32.1", "sentry-cli");
        test_no_mentions("selene-light-0.27.1-linux.zip", "selene-light");
    }

    #[test]
    fn test_prefers_known_format() {
        let artifact_names = vec![
            "tool-v1.0.0-x86_64-linux",
            "tool-v1.0.0-x86_64-linux.elf",
            "tool-v1.0.0-x86_64-linux.zip",
            "tool-v1.0.0-x86_64-linux.tar",
            "tool-v1.0.0-x86_64-linux.tar.gz",
            "tool-v1.0.0-x86_64-linux.gz",
        ];

        let mut artifacts = artifact_names
            .into_iter()
            .map(|name| Artifact {
                provider: ArtifactProvider::GitHub,
                format: ArtifactFormat::from_path_or_url(name),
                id: Some("id".to_string()),
                url: Some("https://github.com".parse().unwrap()),
                name: Some(name.to_string()),
                tool_spec: new_id("author", name).into_spec(Version::parse("1.0.0").unwrap()),
            })
            .collect::<Vec<_>>();

        artifacts.sort_by(sort_preferred_formats);

        let artifact_names_sorted = artifacts
            .iter()
            .map(|artifact| artifact.name.as_deref().unwrap())
            .collect::<Vec<_>>();

        assert_eq!(
            artifact_names_sorted,
            vec![
                "tool-v1.0.0-x86_64-linux.tar.gz",
                "tool-v1.0.0-x86_64-linux.tar",
                "tool-v1.0.0-x86_64-linux.zip",
                "tool-v1.0.0-x86_64-linux.gz",
                "tool-v1.0.0-x86_64-linux",
                "tool-v1.0.0-x86_64-linux.elf",
            ]
        );
    }
}
