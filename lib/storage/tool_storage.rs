use std::{
    env::{
        consts::{EXE_EXTENSION, EXE_SUFFIX},
        var,
    },
    path::{Path, PathBuf},
    sync::Arc,
};

use filepath::FilePath;
use futures::{stream::FuturesUnordered, TryStreamExt};
use tokio::{
    fs::{create_dir_all, read, read_dir, remove_file, rename},
    sync::Mutex as AsyncMutex,
};
use tracing::{debug, trace};

use crate::{
    manifests::{AuthManifest, RokitManifest},
    result::RokitResult,
    system::current_exe_contents,
    tool::{ToolAlias, ToolSpec},
    util::fs::{path_exists, write_executable_file, write_executable_link},
};

/**
    Storage for tool binaries and aliases.

    Can be cheaply cloned while still
    referring to the same underlying data.
*/
#[derive(Debug, Clone)]
pub struct ToolStorage {
    pub(super) tools_dir: Arc<Path>,
    pub(super) aliases_dir: Arc<Path>,
    current_rokit_contents: Arc<AsyncMutex<Option<Vec<u8>>>>,
    no_symlinks: bool,
}

impl ToolStorage {
    fn tool_paths(&self, spec: &ToolSpec) -> (PathBuf, PathBuf) {
        // NOTE: We use uncased strings for the tool author and name
        // to ensure that the tool paths are always case-insensitive
        let tool_dir = self
            .tools_dir
            .join(spec.id.author.uncased_str())
            .join(spec.id.name.uncased_str())
            .join(spec.version.to_string());

        let tool_file_name = format!("{}{EXE_SUFFIX}", spec.id.name.uncased_str());
        let tool_file = tool_dir.join(tool_file_name);

        (tool_dir, tool_file)
    }

    fn alias_path(&self, alias: &ToolAlias) -> PathBuf {
        let alias_file_name = format!("{}{EXE_SUFFIX}", alias.name.uncased_str());
        self.aliases_dir.join(alias_file_name)
    }

    fn rokit_path(&self) -> PathBuf {
        self.aliases_dir.join(format!("rokit{EXE_SUFFIX}"))
    }

    async fn rokit_contents(&self) -> RokitResult<Vec<u8>> {
        let mut guard = self.current_rokit_contents.lock().await;
        if let Some(contents) = &*guard {
            return Ok(contents.clone());
        }
        let contents = current_exe_contents().await;
        *guard = Some(contents.clone());
        Ok(contents)
    }

    /**
        Returns the path to the binary for the given tool.

        Note that this does not check if the binary actually exists.
    */
    #[must_use]
    pub fn tool_path(&self, spec: &ToolSpec) -> PathBuf {
        self.tool_paths(spec).1
    }

    /**
        Replaces the binary contents for the given tool.

        # Errors

        - If the binary could not be written.
    */
    pub async fn replace_tool_contents(
        &self,
        spec: &ToolSpec,
        contents: impl AsRef<[u8]>,
    ) -> RokitResult<()> {
        let (dir_path, file_path) = self.tool_paths(spec);
        create_dir_all(dir_path).await?;
        write_executable_file(&file_path, contents).await?;
        Ok(())
    }

    /**
        Replaces the contents of the stored Rokit binary in memory.

        Note that this **does not** update the actual Rokit binary or any links.

        To update the Rokit binary and all links, use `recreate_all_links`.
    */
    pub async fn replace_rokit_contents(&self, contents: Vec<u8>) {
        self.current_rokit_contents.lock().await.replace(contents);
    }

    /**
        Creates a link for the given tool alias.

        Note that if the link already exists, it will be overwritten.

        # Errors

        - If the link could not be written.
    */
    pub async fn create_tool_link(&self, alias: &ToolAlias) -> RokitResult<()> {
        let path = self.alias_path(alias);

        // NOTE: A previous version of Rokit was not adding exe extensions correctly,
        // so look for and try to remove existing links that do not have the extension
        if should_check_exe_extensions() {
            let no_extension = strip_exe_extension(&path);
            if no_extension != path && path_exists(&no_extension).await {
                remove_file(&no_extension).await?;
            }
        }

        // Create the new link
        if cfg!(unix) && !self.no_symlinks {
            let rokit_path = self.rokit_path();
            write_executable_link(path, &rokit_path).await?;
        } else {
            let contents = self.rokit_contents().await?;
            write_executable_file(path, &contents).await?;
        }

        Ok(())
    }

    /**
        Reads all currently known link paths for tool aliases in the binary directory.

        This *does not* include the link / main executable for Rokit itself.

        # Errors

        - If the directory could not be read.
        - If any link could not be read.
    */
    pub async fn all_link_paths(&self) -> RokitResult<Vec<PathBuf>> {
        let rokit_path = self.rokit_path();

        let mut link_paths = Vec::new();
        let mut link_reader = read_dir(&self.aliases_dir).await?;
        while let Some(entry) = link_reader.next_entry().await? {
            let path = entry.path();
            if path == rokit_path {
                debug!(?path, "found Rokit link");
            } else {
                debug!(?path, "found tool link");
                link_paths.push(path);
            }
        }

        Ok(link_paths)
    }

    /**
        Recreates all known links for tool aliases in the binary directory.
        This includes the link / main executable for Rokit itself.

        Returns a tuple with information about any existing Rokit link:

        - The first value is `true` if the existing Rokit link was found, `false` otherwise.
        - The second value is `true` if the existing Rokit link was different compared to the
          newly written Rokit binary, `false` otherwise. This is useful for determining if
          the Rokit binary itself existed but was updated, such as during `self-install`.

        # Errors

        - If any link could not be written.
    */
    pub async fn recreate_all_links(&self) -> RokitResult<(bool, bool)> {
        let rokit_path = self.rokit_path();
        let rokit_contents = self.rokit_contents().await?;
        let rokit_link_existed = path_exists(&rokit_path).await;

        let mut link_paths = self.all_link_paths().await?;

        // NOTE: A previous version of Rokit was not adding exe extensions correctly,
        // so look for and try to remove existing links that do not have the extension
        if should_check_exe_extensions() {
            for link_path in &mut link_paths {
                if !has_exe_extension(&link_path) {
                    remove_file(&link_path).await?;
                    *link_path = append_exe_extension(&link_path);
                }
            }
        }

        // Write the Rokit binary if necessary to ensure it's up-to-date
        let existing_rokit_binary = read(&rokit_path).await.unwrap_or_default();
        let was_rokit_updated = if existing_rokit_binary == rokit_contents {
            false
        } else {
            // NOTE: If the currently running Rokit binary is being updated,
            // we need to move it to a temporary location first to avoid issues
            // with the OS killing the current executable when its overwritten.
            if rokit_link_existed {
                let temp_file = tempfile::tempfile()?;
                let temp_path = temp_file.path()?;
                trace!(
                    ?temp_path,
                    "moving existing Rokit binary to temporary location"
                );
                rename(&rokit_path, temp_path).await?;
            }
            write_executable_file(&rokit_path, &rokit_contents).await?;
            true
        };

        // Then we can write the rest of the links - on unix we can use
        // symlinks pointing to the Rokit binary to save on disk space.
        link_paths
            .into_iter()
            .map(|link_path| async {
                if cfg!(unix) && !self.no_symlinks {
                    write_executable_link(link_path, &rokit_path).await
                } else {
                    write_executable_file(link_path, &rokit_contents).await
                }
            })
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            .await?;

        Ok((rokit_link_existed, was_rokit_updated))
    }

    pub(crate) async fn load(home_path: impl AsRef<Path>) -> RokitResult<Self> {
        let home_path = home_path.as_ref();

        let tools_dir = home_path.join("tool-storage").into();
        let aliases_dir = home_path.join("bin").into();

        tokio::try_join!(
            RokitManifest::load_or_create(&home_path),
            AuthManifest::load_or_create(&home_path),
            async { Ok(create_dir_all(&tools_dir).await?) },
            async { Ok(create_dir_all(&aliases_dir).await?) },
        )?;

        let current_rokit_contents = Arc::new(AsyncMutex::new(None));
        let no_symlinks = var("ROKIT_NO_SYMLINKS")
            .is_ok_and(|val| matches!(val.to_ascii_lowercase().as_str(), "1" | "true"));

        Ok(Self {
            tools_dir,
            aliases_dir,
            current_rokit_contents,
            no_symlinks,
        })
    }

    #[allow(clippy::unused_self)]
    pub(crate) fn needs_saving(&self) -> bool {
        // Tool storage always writes all state directly
        // to the disk, but this may change in the future
        false
    }
}

// Utility functions for migrating missing exe extensions from old Rokit versions

fn should_check_exe_extensions() -> bool {
    !EXE_EXTENSION.is_empty()
}

fn has_exe_extension(path: impl AsRef<Path>) -> bool {
    !EXE_EXTENSION.is_empty()
        && path
            .as_ref()
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(EXE_EXTENSION))
}

fn strip_exe_extension(path: impl Into<PathBuf>) -> PathBuf {
    let mut path: PathBuf = path.into();
    if has_exe_extension(&path) {
        path.set_extension("");
    }
    path
}

fn append_exe_extension(path: impl Into<PathBuf>) -> PathBuf {
    let mut path: PathBuf = path.into();
    if !has_exe_extension(&path) {
        path.set_extension(EXE_EXTENSION);
    }
    path
}
