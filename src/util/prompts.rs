use std::io::{stderr, IsTerminal};

use aftman::tool::ToolId;
use anyhow::{bail, Result};
use dialoguer::theme::ColorfulTheme;
use tokio::task::spawn_blocking;

pub async fn prompt_for_install_trust(tool_id: &ToolId) -> Result<bool> {
    let tool_id = tool_id.clone();
    spawn_blocking(move || prompt_for_install_trust_inner(&tool_id)).await?
}

fn prompt_for_install_trust_inner(tool_id: &ToolId) -> Result<bool> {
    let theme = ColorfulTheme::default();

    // If the terminal isn't interactive, tell the user that they
    // need to open an interactive terminal to trust this tool.
    if !stderr().is_terminal() {
        bail!(
            "The following tool has not been marked as trusted: {tool_id}\
            \nRun `aftman add {tool_id}` to install and trust this tool.",
        );
    }

    // Since the terminal is interactive, ask the user
    // if they're sure they want to install this tool.
    let trusted = dialoguer::Confirm::with_theme(&theme)
        .with_prompt(format!(
            "Tool '{tool_id}' has not been installed before - install it?",
        ))
        .default(true)
        .interact_opt()?;

    Ok(trusted.unwrap_or_default())
}
