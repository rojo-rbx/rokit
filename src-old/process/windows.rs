//! On Windows, we use command_group to spawn processes in a job group that will
//! be automatically cleaned up when this process exits.

use std::path::Path;
use std::process::Command;

use anyhow::Context;
use command_group::CommandGroup;

pub async fn run(exe_path: &Path, args: Vec<String>) -> anyhow::Result<i32> {
    // On Windows, using a job group here will cause the subprocess to terminate
    // automatically when Aftman is terminated.
    let mut child = Command::new(exe_path)
        .args(args)
        .group_spawn()
        .with_context(|| format!("Could not spawn {}", exe_path.display()))?;

    let status = child.wait()?;
    Ok(status.code().unwrap_or(1))
}
