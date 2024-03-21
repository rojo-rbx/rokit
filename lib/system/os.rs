use std::env::consts::OS as CURRENT_OS;

const KEYWORDS_WINDOWS: [&str; 3] = ["windows", "win32", "win64"];
const KEYWORDS_MACOS: [&str; 3] = ["macos", "osx", "darwin"];
const KEYWORDS_LINUX: [&str; 3] = ["linux", "ubuntu", "debian"];

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
    pub fn current() -> Self {
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
        for keyword in &KEYWORDS_WINDOWS {
            if lowercased.contains(keyword) {
                return Some(Self::Windows);
            }
        }
        for keyword in &KEYWORDS_MACOS {
            if lowercased.contains(keyword) {
                return Some(Self::MacOS);
            }
        }
        for keyword in &KEYWORDS_LINUX {
            if lowercased.contains(keyword) {
                return Some(Self::Linux);
            }
        }
        None
    }
}
