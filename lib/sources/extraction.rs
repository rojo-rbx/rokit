use std::{
    env::consts::EXE_SUFFIX,
    io::{self, Read},
};

use tokio::{task::spawn_blocking, time::Instant};

use crate::result::RokitResult;

pub async fn extract_zip_file(
    zip_contents: Vec<u8>,
    desired_file_name: impl Into<String>,
) -> RokitResult<Option<Vec<u8>>> {
    let desired_file_name = format!("{}{EXE_SUFFIX}", desired_file_name.into());
    // Reading a zip file is a potentially expensive operation, so
    // spawn it as a blocking task and use the tokio thread pool.
    spawn_blocking(move || {
        let num_bytes = zip_contents.len();
        tracing::debug!(num_bytes, "Extracting zip file");
        let start = Instant::now();

        let mut found = None;
        let mut reader = io::Cursor::new(&zip_contents);
        let mut zip = zip::ZipArchive::new(&mut reader)?;

        // If there is a file with an
        // exact name match, return that ...
        for i in 0..zip.len() {
            let mut file = zip.by_index(i)?;

            if file.name() == desired_file_name {
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes)?;
                found = Some(bytes);
                break;
            }
        }

        // ...otherwise, look for any file with the
        // system's EXE_SUFFIX and return that.
        if found.is_none() {
            for i in 0..zip.len() {
                let mut file = zip.by_index(i)?;

                if file.name().ends_with(EXE_SUFFIX) {
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
            "Extracted zip file"
        );
        Ok(found)
    })
    .await?
}
