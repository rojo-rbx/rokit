use std::{
    env::{self, consts::EXE_SUFFIX},
    path::PathBuf,
    sync::OnceLock,
};

use async_once_cell::OnceCell as AsyncOnceCell;
use tokio::{fs::read, task::spawn_blocking};

static CURRENT_DIR: AsyncOnceCell<PathBuf> = AsyncOnceCell::new();
static CURRENT_EXE: AsyncOnceCell<PathBuf> = AsyncOnceCell::new();
static CURRENT_CONTENTS: AsyncOnceCell<Vec<u8>> = AsyncOnceCell::new();
static CURRENT_EXE_NAME: OnceLock<String> = OnceLock::new();

pub async fn current_dir() -> PathBuf {
    CURRENT_DIR
        .get_or_init(async move {
            let current = spawn_blocking(env::current_dir).await.unwrap();
            current.expect("Failed to get path to current directory")
        })
        .await
        .clone()
}

pub async fn current_exe() -> PathBuf {
    CURRENT_EXE
        .get_or_init(async move {
            let current = spawn_blocking(env::current_exe).await.unwrap();
            current.expect("Failed to get path to current executable")
        })
        .await
        .clone()
}

pub async fn current_exe_contents() -> Vec<u8> {
    CURRENT_CONTENTS
        .get_or_init(async move {
            let path = current_exe().await;
            let contents = read(path).await;
            contents.expect("Failed to read current executable")
        })
        .await
        .clone()
}

pub fn current_exe_name() -> String {
    CURRENT_EXE_NAME
        .get_or_init(|| {
            let arg0 = env::args().next().expect("Missing arg0");

            let exe_path = PathBuf::from(arg0);
            let exe_name = exe_path
                .file_name()
                .expect("Invalid file name passed as arg0")
                .to_str()
                .expect("Non-UTF8 file name passed as arg0");

            // NOTE: Shells on Windows can be weird sometimes and pass arg0
            // using either a lowercase or uppercase extension, so we fix that
            let exe_name = if EXE_SUFFIX.is_empty() {
                exe_name
            } else {
                let suffix_lower = EXE_SUFFIX.to_ascii_lowercase();
                let suffix_upper = EXE_SUFFIX.to_ascii_uppercase();
                if let Some(stripped) = exe_name.strip_suffix(&suffix_lower) {
                    stripped
                } else if let Some(stripped) = exe_name.strip_suffix(&suffix_upper) {
                    stripped
                } else {
                    exe_name
                }
            };

            exe_name.to_string()
        })
        .clone()
}
