use std::cmp::Ordering;

use super::{Arch, Toolchain, OS};

/**
    Information describing a system, such as its operating
    system, architecture, and preferred toolchain.

    May represent the current system or a target system, and is typically
    used to check for compatibility between two or more specified systems.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Description {
    os: OS,
    arch: Arch,
    toolchain: Option<Toolchain>,
}

impl Description {
    /**
        Get the description for the current host system.
    */
    pub fn current() -> Self {
        Self {
            os: OS::current(),
            arch: Arch::current(),
            toolchain: Toolchain::current(),
        }
    }

    /**
        Detect system description by identifying keywords in a search string.

        Returns `None` if operating system or architecture could not be detected.
    */
    pub fn detect(search_string: impl AsRef<str>) -> Option<Self> {
        let search_string = search_string.as_ref();

        let os = OS::detect(search_string)?;
        let arch = Arch::detect(search_string)?;
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
    pub fn is_compatible_with(&self, other: &Description) -> bool {
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
    pub fn sort_by_preferred_compat<'a>(self, a: &'a Self, b: &'a Self) -> Ordering {
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
