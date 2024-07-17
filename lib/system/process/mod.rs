#![allow(clippy::unused_async)]

use std::io::{stderr, stdout, IsTerminal};

#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod windows;

#[cfg(unix)]
use self::unix as platform;

#[cfg(windows)]
use self::windows as platform;

/**
    Enum representing possible sources that may have launched Rokit.

    Note that this in non-exhaustive, and may be extended in the future.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Launcher {
    WindowsExplorer,
    MacOsFinder,
}

/**
    Enum representing the detected kind of parent process of Rokit.

    Note that this in non-exhaustive, and may be extended in the future.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Parent {
    Launcher(Launcher),
    Terminal,
}

impl Parent {
    /**
        Returns `true` if the parent is a launcher.
    */
    #[must_use]
    pub const fn is_launcher(self) -> bool {
        matches!(self, Self::Launcher(_))
    }

    /**
        Returns `true` if the parent is a terminal.
    */
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Terminal)
    }

    /**
        Tries to detect the parent process of Rokit.

        Returns `None` if the parent process could not be detected.
    */
    pub async fn get() -> Option<Self> {
        platform::try_detect_launcher()
            .await
            .map(Self::Launcher)
            .or_else(|| {
                if stdout().is_terminal() || stderr().is_terminal() {
                    Some(Self::Terminal)
                } else {
                    None
                }
            })
    }
}
