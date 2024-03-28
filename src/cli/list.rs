use anyhow::Result;
use clap::Parser;
use console::style;

use rokit::{storage::Home, tool::ToolId};

/// Lists all existing tools managed by Rokit.
#[derive(Debug, Parser)]
pub struct ListSubcommand {
    /// A specific tool identifier to list versions for.
    pub id: Option<ToolId>,
}

impl ListSubcommand {
    pub async fn run(self, home: &Home) -> Result<()> {
        let no_tools_installed = home.tool_cache().all_installed_ids().is_empty();

        let (header, lines) = if no_tools_installed {
            list_versions_for_empty(home)
        } else if let Some(id) = self.id {
            list_versions_for_id(home, &id)
        } else {
            list_versions_for_all(home)
        };

        println!(
            "{header}{}{}",
            if lines.is_empty() { "\n" } else { "\n\n" },
            lines.join("\n")
        );

        Ok(())
    }
}

// No tools are installed - just print out a message
fn list_versions_for_empty(_home: &Home) -> (String, Vec<String>) {
    let header = String::from("🛠️  No tools are installed.");
    (header, Vec::new())
}

// Lists all versions for a specific tool - if it is installed
fn list_versions_for_id(home: &Home, id: &ToolId) -> (String, Vec<String>) {
    let cache = home.tool_cache();

    let mut versions = cache.all_installed_versions_for_id(id);
    versions.reverse(); // List newest versions first

    if versions.is_empty() {
        let header = format!("🛠️  No versions of {id} are installed.");
        (header, Vec::new())
    } else {
        let header = format!("🛠️  Installed versions of {id}:");
        let bullet = style("•").dim();
        let lines = versions
            .into_iter()
            .map(|version| format!("  {bullet} {version}"))
            .collect();
        (header, lines)
    }
}

// Lists all versions for all installed tools
fn list_versions_for_all(home: &Home) -> (String, Vec<String>) {
    let cache = home.tool_cache();
    let tools = cache
        .all_installed_ids()
        .into_iter()
        .map(|id| (id.clone(), cache.all_installed_versions_for_id(&id)))
        .collect::<Vec<_>>();

    let header = String::from("🛠️  All installed tools:");
    let bullet = style("•").dim();
    let lines = tools
        .into_iter()
        .flat_map(|(id, mut versions)| {
            versions.reverse(); // List newest versions first
            let mut lines = vec![id.to_string()];
            for version in versions {
                lines.push(format!("  {bullet} {version}"));
            }
            lines
        })
        .collect();
    (header, lines)
}
