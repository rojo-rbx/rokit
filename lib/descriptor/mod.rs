use std::{cmp::Ordering, str::FromStr};

use thiserror::Error;

mod arch;
mod os;
mod toolchain;

pub use self::arch::Arch;
pub use self::os::OS;
pub use self::toolchain::Toolchain;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum DescriptionParseError {
    #[error("unknown OS, or no OS detected")]
    OS,
}

/**
    Information describing a system, such as its operating
    system, architecture, and preferred toolchain.

    May represent the current system or a target system, and is typically
    used to check for compatibility between two or more specified systems.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Descripor {
    os: OS,
    arch: Arch,
    toolchain: Option<Toolchain>,
}

impl Descripor {
    /**
        Get the description for the current host system.
    */
    pub fn current_system() -> Self {
        Self {
            os: OS::current_system(),
            arch: Arch::current_system(),
            toolchain: Toolchain::current_system(),
        }
    }

    /**
        Detect system description by identifying keywords in a search string.

        Returns `None` if operating system or architecture could not be detected.
    */
    pub fn detect(search_string: impl AsRef<str>) -> Option<Self> {
        let search_string = search_string.as_ref();

        let os = OS::detect(search_string)?;
        let arch = Arch::detect(search_string).unwrap_or_default();
        let toolchain = Toolchain::detect(search_string);

        Some(Self {
            os,
            arch,
            toolchain,
        })
    }

    /**
        Check if this description is compatible with another description.

        Two descriptions are compatible if they have the same operating
        system and architecture, except for two special cases:

        - Windows and Linux 64-bit can run 32-bit executables
        - macOS Apple Silicon can run x64 (Intel) executables
    */
    pub fn is_compatible_with(&self, other: &Descripor) -> bool {
        // Operating system must _always_ match
        (self.os == other.os)
            && (
                // Accept general cases for when architectures match ...
                self.arch == other.arch
                // ... or special cases for architecture compatibility
                || matches!(
                    (self.os, self.arch, other.arch),
                    (OS::Windows, Arch::X64, Arch::X86)
                    | (OS::Linux, Arch::X64, Arch::X86)
                    | (OS::MacOS, Arch::Arm64, Arch::X64)
                )
            )
    }

    /**
        Sort two descriptions by their preferred order, compared to this description.

        The two descriptions will be sorted by their _how_ compatible they
        are, meaning native binaries / descriptions will be preferred over
        emulatable ones, and preferred architectures will also come first.

        Two descriptions that are not compatible _at all_ have no defined order.
    */
    pub fn sort_by_preferred_compat(self, a: &Self, b: &Self) -> Ordering {
        // Check for strict compatibility first (exact matches)
        let a_compat = a.os == self.os && a.arch == self.arch;
        let b_compat = b.os == self.os && b.arch == self.arch;
        if a_compat && !b_compat {
            return Ordering::Less;
        }
        if !a_compat && b_compat {
            return Ordering::Greater;
        }

        // Sort by preferred architecture or toolchain
        if a.arch != b.arch {
            return a.arch.cmp(&b.arch);
        }
        if a.toolchain != b.toolchain {
            return a.toolchain.cmp(&b.toolchain);
        }

        // Fallback for when no order is defined
        a.os.cmp(&b.os)
    }
}

impl FromStr for Descripor {
    type Err = DescriptionParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let os = OS::detect(s).ok_or(DescriptionParseError::OS)?;
        let arch = Arch::detect(s).unwrap_or_default();
        let toolchain = Toolchain::detect(s);

        Ok(Self {
            os,
            arch,
            toolchain,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_desc(description: &str, expected: Descripor) {
        assert_eq!(
            Descripor::detect(description),
            Some(expected),
            "{description}"
        );
    }

    #[test]
    fn current_description() {
        let current = Descripor::current_system();
        assert_eq!(current.os, OS::current_system());
        assert_eq!(current.arch, Arch::current_system());
        assert_eq!(current.toolchain, Toolchain::current_system());
    }

    #[test]
    fn detect_description_valid() {
        // Windows
        check_desc(
            "windows-x64-msvc",
            Descripor {
                os: OS::Windows,
                arch: Arch::X64,
                toolchain: Some(Toolchain::Msvc),
            },
        );
        check_desc(
            "win64",
            Descripor {
                os: OS::Windows,
                arch: Arch::X64,
                toolchain: None,
            },
        );
        check_desc(
            "windows-x86-gnu",
            Descripor {
                os: OS::Windows,
                arch: Arch::X86,
                toolchain: Some(Toolchain::Gnu),
            },
        );
        check_desc(
            "windows-x86",
            Descripor {
                os: OS::Windows,
                arch: Arch::X86,
                toolchain: None,
            },
        );
        check_desc(
            "win32",
            Descripor {
                os: OS::Windows,
                arch: Arch::X86,
                toolchain: None,
            },
        );
        // macOS
        check_desc(
            "aarch64-macos",
            Descripor {
                os: OS::MacOS,
                arch: Arch::Arm64,
                toolchain: None,
            },
        );
        check_desc(
            "macos-x64-gnu",
            Descripor {
                os: OS::MacOS,
                arch: Arch::X64,
                toolchain: Some(Toolchain::Gnu),
            },
        );
        check_desc(
            "macos-x64",
            Descripor {
                os: OS::MacOS,
                arch: Arch::X64,
                toolchain: None,
            },
        );
        // Linux
        check_desc(
            "linux-x86_64-gnu",
            Descripor {
                os: OS::Linux,
                arch: Arch::X64,
                toolchain: Some(Toolchain::Gnu),
            },
        );
        check_desc(
            "linux-gnu-x86",
            Descripor {
                os: OS::Linux,
                arch: Arch::X86,
                toolchain: Some(Toolchain::Gnu),
            },
        );
        check_desc(
            "armv7-linux-musl",
            Descripor {
                os: OS::Linux,
                arch: Arch::Arm32,
                toolchain: Some(Toolchain::Musl),
            },
        );
    }

    #[test]
    fn detect_description_universal() {
        // macOS universal binaries should parse as x64 (most compatible)
        check_desc(
            "macos-universal",
            Descripor {
                os: OS::MacOS,
                arch: Arch::X64,
                toolchain: None,
            },
        );
        check_desc(
            "darwin-universal",
            Descripor {
                os: OS::MacOS,
                arch: Arch::X64,
                toolchain: None,
            },
        );
    }

    #[test]
    fn detect_description_invalid() {
        // Anything that is missing a valid os or arch should be invalid
        const INVALID_DESCRIPTIONS: &[&str] = &[
            "widows-x64-unknown",
            "macccos-x64-unknown",
            "linucks-x64-unknown",
            "unknown-x64-gnu",
            "unknown-x64",
            "unknown-x86-gnu",
            "unknown-x86",
            "unknown-armv7-musl",
        ];
        for description in INVALID_DESCRIPTIONS {
            assert_eq!(Descripor::detect(description), None);
        }
    }

    #[test]
    fn parse_from_str_valid() {
        const VALID_STRINGS: &[&str] = &[
            "windows-x64-msvc",
            "win64",
            "windows-x86-gnu",
            "windows-x86",
            "win32",
            "aarch64-macos",
            "macos-x64-gnu",
            "macos-x64",
            "linux-x86_64-gnu",
            "linux-gnu-x86",
            "armv7-linux-musl",
        ];
        for description in VALID_STRINGS {
            assert!(description.parse::<Descripor>().is_ok());
        }
    }

    #[test]
    fn parse_from_str_invalid_os() {
        const INVALID_OS_STRINGS: &[&str] = &[
            "widows-x64-msvc",
            "macccos-x64-gnu",
            "linucks-x86-gnu",
            "unknown-x64-gnu",
        ];
        for description in INVALID_OS_STRINGS {
            assert_eq!(
                description.parse::<Descripor>(),
                Err(DescriptionParseError::OS)
            );
        }
    }
}
