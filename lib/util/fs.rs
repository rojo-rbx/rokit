use std::{env::consts::EXE_EXTENSION, path::Path, str::FromStr};

use tokio::fs::{metadata, read_to_string, write};
use tracing::{error, warn};

use crate::result::{RokitError, RokitResult};

/**
    Loads the given type from the file at the given path.

    Will return an error if the file does not exist or could not be parsed.
*/
pub(crate) async fn load_from_file<P, T, E>(path: P) -> RokitResult<T>
where
    P: AsRef<Path>,
    T: FromStr<Err = E>,
    E: Into<RokitError>,
{
    let path = path.as_ref();
    match read_to_string(path).await {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Err(RokitError::FileNotFound(path.into()))
        }
        Err(e) => Err(e.into()),
        Ok(s) => match s.parse() {
            Ok(t) => Ok(t),
            Err(e) => Err(e.into()),
        },
    }
}

/**
    Saves the given data, stringified, to the file at the given path.
*/
pub(crate) async fn save_to_file<P, T>(path: P, data: T) -> RokitResult<()>
where
    P: AsRef<Path>,
    T: Clone + ToString,
{
    let path = path.as_ref();
    write(path, data.to_string()).await?;
    Ok(())
}

/**
    Checks if the given path exists.

    Note that this may return `false` if the caller
    does not have permissions to access the given path.
*/
pub async fn path_exists(path: impl AsRef<Path>) -> bool {
    metadata(path).await.is_ok()
}

/**
    Writes the given contents to the file at the
    given path, and adds executable permissions to it.
*/
pub async fn write_executable_file(
    path: impl AsRef<Path>,
    contents: impl AsRef<[u8]>,
) -> RokitResult<()> {
    let path = path.as_ref();

    if !EXE_EXTENSION.is_empty() {
        match path.extension() {
            Some(extension) if extension == EXE_EXTENSION => {}
            _ => warn!(
                "An executable file was written without an executable extension!\
                \nThe file at '{path:?}' may not be usable.\
                \nThis is most likely a bug in Rokit, please report it at {}",
                env!("CARGO_PKG_REPOSITORY").trim_end_matches(".git")
            ),
        }
    }
    if let Err(e) = write(path, contents).await {
        error!("Failed to write executable to {path:?}:\n{e}");
        return Err(e.into());
    }

    add_executable_permissions(path).await?;

    Ok(())
}

#[cfg(unix)]
async fn add_executable_permissions(path: impl AsRef<Path>) -> RokitResult<()> {
    use std::fs::Permissions;
    use std::os::unix::fs::PermissionsExt;
    use tokio::fs::set_permissions;

    let path = path.as_ref();
    if let Err(e) = set_permissions(path, Permissions::from_mode(0o755)).await {
        error!("Failed to set executable permissions on {path:?}:\n{e}");
        return Err(e.into());
    }

    Ok(())
}

#[cfg(not(unix))]
async fn add_executable_permissions(_path: impl AsRef<Path>) -> RokitResult<()> {
    Ok(())
}
