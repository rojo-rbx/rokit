use std::path::PathBuf;

use futures::{stream::FuturesUnordered, StreamExt};
use tokio::{
    fs::{read_to_string, write},
    io::ErrorKind,
};

use crate::{
    result::{RokitError, RokitResult},
    storage::Home,
};

use super::shell::Shell;

const ENV_SHELL_FILE_PATH: &str = "env";
const ENV_SHELL_SCRIPT: &str = include_str!("./env.sh");

pub async fn add_to_path(home: &Home) -> RokitResult<bool> {
    // Find our binaries dir and try to format it as "$HOME/.rokit/bin"
    let bin_dir = home.path().join("bin");
    let bin_dir_str = bin_dir.to_str().ok_or(RokitError::InvalidUtf8)?;
    let bin_dir_in_home = replace_home_path_with_var(bin_dir_str);

    // Do the same for the shell script path - "$HOME/.rokit/env"
    let file_path = home.path().join(ENV_SHELL_FILE_PATH);
    let file_path_str = file_path.to_str().ok_or(RokitError::InvalidUtf8)?;
    let file_path_in_home = replace_home_path_with_var(file_path_str);

    // Write our shell init script to the known location
    let file_contents = ENV_SHELL_SCRIPT.replace("{rokit_bin_path}", &bin_dir_in_home);
    write(file_path, file_contents).await?;

    // Add the path to known shell profiles
    let added_any = if let Some(home_dir) = dirs::home_dir() {
        let futs = Shell::ALL
            .iter()
            .map(|shell| {
                let shell_env_path = home_dir.join(shell.env_file_path());
                let shell_should_create = shell.env_file_should_create_if_nonexistent();
                append_to_shell_file(
                    shell_env_path,
                    format!(". \"{file_path_in_home}\""),
                    shell_should_create,
                )
            })
            .collect::<FuturesUnordered<_>>();
        // NOTE: append_to_shell_file returns `true` if the line was added,
        // we need to preserve this information, but also not fail if
        // any of the file operations do, so we unwrap_or_default
        futs.collect::<Vec<_>>()
            .await
            .into_iter()
            .any(Result::unwrap_or_default)
    } else {
        false
    };

    Ok(added_any)
}

async fn append_to_shell_file(
    file_path: PathBuf,
    line_to_append: String,
    create_if_nonexistent: bool,
) -> RokitResult<bool> {
    let mut file_contents = match read_to_string(&file_path).await {
        Ok(contents) => contents,
        Err(e) if e.kind() == ErrorKind::NotFound && create_if_nonexistent => String::new(),
        Err(e) => return Err(e.into()),
    };

    if file_contents.contains(&line_to_append) {
        return Ok(false);
    }

    // NOTE: Make sure we put the new contents on their own
    // line and not conflicting with any existing command(s)
    if !file_contents.ends_with('\n') {
        file_contents.push('\n');
    }

    file_contents.push_str(&line_to_append);
    file_contents.push('\n');

    write(file_path, file_contents).await?;

    Ok(true)
}

fn replace_home_path_with_var(path: &str) -> String {
    let Some(home_dir) = dirs::home_dir() else {
        return path.to_string();
    };
    let Some(home_dir_str) = home_dir.to_str() else {
        return path.to_string();
    };
    path.replace(home_dir_str, "$HOME")
}
