use std::env::consts::OS as CURRENT_OS;

use super::executable_parsing::parse_executable;

#[rustfmt::skip]
const OS_KEYWORDS: [(OS, &[&str]); 3] = [
    (OS::Windows, &["windows", "win32", "win64", "win-x86", "win-x64"]),
    (OS::MacOS,   &["macos", "osx", "darwin", "apple"]),
    (OS::Linux,   &["linux", "ubuntu", "debian"]),
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
    pub fn current_system() -> Self {
        match CURRENT_OS {
            "windows" => Self::Windows,
            "macos" => Self::MacOS,
            "linux" => Self::Linux,
            _ => panic!("Unsupported OS: {CURRENT_OS}"),
        }
    }

    /**
        Detect an operating system by identifying keywords in a search string.
    */
    pub fn detect(search_string: impl AsRef<str>) -> Option<Self> {
        let lowercased = search_string.as_ref().to_lowercase();
        for (os, keywords) in OS_KEYWORDS {
            for keyword in keywords {
                if lowercased.contains(keyword) {
                    return Some(os);
                }
            }
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
}

#[cfg(test)]
mod tests {
    #![allow(clippy::uninlined_format_args)]
    #![allow(clippy::inefficient_to_string)]

    use super::*;

    #[test]
    fn keywords_are_lowercase() {
        for (toolchain, keywords) in OS_KEYWORDS {
            for keyword in keywords {
                assert_eq!(
                    keyword.to_string(),
                    keyword.to_lowercase(),
                    "OS keyword for {:?} is not lowercase: {}",
                    toolchain,
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
    }

    #[test]
    fn detect_os_invalid() {
        assert_eq!(OS::detect("APP-widows-ARCH-VER"), None);
        assert_eq!(OS::detect("APP-mac_in_tosh-ARCH-VER"), None);
        assert_eq!(OS::detect("APP-myOS-ARCH-VER"), None);
        assert_eq!(OS::detect("APP-fedoooruhh-ARCH-VER"), None);
        assert_eq!(OS::detect("APP-linucks-ARCH-VER"), None);
    }
}
