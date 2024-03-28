use std::{
    collections::BTreeSet,
    io::{stderr, IsTerminal},
};

use anyhow::{bail, Context, Result};
use console::{style, Style};
use dialoguer::theme::ColorfulTheme;
use rokit::tool::{ToolId, ToolSpec};
use tokio::task::spawn_blocking;

#[derive(Debug, Clone, Copy)]
pub enum TrustPromptKind {
    Install,
    InstallMany,
}

pub async fn prompt_for_trust(tool_id: ToolId) -> Result<bool> {
    spawn_blocking(move || prompt_for_install_trust_inner(TrustPromptKind::Install, &tool_id))
        .await?
}

pub async fn prompt_for_trust_specs(tool_specs: Vec<ToolSpec>) -> Result<Vec<ToolSpec>> {
    spawn_blocking(move || {
        if tool_specs.is_empty() {
            Ok(Vec::new())
        } else if tool_specs.len() == 1 {
            println!("A tool is not yet trusted and needs your approval.");
            let spec = tool_specs.first().unwrap();
            if prompt_for_install_trust_inner(TrustPromptKind::Install, spec.id())? {
                Ok(vec![spec.clone()])
            } else {
                Ok(Vec::new())
            }
        } else {
            println!(
                "Some tools are not yet trusted and need your approval.\
                \nYou will be prompted for each tool individually, and \
                any tool you do not trust will not be installed."
            );
            let ids_to_prompt_for = tool_specs
                .iter()
                .map(|spec| spec.id().clone())
                .collect::<BTreeSet<_>>();

            let mut newly_trusted_ids = Vec::new();
            for id in ids_to_prompt_for {
                if prompt_for_install_trust_inner(TrustPromptKind::InstallMany, &id)? {
                    newly_trusted_ids.push(id);
                }
            }

            let newly_trusted_specs = tool_specs
                .into_iter()
                .filter(|spec| newly_trusted_ids.contains(spec.id()))
                .collect();
            Ok(newly_trusted_specs)
        }
    })
    .await?
}

fn prompt_for_install_trust_inner(kind: TrustPromptKind, tool_id: &ToolId) -> Result<bool> {
    let theme = ColorfulTheme {
        active_item_prefix: style("🔒 ".to_string()),
        prompt_style: Style::new(),
        ..Default::default()
    };

    // If the terminal isn't interactive, tell the user that they
    // need to open an interactive terminal to trust this tool.
    if !stderr().is_terminal() {
        bail!(
            "The following tool has not been marked as trusted: {tool_id}\
            \nRun `rokit add {tool_id}` to install and trust this tool.",
        );
    }

    // Since the terminal is interactive, ask the user
    // if they're sure they want to install this tool.
    let trusted = dialoguer::Confirm::with_theme(&theme)
        .with_prompt(match kind {
            TrustPromptKind::Install => format!("Trust and install {tool_id}?"),
            TrustPromptKind::InstallMany => format!("Trust {tool_id}?"),
        })
        .interact_opt()?
        .with_context(|| match kind {
            TrustPromptKind::Install => format!("Exited without trusting tool {tool_id}"),
            TrustPromptKind::InstallMany => String::from("Exited without trusting tools"),
        })?;

    Ok(trusted)
}
