use std::env;

use crate::home::Home;

mod unix;

#[cfg(windows)]
mod windows;

#[allow(unreachable_code)]
pub fn init(home: &Home) -> anyhow::Result<()> {
    // Users can define this environment variable to force Aftman to interact
    // with the user's PATH like a Unix machine. This is helpful for running
    // tests on Windows.
    if cfg!(unix) || env::var("AFTMAN_PATH_UNIX").is_ok() {
        return unix::init(home);
    }

    #[cfg(windows)]
    {
        return windows::init(home);
    }

    Ok(())
}

#[allow(unreachable_code)]
pub fn add(home: &Home) -> anyhow::Result<bool> {
    if cfg!(unix) || env::var("AFTMAN_PATH_UNIX").is_ok() {
        return unix::add(home);
    }

    #[cfg(windows)]
    {
        return windows::add(home);
    }

    tracing::debug!("Not adding value to path because this platform is not supported.");
    Ok(false)
}
