use std::env::consts::ARCH as CURRENT_ARCH;

use super::OS;

const KEYWORDS_X64: [&str; 4] = ["x86-64", "x86_64", "x64", "amd64"];
const KEYWORDS_X86: [&str; 4] = ["x86", "i686", "win32", "i386"];
const KEYWORDS_ARM64: [&str; 3] = ["aarch64", "arm64", "armv9"];
const KEYWORDS_ARM32: [&str; 3] = ["arm", "arm32", "armv7"];

/**
    Enum representing a system architecture, such as x86-64 or ARM.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Arch {
    // NOTE: The ordering here is important! Putting arm architectures before
    // x86 architectures prioritizes native binaries on ARM systems over x86
    // binaries, which would most likely get emulated (eg. Rosetta on macOS)
    Arm64, // aka AArch64
    X64,   // aka x86-64, AMD64
    Arm32, // aka ARMv7
    X86,   // aka i686
}

impl Arch {
    /**
        Get the architecture of the current host system.
    */
    pub fn current() -> Self {
        match CURRENT_ARCH {
            "aarch64" => Self::Arm64,
            "x86_64" => Self::X64,
            "x86" => Self::X86,
            "arm" => Self::Arm32,
            _ => panic!("Unsupported architecture: {CURRENT_ARCH}"),
        }
    }

    /**
        Detect an architecture by identifying keywords in a search string.
    */
    pub fn detect(search_string: impl AsRef<str>) -> Option<Self> {
        let lowercased = search_string.as_ref().to_lowercase();

        for keyword in &KEYWORDS_X64 {
            if lowercased.contains(keyword) {
                return Some(Self::X64);
            }
        }
        for keyword in &KEYWORDS_X86 {
            if lowercased.contains(keyword) {
                return Some(Self::X86);
            }
        }
        for keyword in &KEYWORDS_ARM64 {
            if lowercased.contains(keyword) {
                return Some(Self::Arm64);
            }
        }
        for keyword in &KEYWORDS_ARM32 {
            if lowercased.contains(keyword) {
                return Some(Self::Arm32);
            }
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
}
