#![allow(clippy::struct_excessive_bools)]

use std::{
    collections::BTreeMap,
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
    fn priority(&self) -> u32 {
        u32::from(self.matched_full_path)
            + u32::from(self.matched_file_exact)
            + u32::from(self.matched_file_inexact)
            + u32::from(self.has_exec_perms)
            + u32::from(self.has_exec_suffix)
            + u32::from(self.has_descriptor)
    }

    fn find_best(
        entry_paths: impl AsRef<[(PathBuf, Option<u32>)]>,
        desired_file_path: impl AsRef<Path>,
        mut read_file_contents: impl FnMut(&Path) -> Option<Vec<u8>>,
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
                let has_descriptor = read_file_contents(path)
                    .and_then(Descriptor::detect_from_executable)
                    .is_some();

                Some(Self {
                    path: path.clone(),
                    matched_full_path,
                    matched_file_exact,
                    matched_file_inexact,
                    has_exec_perms,
                    has_exec_suffix,
                    has_descriptor,
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
        let mut zip_cursor = io::Cursor::new(&zip_contents);
        let mut zip_reader = ZipArchive::new(&mut zip_cursor)?;

        // Gather simple path + permissions pairs to find candidates from
        let entry_paths = zip_reader
            .file_names()
            .map(|name| (PathBuf::from(name), None::<u32>))
            .collect::<Vec<_>>();

        // Lazily cache any files that we read, to ensure that we do not
        // try to read a file entry which has already been read to its end
        let mut read_file_cache = BTreeMap::<_, Vec<u8>>::new();
        let mut read_file_contents = |path: &Path| {
            if let Some(existing) = read_file_cache.get(path) {
                Ok(existing.clone())
            } else if let Ok(mut entry) = zip_reader.by_name(path.to_str().unwrap()) {
                let mut bytes = Vec::new();
                entry.read_to_end(&mut bytes)?;
                read_file_cache.insert(path.to_path_buf(), bytes.clone());
                Ok(bytes)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("File not found: {}", path.display()),
                ))
            }
        };

        // Find the best candidate to extract, if any
        let best = Candidate::find_best(entry_paths, &desired_file_path, |path| {
            read_file_contents(path).ok()
        });
        let (path, found) = match best {
            None => (None, None),
            Some(candidate) => {
                let contents = read_file_contents(&candidate.path)?;
                (Some(candidate.path), Some(contents))
            }
        };

        tracing::debug!(
            num_kilobytes,
            elapsed = ?start.elapsed(),
            found_any = found.is_some(),
            found_path = path.map(|path| path.display().to_string()),
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
        // Gather paths and references to their respective entries,
        // without reading actual file contents into memory just yet
        let mut tar_cursor = io::Cursor::new(&tar_contents);
        let mut tar_reader = TarArchive::new(&mut tar_cursor);
        let mut tar_files = tar_reader
            .entries_with_seek()?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                if entry.header().entry_type().is_dir() {
                    return None;
                }
                let path = entry.path().ok()?;
                Some((path.to_path_buf(), entry))
            })
            .collect::<BTreeMap<PathBuf, _>>();

        // Map to simple path + permissions pairs to find candidates from
        let entry_paths = tar_files
            .iter()
            .map(|(path, entry)| {
                let perms = entry.header().mode().ok();
                (path.clone(), perms)
            })
            .collect::<Vec<_>>();

        // Lazily cache any files that we read, to ensure that we do not
        // try to read a file entry which has already been read to its end
        let mut read_file_cache = BTreeMap::<_, Vec<u8>>::new();
        let mut read_file_contents = |path: &Path| {
            if let Some(existing) = read_file_cache.get(path) {
                Ok(existing.clone())
            } else if let Some(entry) = tar_files.get_mut(path) {
                let mut bytes = Vec::new();
                entry.read_to_end(&mut bytes)?;
                read_file_cache.insert(path.to_path_buf(), bytes.clone());
                Ok(bytes)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("File not found: {}", path.display()),
                ))
            }
        };

        // Find the best candidate to extract, if any
        let best = Candidate::find_best(entry_paths, &desired_file_path, |path| {
            read_file_contents(path).ok()
        });
        let (path, found) = match best {
            None => (None, None),
            Some(candidate) => {
                let contents = read_file_contents(&candidate.path)?;
                (Some(candidate.path), Some(contents))
            }
        };

        tracing::debug!(
            num_kilobytes,
            elapsed = ?start.elapsed(),
            found_any = found.is_some(),
            found_path = path.map(|path| path.display().to_string()),
            "extracted tar file"
        );
        Ok(found)
    })
    .await?
}
