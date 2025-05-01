use std::env::consts::ARCH as CURRENT_ARCH;

use crate::util::str::char_is_word_separator;

use super::{executable_parsing::parse_executable, OS};

// Matching substrings - these can be partial matches, eg. "wordwin64" will match as x64 arch
// These will take priority over full word matches, and should be as precise as possible
#[rustfmt::skip]
const ARCH_SUBSTRINGS: [(Arch, &[&str]); 4] = [
    (Arch::Arm64, &["aarch64", "arm64", "armv9"]),
    (Arch::X64,   &["x86-64", "x86_64", "amd64", "win64", "win-x64"]),
    (Arch::Arm32, &["arm32", "armv7"]),
    (Arch::X86,   &["i686", "i386", "win32", "win-x86"]),
];

// Matching words - these must be full word matches, eg. "tarmac" will not match as arm arch
// Note that these can not contain word separators like "-" or "_", since they're stripped
#[rustfmt::skip]
const ARCH_FULL_WORDS: [(Arch, &[&str]); 4] = [
    (Arch::Arm64, &[]),
    (Arch::X64,   &["x64", "win"]),
    (Arch::Arm32, &["arm"]),
    (Arch::X86,   &["x86"]),
];

/**
    Enum representing a system architecture, such as x86-64 or ARM.
*/
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Arch {
    // NOTE: The ordering here is important! Putting arm architectures before
    // x86 architectures prioritizes native binaries on ARM systems over x86
    // binaries, which would most likely get emulated (eg. Rosetta on macOS)
    Arm64,
    // NOTE: We use X64 as our default architecture, since it's the most common
    // and tools that don't specify an architecture are most likely using x86-64.
    #[default]
    X64,
    Arm32,
    X86,
}

impl Arch {
    /**
        Get the architecture of the current host system.
    */
    #[must_use]
    pub const fn current_system() -> Self {
        match CURRENT_ARCH.as_bytes() {
            b"aarch64" => Self::Arm64,
            b"x86_64" => Self::X64,
            b"x86" => Self::X86,
            b"arm" => Self::Arm32,
            _ => panic!("Unsupported architecture"),
        }
    }

    /**
        Detect an architecture by identifying keywords in a search string.
    */
    pub fn detect(search_string: impl AsRef<str>) -> Option<Self> {
        let lowercased = search_string.as_ref().to_lowercase();

        // Try to find a substring match first, these are generally longer and
        // contain more symbol-like characters, less likely to be a false positive
        for (arch, keywords) in ARCH_SUBSTRINGS {
            for keyword in keywords {
                if lowercased.contains(keyword) {
                    return Some(arch);
                }
            }
        }

        // Try to find a strict keyword given as a standalone word in our search string
        if let Some(arch) = lowercased.split(char_is_word_separator).find_map(|part| {
            ARCH_FULL_WORDS.iter().find_map(|(arch, keywords)| {
                if keywords.contains(&part) {
                    Some(*arch)
                } else {
                    None
                }
            })
        }) {
            return Some(arch);
        }

        /*
            HACK: If nothing else matched, but the search string contains "universal",
            we may have found a macOS universal binary, which is compatible with both
            x64 and arm64 architectures. In this case, we'll say we found an x64 binary,
            since that will pass compatibility checks with both x64 and aarch64 systems.

            Native binaries for arm64 systems should still be prioritized over x64 binaries
            due to the ordering of the Arch enum variants and the implementation note above.
            Older macOS systems may accidentally pick universal binaries over native x64,
            but this should be a rare edge case and only affect binary size, not performance.
        */
        if lowercased.contains("universal") && matches!(OS::detect(lowercased), Some(OS::MacOS)) {
            return Some(Self::X64);
        }

        None
    }

    /**
        Detect an architecture from the binary contents of an executable file.

        Parsing binaries is a potentially expensive operation, so this method should
        preferrably only be used as a fallback or for more descriptive error messages.
    */
    pub fn detect_from_executable(binary_contents: impl AsRef<[u8]>) -> Option<Self> {
        Some(parse_executable(binary_contents)?.1)
    }

    /**
        Get the architecture as a string, such as "x64" or "arm64".
    */
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Arm64 => "arm64",
            Self::X64 => "x64",
            Self::Arm32 => "arm32",
            Self::X86 => "x86",
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
        for (arch, keywords) in ARCH_SUBSTRINGS
            .into_iter()
            .chain(ARCH_FULL_WORDS.into_iter())
        {
            for keyword in keywords {
                assert_eq!(
                    keyword.to_string(),
                    keyword.to_lowercase(),
                    "Arch substring / word for {:?} is not lowercase: {}",
                    arch,
                    keyword
                );
            }
        }
    }

    #[test]
    fn words_do_not_contain_word_separators() {
        for (toolchain, keywords) in ARCH_FULL_WORDS {
            for keyword in keywords {
                assert!(
                    !keyword.contains(char_is_word_separator),
                    "Arch keyword for {:?} contains word separator: {}",
                    toolchain,
                    keyword
                );
            }
        }
    }

    #[test]
    fn current_arch() {
        let arch = Arch::current_system();
        if cfg!(target_arch = "aarch64") {
            assert_eq!(arch, Arch::Arm64);
        } else if cfg!(target_arch = "x86_64") {
            assert_eq!(arch, Arch::X64);
        } else if cfg!(target_arch = "x86") {
            assert_eq!(arch, Arch::X86);
        } else if cfg!(target_arch = "arm") {
            assert_eq!(arch, Arch::Arm32);
        } else {
            panic!("Unknown architecture for testing: {CURRENT_ARCH}");
        }
    }

    #[test]
    fn detect_arch_valid() {
        const REAL_ARCHITECTURES: [(&str, Arch); 8] = [
            ("APP-x86-64-VER", Arch::X64),
            ("APP-x86_64-VER", Arch::X64),
            ("APP-x64-VER", Arch::X64),
            ("APP-amd64-VER", Arch::X64),
            ("APP-x86-VER", Arch::X86),
            ("APP-i686-VER", Arch::X86),
            ("APP-arm64-VER", Arch::Arm64),
            ("APP-arm-VER", Arch::Arm32),
        ];
        for (real_arch, expected) in REAL_ARCHITECTURES {
            assert_eq!(Arch::detect(real_arch), Some(expected));
        }
    }

    #[test]
    fn detect_arch_invalid() {
        const FAKE_ARCHITECTURES: [&str; 5] = [
            "APP-x84-48-VER",
            "APP-x87-65-VER",
            "APP-x62-VER",
            "APP-nvidia4-VER",
            "APP-intel999-VER",
        ];
        for fake_arch in FAKE_ARCHITECTURES {
            assert_eq!(Arch::detect(fake_arch), None);
        }
    }

    #[test]
    fn detect_arch_universal() {
        assert_eq!(Arch::detect("APP-macos-universal-VER"), Some(Arch::X64));
    }

    #[test]
    fn real_tool_specs() {
        const REAL_TOOLS: [(&str, Option<Arch>); 10] = [
            ("stylua-linux-x86_64-musl", Some(Arch::X64)),
            ("remodel-0.11.0-linux-x86_64", Some(Arch::X64)),
            ("rojo-0.6.0-alpha.1-win64", Some(Arch::X64)),
            ("lune-0.6.7-windows-aarch64", Some(Arch::Arm64)),
            ("darklua-linux-aarch64", Some(Arch::Arm64)),
            ("tarmac-0.7.5-macos", None),
            ("sentry-cli-Darwin-universal", Some(Arch::X64)),
            ("sentry-cli-linux-i686-2.32.1", Some(Arch::X86)),
            (
                "just-1.28.0-armv7-unknown-linux-musleabihf",
                Some(Arch::Arm32),
            ),
            (
                "just-1.28.0-arm-unknown-linux-musleabihf",
                Some(Arch::Arm32),
            ),
        ];
        for (tool, expected) in REAL_TOOLS {
            assert_eq!(Arch::detect(tool), expected, "Tool: {tool}");
        }
    }
}
