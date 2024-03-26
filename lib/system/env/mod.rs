use crate::{result::RokitResult, storage::Home};

mod shell;

#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod windows;

/**
    Tries to add the Rokit binaries directory to the system PATH.

    Returns `true` if the directory was added to the PATH, `false` otherwise.
*/
pub async fn add_to_path(home: &Home) -> RokitResult<bool> {
    #[cfg(unix)]
    {
        self::unix::add_to_path(home).await
    }
    #[cfg(windows)]
    {
        self::windows::add_to_path(home).await
    }
}
