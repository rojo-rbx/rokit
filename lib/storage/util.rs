use std::{convert::Infallible, path::Path, str::FromStr};

use tokio::fs::{read_to_string, write};

use super::StorageResult;

/**
    Loads the given type from the file at the given path.

    If the file does not exist, it will be created with
    the default stringified contents of the type.
*/
pub(super) async fn load_from_file<P, T>(path: P) -> StorageResult<T>
where
    P: AsRef<Path>,
    T: Default + FromStr<Err = Infallible> + ToString,
{
    let path = path.as_ref();
    match read_to_string(path).await {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let new: T = Default::default();
            write(path, new.to_string()).await?;
            Ok(new)
        }
        Err(e) => Err(e.into()),
        Ok(s) => Ok(s.parse().unwrap()),
    }
}

/**
    Saves the given data, stringified, to the file at the given path.
*/
pub(super) async fn save_to_file<P, T>(path: P, data: T) -> StorageResult<()>
where
    P: AsRef<Path>,
    T: Clone + ToString,
{
    let path = path.as_ref();
    write(path, data.to_string()).await?;
    Ok(())
}
