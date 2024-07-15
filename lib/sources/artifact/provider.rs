use std::{fmt, str::FromStr};

/**
    An artifact provider supported by Rokit.

    The default provider is [`ArtifactProvider::GitHub`].
*/
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArtifactProvider {
    #[default]
    GitHub,
}

impl ArtifactProvider {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GitHub => "github",
        }
    }

    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::GitHub => "GitHub",
        }
    }
}

impl FromStr for ArtifactProvider {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let l = s.trim().to_lowercase();
        match l.as_str() {
            "github" => Ok(Self::GitHub),
            _ => Err(format!("unknown artifact provider '{l}'")),
        }
    }
}

impl fmt::Display for ArtifactProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}
