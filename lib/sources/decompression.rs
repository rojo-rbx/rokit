use std::io::Read;

use flate2::read::GzDecoder;
use tokio::{task::spawn_blocking, time::Instant};

use crate::result::RokitResult;

pub async fn decompress_gzip(gz_contents: impl AsRef<[u8]>) -> RokitResult<Vec<u8>> {
    let gz_contents = gz_contents.as_ref().to_vec();
    let num_kilobytes = gz_contents.len() / 1024;
    let start = Instant::now();

    // Decompressing gzip is a potentially expensive operation, so
    // spawn it as a blocking task and use the tokio thread pool.
    spawn_blocking(move || {
        let mut decoder = GzDecoder::new(gz_contents.as_slice());
        let mut contents = Vec::new();
        decoder.read_to_end(&mut contents)?;

        tracing::trace!(
            num_kilobytes,
            elapsed = ?start.elapsed(),
            "decompressed gzip"
        );
        Ok(contents)
    })
    .await?
}
