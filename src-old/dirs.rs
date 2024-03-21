use std::env;
use std::path::PathBuf;

pub fn home_dir() -> Option<PathBuf> {
    if let Ok(home) = env::var("AFTMAN_HOME") {
        return Some(PathBuf::from(home));
    }

    dirs::home_dir()
}
