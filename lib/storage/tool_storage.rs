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

use crate::{
    result::AftmanResult,
    tool::{ToolAlias, ToolSpec},
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

    fn aftman_path(&self) -> PathBuf {
        self.aliases_dir.join(format!("aftman{EXE_SUFFIX}"))
    }

    async fn aftman_contents(&self) -> AftmanResult<Vec<u8>> {
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
    ) -> AftmanResult<()> {
        let (dir_path, file_path) = self.tool_paths(spec);
        create_dir_all(dir_path).await?;
        write_executable(&file_path, contents).await?;
        Ok(())
    }

    /**
        Replaces the contents of the stored aftman binary.

        If `contents` is `None`, the current executable will
        be used, otherwise the given contents will be used.

        This would also update the cached contents of
        the current executable stored in this struct.
    */
    pub async fn replace_aftman_contents(&self, contents: Option<Vec<u8>>) -> AftmanResult<()> {
        let contents = match contents {
            Some(contents) => {
                self.current_exe_contents
                    .lock()
                    .await
                    .replace(contents.clone());
                contents
            }
            None => self.aftman_contents().await?,
        };
        write_executable(self.aftman_path(), &contents).await?;
        Ok(())
    }

    /**
        Creates a link for the given tool alias.

        Note that if the link already exists, it will be overwritten.
    */
    pub async fn create_tool_link(&self, alias: &ToolAlias) -> AftmanResult<()> {
        let path = self.aliases_dir.join(alias.name());
        let contents = self.aftman_contents().await?;
        write_executable(path, &contents).await?;
        Ok(())
    }

    /**
        Recreates all known links for tool aliases in the binary directory.

        This includes the link for Aftman itself - and if the link for Aftman does
        not exist, `true` will be returned to indicate that the link was created.
    */
    pub async fn recreate_all_links(&self) -> AftmanResult<bool> {
        let contents = self.aftman_contents().await?;
        let aftman_path = self.aftman_path();
        let mut aftman_found = false;

        let mut link_paths = Vec::new();
        let mut link_reader = read_dir(&self.aliases_dir).await?;
        while let Some(entry) = link_reader.next_entry().await? {
            let path = entry.path();
            if path != aftman_path {
                link_paths.push(path);
            } else {
                aftman_found = true;
            }
        }

        // Always write the Aftman binary to ensure it's up-to-date
        write_executable(&aftman_path, &contents).await?;

        // Then we can write the rest of the links - on unix we can use
        // symlinks pointing to the aftman binary to save on disk space.
        link_paths
            .into_iter()
            .map(|link_path| async {
                if cfg!(unix) {
                    write_executable_link(link_path, &aftman_path).await
                } else {
                    write_executable(link_path, &contents).await
                }
            })
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            .await?;

        Ok(!aftman_found)
    }

    pub(crate) async fn load(home_path: impl AsRef<Path>) -> AftmanResult<Self> {
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
}

async fn write_executable(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> AftmanResult<()> {
    let path = path.as_ref();

    use tokio::fs::write;
    write(path, contents).await?;

    #[cfg(unix)]
    {
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;
        use tokio::fs::set_permissions;
        set_permissions(path, Permissions::from_mode(0o755)).await?;
    }

    Ok(())
}

async fn write_executable_link(
    link_path: impl AsRef<Path>,
    target_path: impl AsRef<Path>,
) -> AftmanResult<()> {
    let link_path = link_path.as_ref();
    let target_path = target_path.as_ref();

    #[cfg(unix)]
    {
        use tokio::fs::symlink;
        symlink(target_path, link_path).await?;
    }

    // NOTE: We set the permissions of the symlink itself only on macOS
    // since that is the only supported OS where symlink permissions matter
    #[cfg(target_os = "macos")]
    {
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;
        use tokio::fs::set_permissions;
        set_permissions(link_path, Permissions::from_mode(0o755)).await?;
    }

    Ok(())
}
