use tokio::fs::write;

use crate::{result::RokitResult, storage::Home};

const ENV_SHELL_FILE_PATH: &str = "env";
const ENV_SHELL_SCRIPT: &str = include_str!("./env.sh");

pub async fn add_to_path(home: &Home) -> RokitResult<bool> {
    // Write our shell script to the known location
    let file_path = home.path().join(ENV_SHELL_FILE_PATH);
    write(file_path, ENV_SHELL_SCRIPT).await?;

    // TODO: Add the path to known shell profile(s)

    Ok(false)
}
