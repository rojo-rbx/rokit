use tokio::fs::write;

use crate::{
    result::{RokitError, RokitResult},
    storage::Home,
};

const ENV_SHELL_FILE_PATH: &str = "env";
const ENV_SHELL_SCRIPT: &str = include_str!("./env.sh");

pub async fn add_to_path(home: &Home) -> RokitResult<bool> {
    // Write our shell script to the known location
    let bin_dir = home.path().join("bin");
    let file_path = home.path().join(ENV_SHELL_FILE_PATH);
    let file_contents = ENV_SHELL_SCRIPT.replace(
        "{rokit_bin_path}",
        bin_dir.to_str().ok_or(RokitError::InvalidUtf8)?,
    );
    write(file_path, file_contents).await?;

    // TODO: Add the path to known shell profile(s)

    Ok(false)
}
