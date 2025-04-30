use std::env::var;

#[derive(Debug, Clone, Copy)]
pub enum Shell {
    Posix,
    Bash,
    Zsh,
}

impl Shell {
    pub const ALL: [Self; 3] = [Self::Posix, Self::Bash, Self::Zsh];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Posix => "sh",
            Self::Bash => "bash",
            Self::Zsh => "zsh",
        }
    }

    pub const fn env_file_path(self) -> &'static str {
        match self {
            Self::Posix => ".profile",
            Self::Bash => ".bashrc",
            Self::Zsh => ".zshenv",
        }
    }

    pub fn env_file_should_create_if_nonexistent(self) -> bool {
        // Create a new shell env file for the user if we are
        // confident that this is the shell that they are using
        var("SHELL").is_ok_and(|current_shell| {
            // Detect /bin/sh, /bin/bash, /bin/zsh, etc
            current_shell.ends_with(&format!("/{}", self.name()))
        })
    }
}
