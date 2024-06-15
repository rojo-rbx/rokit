use anyhow::Result;
use clap::Parser;
use console::style;

use rokit::{discovery::discover_all_manifests, storage::Home, system::current_dir, tool::ToolId};

/// Lists all existing tools managed by Rokit.
#[derive(Debug, Parser)]
pub struct ListSubcommand {
    /// A specific tool identifier to list installed versions for.
    pub id: Option<ToolId>,
}

impl ListSubcommand {
    pub async fn run(self, home: &Home) -> Result<()> {
        let (header, lines) = if let Some(id) = self.id {
            list_versions_for_id(home, &id)
        } else {
            list_versions(home).await
        };

        println!("{header}\n{}", lines.join("\n"));

        Ok(())
    }
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

// Lists versions for the current manifest, and the global manifest
async fn list_versions(home: &Home) -> (String, Vec<String>) {
    let cwd = current_dir().await;
    let manifests = discover_all_manifests(true, false).await;

    let bullet = style("•").dim();
    let arrow = style("→").dim();
    let at = style("@").dim();

    let mut manifest_lines = Vec::new();
    for manifest in manifests {
        let mut sorted_tools = manifest.tools.into_iter().collect::<Vec<_>>();
        sorted_tools.sort_by(|(alias_a, _), (alias_b, _)| alias_a.name().cmp(alias_b.name()));
        if sorted_tools.is_empty() {
            continue;
        }

        let longest_alias_len = sorted_tools
            .iter()
            .map(|(alias, _)| alias.name().len())
            .max()
            .unwrap_or(0);
        let longest_id_len = sorted_tools
            .iter()
            .map(|(_, spec)| spec.id().to_string().len())
            .max()
            .unwrap_or(0);

        let mut lines = Vec::new();
        for (alias, spec) in sorted_tools {
            lines.push(format!(
                "{bullet} {}{} {arrow} {} {}{at} {}",
                style(alias.name()).bold().cyan(),
                " ".repeat(longest_alias_len - alias.name().len()),
                spec.id(),
                " ".repeat(longest_id_len - spec.id().to_string().len()),
                spec.version(),
            ));
        }

        if lines.is_empty() {
            continue;
        }

        lines.sort();
        manifest_lines.push((manifest.path, lines));
    }

    let mut lines = vec![];
    for (index, (path, mlines)) in manifest_lines.iter().enumerate() {
        if let Ok(stripped) = path.strip_prefix(home.path()) {
            lines.push(format!("~/.rokit/{}", stripped.display()));
        } else if let Ok(stripped) = path.strip_prefix(&cwd) {
            lines.push(format!("./{}", stripped.display()));
        } else {
            lines.push(path.display().to_string());
        }
        lines.extend_from_slice(mlines);
        if index < manifest_lines.len() - 1 {
            lines.push(String::new()); // Add a newline between manifests
        }
    }

    if lines.is_empty() {
        let header = String::from("🛠️  No tools found.");
        (header, Vec::new())
    } else {
        let header = String::from("🛠️  Found tools:\n");
        (header, lines)
    }
}
