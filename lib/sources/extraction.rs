#![allow(clippy::struct_excessive_bools)]

use std::{
    env::consts::{EXE_EXTENSION, EXE_SUFFIX},
    io::{self, Read},
    path::{Path, PathBuf, MAIN_SEPARATOR_STR},
};

use tar::Archive as TarArchive;
use thiserror::Error;
use tokio::{task::spawn_blocking, time::Instant};
use zip::ZipArchive;

use crate::{descriptor::OS, result::RokitResult, sources::ArtifactFormat};

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
}

impl Candidate {
    fn priority(&self) -> u32 {
        u32::from(self.matched_full_path)
            + u32::from(self.matched_file_exact)
            + u32::from(self.matched_file_inexact)
            + u32::from(self.has_exec_perms)
            + u32::from(self.has_exec_suffix)
    }

    fn find_best(
        entry_paths: impl AsRef<[(PathBuf, Option<u32>)]>,
        desired_file_path: impl AsRef<Path>,
    ) -> Option<Self> {
        let entry_paths = entry_paths.as_ref();
        let desired_file_path = desired_file_path.as_ref();
        let desired_file_name = desired_file_path.file_name()?.to_str()?;

        // Gather all candidates
        let mut candidates = entry_paths
            .iter()
            .filter_map(|(path, perms)| {
                if path.ends_with(MAIN_SEPARATOR_STR) {
                    return None;
                }

                let file_name = path.file_name().and_then(|name| name.to_str());

                let matched_full_path = path == desired_file_path;
                let matched_file_exact = file_name == Some(desired_file_name);
                let matched_file_inexact =
                    file_name.is_some_and(|name| name.eq_ignore_ascii_case(desired_file_name));

                let has_exec_perms = perms.is_some_and(|perms| (perms & 0o111) != 0);
                let has_exec_suffix = path.extension().is_some_and(|ext| ext == EXE_EXTENSION);

                Some(Self {
                    path: path.clone(),
                    matched_full_path,
                    matched_file_exact,
                    matched_file_inexact,
                    has_exec_perms,
                    has_exec_suffix,
                })
            })
            .filter(|c| c.priority() > 0) // Filter out candidates with no matches at all
            .collect::<Vec<_>>();

        // Sort by their priority, best first
        candidates.sort_by_key(Candidate::priority);
        candidates.reverse();

        // The first candidate, if one exists, should now be the best one
        let candidate = candidates.into_iter().next()?;
        tracing::trace!(path = ?candidate.path, "found candidate");
        Some(candidate)
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
    let num_kilobytes = zip_contents.len() / 1024;
    let start = Instant::now();

    // Reading a zip file is a potentially expensive operation, so
    // spawn it as a blocking task and use the tokio thread pool.
    spawn_blocking(move || {
        let mut found = None;
        let mut reader = io::Cursor::new(&zip_contents);
        let mut zip = ZipArchive::new(&mut reader)?;

        // Gather paths and their permissions,
        // avoiding reading the entire zip file
        let entry_paths = zip
            .file_names()
            .map(|name| {
                // NOTE: We don't need to sanitize the files names here
                // since we only use them for matching *within the zip file*
                (PathBuf::from(name), None::<u32>)
            })
            .collect::<Vec<_>>();

        // Find the best candidate to extract, if any
        let best = Candidate::find_best(entry_paths, &desired_file_path);
        if let Some(candidate) = best {
            if let Some(path_str) = candidate.path.to_str() {
                if let Ok(mut entry) = zip.by_name(path_str) {
                    let mut bytes = Vec::new();
                    entry.read_to_end(&mut bytes)?;
                    found = Some(bytes);
                }
            }
            if found.is_none() {
                tracing::warn!(
                    path = ?candidate.path,
                    "found candidate path, but failed to extract file"
                );
            }
        }

        tracing::debug!(
            num_kilobytes,
            elapsed = ?start.elapsed(),
            found = found.is_some(),
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
    let num_kilobytes = tar_contents.len() / 1024;
    let start = Instant::now();

    // Reading a tar file is a potentially expensive operation, so
    // spawn it as a blocking task and use the tokio thread pool.
    spawn_blocking(move || {
        let mut found = None;

        /*
            Gather paths and their permissions - note that we
            need to read the tar file twice to be able to use
            our find_best_candidate matching implementation...

            We can however use the `entries_with_seek` method
            to avoid reading actual file contents into memory.
        */
        let mut entry_cursor = io::Cursor::new(&tar_contents);
        let mut entry_reader = TarArchive::new(&mut entry_cursor);
        let entry_paths = entry_reader
            .entries_with_seek()?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                if entry.header().entry_type().is_dir() {
                    return None;
                }
                let path = entry.path().ok()?;
                let perms = entry.header().mode().ok();
                Some((path.to_path_buf(), perms))
            })
            .collect::<Vec<_>>();

        // Find the best candidate to extract, if any
        let best = Candidate::find_best(entry_paths, &desired_file_path);
        if let Some(candidate) = best {
            let contents_cursor = io::Cursor::new(&tar_contents);
            let mut contents_reader = TarArchive::new(contents_cursor);
            for entry in contents_reader.entries_with_seek()? {
                let mut entry = entry?;
                let entry_path = entry.path()?;
                if entry_path == candidate.path.as_path() {
                    let mut bytes = Vec::new();
                    entry.read_to_end(&mut bytes)?;
                    found = Some(bytes);
                    break;
                }
            }
            if found.is_none() {
                tracing::warn!(
                    path = ?candidate.path,
                    "found candidate path, but failed to extract file"
                );
            }
        }

        tracing::debug!(
            num_kilobytes,
            elapsed = ?start.elapsed(),
            found = found.is_some(),
            "extracted tar file"
        );
        Ok(found)
    })
    .await?
}
