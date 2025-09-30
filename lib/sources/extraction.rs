#![allow(clippy::struct_excessive_bools)]

use std::{
    cmp::Reverse,
    env::consts::{EXE_EXTENSION, EXE_SUFFIX},
    io::{self, Read},
    path::{Path, PathBuf, MAIN_SEPARATOR_STR},
};

use tar::Archive as TarArchive;
use thiserror::Error;
use tokio::{task::spawn_blocking, time::Instant};
use zip::ZipArchive;

use crate::{
    descriptor::{Descriptor, OS},
    result::RokitResult,
    sources::ArtifactFormat,
};

#[derive(Debug, Error)]
pub enum ExtractError {
    #[error("unknown format")]
    UnknownFormat,
    #[error(
        "missing binary '{file_name}' \
        in {format} file '{archive_name}'"
    )]
    FileMissing {
        format: ArtifactFormat,
        file_name: String,
        archive_name: String,
    },
    #[error(
        "mismatch in OS for binary '{file_name}' in archive '{archive_name}'\
        \ncurrent OS is {current_os:?}, binary is {file_os:?}"
    )]
    OSMismatch {
        current_os: OS,
        file_os: OS,
        file_name: String,
        archive_name: String,
    },
    #[error(
        "{source}\
        \nresponse body first bytes:\
        \n{body}"
    )]
    Generic {
        source: Box<dyn std::error::Error + Send + Sync>,
        body: String,
    },
}

/**
    A candidate for extraction from an archive.

    Contains information about the path and how many
    properties matched compared to the desired file path.
*/
#[derive(Debug, Clone)]
struct Candidate {
    path: PathBuf,
    matched_full_path: bool,
    matched_file_exact: bool,   // Case-sensitive filename match
    matched_file_inexact: bool, // Case-insensitive filename match
    has_exec_perms: bool,       // Has executable permissions (UNIX only)
    has_exec_suffix: bool,      // Has an executable suffix (e.g. `.exe`)
    has_descriptor: bool,       // Has executable contents (any platform)
}

impl Candidate {
    fn new(
        desired_file_path: &Path,
        file_path: &Path,
        file_perms: Option<u32>,
        file_contents: &[u8],
    ) -> Option<Self> {
        let desired_file_name = desired_file_path.file_name()?;

        if file_path.ends_with(MAIN_SEPARATOR_STR) {
            return None;
        }

        let file_name = file_path.file_name();

        let matched_full_path = file_path == desired_file_path;
        let matched_file_exact = file_name == Some(desired_file_name);
        let matched_file_inexact =
            file_name.is_some_and(|name| name.eq_ignore_ascii_case(desired_file_name));

        let has_exec_perms = file_perms.is_some_and(|perms| (perms & 0o111) != 0);
        let has_exec_suffix = file_path
            .extension()
            .is_some_and(|ext| ext == EXE_EXTENSION);
        let has_descriptor = Descriptor::detect_from_executable(file_contents).is_some();

        Some(Self {
            path: file_path.to_path_buf(),
            matched_full_path,
            matched_file_exact,
            matched_file_inexact,
            has_exec_perms,
            has_exec_suffix,
            has_descriptor,
        })
    }

    fn score(&self) -> u32 {
        u32::from(self.matched_full_path)
            + u32::from(self.matched_file_exact)
            + u32::from(self.matched_file_inexact)
            + u32::from(self.has_exec_perms)
            + u32::from(self.has_exec_suffix)
            + u32::from(self.has_descriptor)
    }

    fn best(mut candidates: Vec<(Self, Vec<u8>)>) -> Option<(Self, Vec<u8>)> {
        // Sort by their score, best (highest score) first
        candidates.sort_by_key(|(c, _)| Reverse(c.score()));

        // The first candidate, if one exists, should now be the best one
        let (candidate, bytes) = candidates.into_iter().next()?;
        tracing::trace!(path = ?candidate.path, "found candidate");
        Some((candidate, bytes))
    }
}

/**
    Searches for and extracts the best matching file from a zip archive.

    May return `None` if no desired file was found in the archive.
*/
pub async fn extract_zip_file(
    zip_contents: impl AsRef<[u8]>,
    desired_file_name: impl Into<String>,
) -> RokitResult<Option<Vec<u8>>> {
    let desired_file_name = format!("{}{EXE_SUFFIX}", desired_file_name.into());
    let desired_file_path = PathBuf::from(&desired_file_name);

    let zip_contents = zip_contents.as_ref().to_vec();
    let zip_kilobytes = zip_contents.len() / 1024;
    let start = Instant::now();

    // Reading a zip file is a potentially expensive operation, so
    // spawn it as a blocking task and use the tokio thread pool.
    spawn_blocking(move || {
        let mut zip_cursor = io::Cursor::new(&zip_contents);
        let mut zip_reader = ZipArchive::new(&mut zip_cursor)?;

        // Extract all of the files in the archive, and only keep their
        // contents in memory if they were successfully ranked (score > 0)
        let candidates = (0..zip_reader.len()).filter_map(|index| {
            let mut entry = zip_reader.by_index(index).ok()?;
            if entry.is_dir() {
                return None;
            }

            let path = entry.enclosed_name()?;
            let perms = entry.unix_mode();

            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).ok()?;

            Candidate::new(&desired_file_path, &path, perms, &bytes)
                .filter(|candidate| candidate.score() > 0)
                .map(|candidate| (candidate, bytes))
        });

        // Pick the best candidate to extract, if any
        let (path, found) = match Candidate::best(candidates.collect()) {
            None => (None, None),
            Some((candidate, bytes)) => (Some(candidate.path), Some(bytes)),
        };

        tracing::debug!(
            size_archive = zip_kilobytes,
            size_binary = found.as_ref().map(|bytes| bytes.len() / 1024),
            elapsed = ?start.elapsed(),
            path = path.map(|path| path.display().to_string()),
            "extracted zip file"
        );
        Ok(found)
    })
    .await?
}

/**
    Searches for and extracts the best matching file from a tar archive.

    May return `None` if no desired file was found in the archive.
*/
pub async fn extract_tar_file(
    tar_contents: impl AsRef<[u8]>,
    desired_file_name: impl Into<String>,
) -> RokitResult<Option<Vec<u8>>> {
    let desired_file_name = format!("{}{EXE_SUFFIX}", desired_file_name.into());
    let desired_file_path = PathBuf::from(&desired_file_name);

    let tar_contents = tar_contents.as_ref().to_vec();
    let tar_kilobytes = tar_contents.len() / 1024;
    let start = Instant::now();

    // Reading a tar file is a potentially expensive operation, so
    // spawn it as a blocking task and use the tokio thread pool.
    spawn_blocking(move || {
        let mut tar_cursor = io::Cursor::new(&tar_contents);
        let mut tar_reader = TarArchive::new(&mut tar_cursor);

        // Extract all of the files in the archive, and only keep their
        // contents in memory if they were successfully ranked (score > 0)
        let candidates = tar_reader.entries_with_seek()?.filter_map(|entry| {
            let mut entry = entry.ok()?;
            if entry.header().entry_type().is_dir() {
                return None;
            }

            let path = entry.path().ok()?.to_path_buf();
            let perms = entry.header().mode().ok();

            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).ok()?;

            Candidate::new(&desired_file_path, &path, perms, &bytes)
                .filter(|candidate| candidate.score() > 0)
                .map(|candidate| (candidate, bytes))
        });

        // Pick the best candidate to extract, if any
        let (path, found) = match Candidate::best(candidates.collect()) {
            None => (None, None),
            Some((candidate, bytes)) => (Some(candidate.path), Some(bytes)),
        };

        tracing::debug!(
            size_archive = tar_kilobytes,
            size_binary = found.as_ref().map(|bytes| bytes.len() / 1024),
            elapsed = ?start.elapsed(),
            path = path.map(|path| path.display().to_string()),
            "extracted tar file"
        );
        Ok(found)
    })
    .await?
}
