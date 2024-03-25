use std::{
    env::{consts::EXE_SUFFIX, current_exe},
    path::{Path, PathBuf},
    sync::Arc,
};

use futures::{stream::FuturesUnordered, TryStreamExt};
use tokio::{
    fs::{create_dir_all, read, read_dir},
    sync::Mutex as AsyncMutex,
    task::spawn_blocking,
};
use tracing::debug;

use crate::{
    result::RokitResult,
    tool::{ToolAlias, ToolSpec},
    util::{write_executable_file, write_executable_link},
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
    current_exe_path: Arc<Path>,
    current_exe_contents: Arc<AsyncMutex<Option<Vec<u8>>>>,
}

impl ToolStorage {
    fn tool_paths(&self, spec: &ToolSpec) -> (PathBuf, PathBuf) {
        let tool_dir = self
            .tools_dir
            .join(spec.author())
            .join(spec.name())
            .join(spec.version().to_string());
        let tool_file = tool_dir.join(format!("{}{EXE_SUFFIX}", spec.name()));
        (tool_dir, tool_file)
    }

    fn rokit_path(&self) -> PathBuf {
        self.aliases_dir.join(format!("rokit{EXE_SUFFIX}"))
    }

    async fn rokit_contents(&self) -> RokitResult<Vec<u8>> {
        let mut guard = self.current_exe_contents.lock().await;
        if let Some(contents) = &*guard {
            return Ok(contents.clone());
        }
        let contents = read(&self.current_exe_path).await?;
        *guard = Some(contents.clone());
        Ok(contents)
    }

    /**
        Returns the path to the binary for the given tool.

        Note that this does not check if the binary actually exists.
    */
    pub fn tool_path(&self, spec: &ToolSpec) -> PathBuf {
        self.tool_paths(spec).1
    }

    /**
        Replaces the binary contents for the given tool.
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
        Replaces the contents of the stored Rokit binary.

        If `contents` is `None`, the current executable will
        be used, otherwise the given contents will be used.

        This would also update the cached contents of
        the current executable stored in this struct.
    */
    pub async fn replace_rokit_contents(&self, contents: Option<Vec<u8>>) -> RokitResult<()> {
        let contents = match contents {
            Some(contents) => {
                self.current_exe_contents
                    .lock()
                    .await
                    .replace(contents.clone());
                contents
            }
            None => self.rokit_contents().await?,
        };
        write_executable_file(self.rokit_path(), &contents).await?;
        Ok(())
    }

    /**
        Creates a link for the given tool alias.

        Note that if the link already exists, it will be overwritten.
    */
    pub async fn create_tool_link(&self, alias: &ToolAlias) -> RokitResult<()> {
        let path = self.aliases_dir.join(alias.name());
        let contents = self.rokit_contents().await?;
        write_executable_file(path, &contents).await?;
        Ok(())
    }

    /**
        Recreates all known links for tool aliases in the binary directory.
        This includes the link / main executable for Rokit itself.

        Returns a tuple with information about any existing Rokit link:

        - The first value is `true` if the existing Rokit link was found, `false` otherwise.
        - The second value is `true` if the existing Rokit link was different compared to the
          newly written Rokit binary, `false` otherwise. This is useful for determining if
          the Rokit binary itself existed but was updated, such as during `self-install`.
    */
    pub async fn recreate_all_links(&self) -> RokitResult<(bool, bool)> {
        let contents = self.rokit_contents().await?;
        let rokit_path = self.rokit_path();
        let mut rokit_found = false;

        let mut link_paths = Vec::new();
        let mut link_reader = read_dir(&self.aliases_dir).await?;
        while let Some(entry) = link_reader.next_entry().await? {
            let path = entry.path();
            if path != rokit_path {
                debug!(?path, "Found existing link");
                link_paths.push(path);
            } else {
                rokit_found = true;
            }
        }

        // Always write the Rokit binary to ensure it's up-to-date
        let existing_rokit_binary = read(&rokit_path).await.unwrap_or_default();
        let was_rokit_updated = existing_rokit_binary != contents;
        write_executable_file(&rokit_path, &contents).await?;

        // Then we can write the rest of the links - on unix we can use
        // symlinks pointing to the Rokit binary to save on disk space.
        link_paths
            .into_iter()
            .map(|link_path| async {
                if cfg!(unix) {
                    write_executable_link(link_path, &rokit_path).await
                } else {
                    write_executable_file(link_path, &contents).await
                }
            })
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            .await?;

        Ok((rokit_found, was_rokit_updated))
    }

    pub(crate) async fn load(home_path: impl AsRef<Path>) -> RokitResult<Self> {
        let home_path = home_path.as_ref();

        let tools_dir = home_path.join("tool-storage").into();
        let aliases_dir = home_path.join("bin").into();

        let (_, _, current_exe_res) = tokio::try_join!(
            create_dir_all(&tools_dir),
            create_dir_all(&aliases_dir),
            // NOTE: A call to current_exe is blocking on some
            // platforms, so we spawn it in a blocking task here.
            async { Ok(spawn_blocking(current_exe).await?) },
        )?;

        let current_exe_path = current_exe_res?.into();
        let current_exe_contents = Arc::new(AsyncMutex::new(None));

        Ok(Self {
            current_exe_path,
            current_exe_contents,
            tools_dir,
            aliases_dir,
        })
    }

    pub(crate) fn needs_saving(&self) -> bool {
        // Tool storage always writes all state directly
        // to the disk, but this may change in the future
        false
    }
}
