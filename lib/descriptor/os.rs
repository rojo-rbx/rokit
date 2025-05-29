use std::env::consts::OS as CURRENT_OS;

use crate::util::str::char_is_word_separator;

use super::executable_parsing::parse_executable;

// Matching substrings - these can be partial matches, eg. "wordwin64" will match as windows OS
// These will take priority over full word matches, and should be as precise as possible
#[rustfmt::skip]
const OS_SUBSTRINGS: [(OS, &[&str]); 3] = [
    (OS::Windows, &["windows"]),
    (OS::MacOS,   &["macos", "darwin", "apple"]),
    (OS::Linux,   &["linux", "ubuntu", "debian", "fedora"]),
];

// Matching words - these must be full word matches, eg. "tarmac" will not match as mac OS
// Note that these can not contain word separators like "-" or "_", since they're stripped
#[rustfmt::skip]
const OS_FULL_WORDS: [(OS, &[&str]); 3] = [
    (OS::Windows, &["win", "win32", "win64"]),
    (OS::MacOS,   &["mac", "osx"]),
    (OS::Linux,   &[]),
];

/**
    Enum representing a system operating system, such as Windows or Linux.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum OS {
    Windows,
    MacOS, // aka OS X
    Linux,
}

impl OS {
    /**
        Get the operating system of the current host system.
    */
    #[must_use]
    pub const fn current_system() -> Self {
        match CURRENT_OS.as_bytes() {
            b"windows" => Self::Windows,
            b"macos" => Self::MacOS,
            b"linux" => Self::Linux,
            _ => panic!("Unsupported OS"),
        }
    }

    /**
        Detect an operating system by identifying keywords in a search string.
    */
    pub fn detect(search_string: impl AsRef<str>) -> Option<Self> {
        let lowercased = search_string.as_ref().to_lowercase();

        // Try to find a substring match first, these are generally longer and
        // contain more symbol-like characters, less likely to be a false positive
        for (os, keywords) in OS_SUBSTRINGS {
            for keyword in keywords {
                if lowercased.contains(keyword) {
                    return Some(os);
                }
            }
        }

        // Try to find a strict keyword given as a standalone word in our search string
        if let Some(os) = lowercased.split(char_is_word_separator).find_map(|part| {
            OS_FULL_WORDS.iter().find_map(|(os, keywords)| {
                if keywords.contains(&part) {
                    Some(*os)
                } else {
                    None
                }
            })
        }) {
            return Some(os);
        }

        None
    }

    /**
        Detect an operating system from the binary contents of an executable file.

        Parsing binaries is a potentially expensive operation, so this method should
        preferrably only be used as a fallback or for more descriptive error messages.
    */
    pub fn detect_from_executable(binary_contents: impl AsRef<[u8]>) -> Option<Self> {
        Some(parse_executable(binary_contents)?.0)
    }

    /**
        Get the name of the operating system as a string.
    */
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::MacOS => "macos",
            Self::Linux => "linux",
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::uninlined_format_args)]
    #![allow(clippy::inefficient_to_string)]

    use super::*;

    #[test]
    fn substrings_and_words_are_lowercase() {
        for (os, keywords) in OS_SUBSTRINGS.into_iter().chain(OS_FULL_WORDS.into_iter()) {
            for keyword in keywords {
                assert_eq!(
                    keyword.to_string(),
                    keyword.to_lowercase(),
                    "OS substring / word for {:?} is not lowercase: {}",
                    os,
                    keyword
                );
            }
        }
    }

    #[test]
    fn words_do_not_contain_word_separators() {
        for (os, keywords) in OS_FULL_WORDS {
            for keyword in keywords {
                assert!(
                    !keyword.contains(char_is_word_separator),
                    "OS keyword for {:?} contains word separator: {}",
                    os,
                    keyword
                );
            }
        }
    }

    #[test]
    fn current_os() {
        let os = OS::current_system();
        if cfg!(target_os = "windows") {
            assert_eq!(os, OS::Windows);
        } else if cfg!(target_os = "macos") {
            assert_eq!(os, OS::MacOS);
        } else if cfg!(target_os = "linux") {
            assert_eq!(os, OS::Linux);
        } else {
            panic!("Unknown OS for testing: {CURRENT_OS}");
        }
    }

    #[test]
    fn detect_os_valid() {
        assert_eq!(OS::detect("APP-windows-ARCH-VER"), Some(OS::Windows));
        assert_eq!(OS::detect("APP-win32-ARCH-VER"), Some(OS::Windows));
        assert_eq!(OS::detect("APP-win64-ARCH-VER"), Some(OS::Windows));
        assert_eq!(OS::detect("APP-macos-ARCH-VER"), Some(OS::MacOS));
        assert_eq!(OS::detect("APP-osx-ARCH-VER"), Some(OS::MacOS));
        assert_eq!(OS::detect("APP-darwin-ARCH-VER"), Some(OS::MacOS));
        assert_eq!(OS::detect("APP-apple-ARCH-VER"), Some(OS::MacOS));
        assert_eq!(OS::detect("APP-linux-ARCH-VER"), Some(OS::Linux));
        assert_eq!(OS::detect("APP-ubuntu-ARCH-VER"), Some(OS::Linux));
        assert_eq!(OS::detect("APP-debian-ARCH-VER"), Some(OS::Linux));
        assert_eq!(OS::detect("APP-fedora-ARCH-VER"), Some(OS::Linux));
    }

    #[test]
    fn detect_os_invalid() {
        assert_eq!(OS::detect("APP-widows-ARCH-VER"), None);
        assert_eq!(OS::detect("APP-macc_in_tosh-ARCH-VER"), None);
        assert_eq!(OS::detect("APP-myOS-ARCH-VER"), None);
        assert_eq!(OS::detect("APP-fedoooruhh-ARCH-VER"), None);
        assert_eq!(OS::detect("APP-linucks-ARCH-VER"), None);
    }

    #[test]
    fn real_tool_specs() {
        const REAL_TOOLS: [(&str, Option<OS>); 10] = [
            ("stylua-linux-x86_64-musl", Some(OS::Linux)),
            ("remodel-0.11.0-linux-x86_64", Some(OS::Linux)),
            ("rojo-0.6.0-alpha.1-win64", Some(OS::Windows)),
            ("lune-0.6.7-windows-aarch64", Some(OS::Windows)),
            ("darklua-linux-aarch64", Some(OS::Linux)),
            ("tarmac-0.7.5-macos", Some(OS::MacOS)),
            ("sentry-cli-Darwin-universal", Some(OS::MacOS)),
            ("sentry-cli-linux-i686-2.32.1", Some(OS::Linux)),
            (
                "just-1.28.0-armv7-unknown-linux-musleabihf",
                Some(OS::Linux),
            ),
            ("just-1.28.0-arm-unknown-linux-musleabihf", Some(OS::Linux)),
        ];
        for (tool, expected) in REAL_TOOLS {
            assert_eq!(OS::detect(tool), expected, "Tool: {tool}");
        }
    }
}
