#[rustfmt::skip]
const TOOLCHAIN_KEYWORDS: [(Toolchain, &[&str]); 3] = [
    (Toolchain::Msvc, &["msvc"]),
    (Toolchain::Gnu,  &["gnu"]),
    (Toolchain::Musl, &["musl"]),
];

/**
    Enum representing a system toolchain, such as MSVC or GNU.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Toolchain {
    Msvc,
    Gnu,
    Musl,
}

impl Toolchain {
    /**
        Get the toolchain of the current host system.
    */
    pub fn current() -> Option<Self> {
        None // TODO: Implement detection of the host toolchain
    }

    /**
        Detect a toolchain by identifying keywords in a search string.
    */
    pub fn detect(search_string: impl AsRef<str>) -> Option<Self> {
        let lowercased = search_string.as_ref().to_lowercase();
        for (toolchain, keywords) in TOOLCHAIN_KEYWORDS {
            for keyword in keywords {
                if lowercased.contains(keyword) {
                    return Some(toolchain);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keywords_are_lowercase() {
        for (toolchain, keywords) in TOOLCHAIN_KEYWORDS {
            for keyword in keywords {
                assert_eq!(
                    keyword.to_string(),
                    keyword.to_lowercase(),
                    "Toolchain keyword for {:?} is not lowercase: {}",
                    toolchain,
                    keyword
                );
            }
        }
    }

    #[test]
    fn detect_toolchain_valid() {
        assert_eq!(Toolchain::detect("msvc"), Some(Toolchain::Msvc));
        assert_eq!(Toolchain::detect("msvc-clang"), Some(Toolchain::Msvc));
        assert_eq!(Toolchain::detect("gnu"), Some(Toolchain::Gnu));
        assert_eq!(Toolchain::detect("musl"), Some(Toolchain::Musl));
        assert_eq!(Toolchain::detect("musl-gcc"), Some(Toolchain::Musl));
    }

    #[test]
    fn detect_toolchain_invalid() {
        assert_eq!(Toolchain::detect("unknown"), None);
        assert_eq!(Toolchain::detect("msrv"), None);
        assert_eq!(Toolchain::detect("gnnuuu!"), None);
        assert_eq!(Toolchain::detect("muscle"), None);
    }
}
