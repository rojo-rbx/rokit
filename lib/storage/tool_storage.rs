use std::{
    env::{consts::EXE_SUFFIX, current_exe},
    path::{Path, PathBuf},
    sync::Arc,
};

use tokio::{
    fs::{create_dir_all, read, read_dir, write},
    sync::Mutex as AsyncMutex,
    task::{spawn_blocking, JoinSet},
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
        write(file_path, contents).await?;
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
        write(self.aftman_path(), &contents).await?;
        Ok(())
    }

    /**
        Creates a link for the given tool alias.

        Note that if the link already exists, it will be overwritten.
    */
    pub async fn create_tool_link(&self, alias: &ToolAlias) -> AftmanResult<()> {
        let path = self.aliases_dir.join(alias.name());
        let contents = self.aftman_contents().await?;
        write(&path, &contents).await?;
        Ok(())
    }

    /**
        Recreates all known links for tool aliases in the binary directory.

        This includes the link for Aftman itself - and if the link for Aftman does
        not exist, `true` will be returned to indicate that the link was created.
    */
    pub async fn recreate_all_links(&self) -> AftmanResult<bool> {
        let contents = self.aftman_contents().await?;

        let mut link_paths = Vec::new();
        let mut link_reader = read_dir(&self.aliases_dir).await?;
        while let Some(entry) = link_reader.next_entry().await? {
            link_paths.push(entry.path());
        }

        let aftman_path = self.aftman_path();
        let aftman_existed = if link_paths.contains(&aftman_path) {
            true
        } else {
            link_paths.push(aftman_path);
            false
        };

        let mut futures = JoinSet::new();
        for link_path in link_paths {
            futures.spawn(write(link_path, contents.clone()));
        }
        while let Some(result) = futures.join_next().await {
            result??;
        }

        Ok(!aftman_existed)
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
