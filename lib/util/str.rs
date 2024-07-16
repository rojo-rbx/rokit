use std::fmt;

/**
    Checks if the given character is a "word" separator.

    For internal use only.
*/
pub(crate) const fn char_is_word_separator(c: char) -> bool {
    c.is_ascii_whitespace() || matches!(c, '-' | '_')
}

/**
    A case-insensitive string wrapper.

    The wrapped string is case-insensitive, but is stored in both cased
    and uncased forms. This allows the original casing to be preserved
    when the string is displayed.

    Comparisons using [`Eq`], [`PartialEq`], [`Ord`], [`PartialOrd`],
    and [`std::hash::Hash`] will always be case-insensitive.
*/
#[derive(Debug, Clone)]
pub struct CaseInsensitiveString {
    uncased: String,
    original: Option<String>,
}

impl CaseInsensitiveString {
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        let s: String = s.into();
        let u = s.to_ascii_lowercase();
        if u == s {
            Self {
                uncased: u,
                original: None,
            }
        } else {
            Self {
                uncased: u,
                original: Some(s),
            }
        }
    }

    #[must_use]
    pub fn uncased_str(&self) -> &str {
        &self.uncased
    }

    #[must_use]
    pub fn original_str(&self) -> &str {
        self.original.as_deref().unwrap_or(&self.uncased)
    }
}

impl Eq for CaseInsensitiveString {}

impl PartialEq for CaseInsensitiveString {
    fn eq(&self, other: &Self) -> bool {
        self.uncased_str() == other.uncased_str()
    }
}

impl std::hash::Hash for CaseInsensitiveString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uncased_str().hash(state);
    }
}

impl Ord for CaseInsensitiveString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uncased_str().cmp(other.uncased_str())
    }
}

impl PartialOrd for CaseInsensitiveString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for CaseInsensitiveString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.original_str().fmt(f)
    }
}
