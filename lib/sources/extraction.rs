use std::{
    env::consts::EXE_SUFFIX,
    io::{self, Read},
    path::PathBuf,
};

use tar::Archive as TarArchive;
use tokio::{task::spawn_blocking, time::Instant};
use zip::ZipArchive;

use crate::result::RokitResult;

pub async fn extract_zip_file(
    zip_contents: impl AsRef<[u8]>,
    desired_file_name: impl Into<String>,
) -> RokitResult<Option<Vec<u8>>> {
    let desired_file_name = format!("{}{EXE_SUFFIX}", desired_file_name.into());
    let desired_file_path = PathBuf::from(&desired_file_name);

    let zip_contents = zip_contents.as_ref().to_vec();
    let num_bytes = zip_contents.len();
    let start = Instant::now();

    // Reading a zip file is a potentially expensive operation, so
    // spawn it as a blocking task and use the tokio thread pool.
    spawn_blocking(move || {
        let mut found = None;
        let mut reader = io::Cursor::new(&zip_contents);
        let mut zip = ZipArchive::new(&mut reader)?;

        // If there is a file with an
        // exact name match, return that ...
        for i in 0..zip.len() {
            let mut file = zip.by_index(i)?;
            let path = match file.enclosed_name() {
                Some(path) => path,
                None => continue,
            };
            if path == desired_file_path {
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes)?;
                found = Some(bytes);
                break;
            }
        }

        // ...otherwise, look for any file with the
        // system's EXE_SUFFIX and return that.
        if found.is_none() && !EXE_SUFFIX.is_empty() {
            for i in 0..zip.len() {
                let mut file = zip.by_index(i)?;
                let path = match file.enclosed_name() {
                    Some(path) => path,
                    None => continue,
                };
                if path.extension().map_or(false, |ext| ext == EXE_SUFFIX) {
                    let mut bytes = Vec::new();
                    file.read_to_end(&mut bytes)?;
                    found = Some(bytes);
                    break;
                }
            }
        }

        tracing::debug!(
            num_bytes,
            elapsed = ?start.elapsed(),
            found = found.is_some(),
            "extracted zip file"
        );
        Ok(found)
    })
    .await?
}

pub async fn extract_tar_file(
    tar_contents: impl AsRef<[u8]>,
    desired_file_name: impl Into<String>,
) -> RokitResult<Option<Vec<u8>>> {
    let desired_file_name = format!("{}{EXE_SUFFIX}", desired_file_name.into());
    let desired_file_path = PathBuf::from(&desired_file_name);

    let tar_contents = tar_contents.as_ref().to_vec();
    let num_bytes = tar_contents.len();
    let start = Instant::now();

    // Reading a tar file is a potentially expensive operation, so
    // spawn it as a blocking task and use the tokio thread pool.
    spawn_blocking(move || {
        let mut found = None;
        let mut reader = io::Cursor::new(&tar_contents);
        let mut tar = TarArchive::new(&mut reader);

        // If there is a file with an
        // exact name match, return that ...
        for entry in tar.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;
            if path == desired_file_path {
                let mut bytes = Vec::new();
                entry.read_to_end(&mut bytes)?;
                found = Some(bytes);
                break;
            }
        }

        // ...otherwise, look for any file with the
        // system's EXE_SUFFIX and return that.
        if found.is_none() && !EXE_SUFFIX.is_empty() {
            for entry in tar.entries()? {
                let mut entry = entry?;
                let path = entry.path()?;
                if path.extension().map_or(false, |ext| ext == EXE_SUFFIX) {
                    let mut bytes = Vec::new();
                    entry.read_to_end(&mut bytes)?;
                    found = Some(bytes);
                    break;
                }
            }
        }

        // Since the tar format also preserves executable flags,
        // if we *still* haven't found a file, we can look for any
        // file with executable permissions set and return that.
        if found.is_none() {
            for entry in tar.entries()? {
                let mut entry = entry?;
                let mode = entry.header().mode()?;
                if (mode & 0o111) != 0 {
                    let mut bytes = Vec::new();
                    entry.read_to_end(&mut bytes)?;
                    found = Some(bytes);
                    break;
                }
            }
        }

        tracing::debug!(
            num_bytes,
            elapsed = ?start.elapsed(),
            found = found.is_some(),
            "extracted tar file"
        );
        Ok(found)
    })
    .await?
}
