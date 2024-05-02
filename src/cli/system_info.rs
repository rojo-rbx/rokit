use std::{
    collections::HashMap,
    fmt::Write,
    path::{Path, MAIN_SEPARATOR_STR},
};

use anyhow::Result;
use clap::Parser;
use console::style;
use futures::{stream::FuturesOrdered, TryStreamExt};
use tokio::{fs::read, task::spawn_blocking};

use rokit::{
    descriptor::Descriptor,
    storage::Home,
    system::{current_dir, current_exe, exists_in_path},
};

/// Prints out information about the current system and installed tools.
#[derive(Debug, Parser)]
pub struct SystemInfoSubcommand {}

impl SystemInfoSubcommand {
    pub async fn run(self, home: &Home) -> Result<()> {
        let cache = home.tool_cache();
        let storage = home.tool_storage();

        let bullet = style("•").dim();
        let arrow = style("→").dim();

        // Gather all installed tools and their descriptors

        let tool_specs = cache.all_installed();
        let tool_paths = tool_specs
            .clone()
            .into_iter()
            .map(|t| {
                let p = storage.tool_path(&t);
                (t, p)
            })
            .collect::<HashMap<_, _>>();
        let tool_bin_descriptors = tool_paths
            .clone()
            .into_iter()
            .map(|(t, p)| async move {
                let contents = read(p).await?;
                let descriptor =
                    spawn_blocking(move || Descriptor::detect_from_executable(contents))
                        .await
                        .unwrap();
                anyhow::Ok((t, descriptor))
            })
            .collect::<FuturesOrdered<_>>()
            .try_collect::<HashMap<_, _>>()
            .await?;

        // Write sections of information:
        // 1. Paths
        // 2. System
        // 3. Binaries
        // 4. Links

        let mut s = String::new();

        // Paths

        writeln!(s, "Paths:")?;
        writeln!(
            s,
            "  {bullet} Rokit dir   {arrow} {}",
            style(display_path(home.path()))
        )?;
        writeln!(
            s,
            "  {bullet} Current dir {arrow} {}",
            style(display_path(current_dir().await))
        )?;
        writeln!(
            s,
            "  {bullet} Current exe {arrow} {}",
            style(display_path(current_exe().await))
        )?;

        // System

        let current = Descriptor::current_system();
        writeln!(
            s,
            "\nSystem:\n  {bullet} {:?} {:?}{}",
            current.os(),
            current.arch(),
            if let Some(tc) = current.toolchain() {
                format!(" ({tc:?})")
            } else {
                String::new()
            }
        )?;
        if exists_in_path(home) {
            writeln!(s, "  {bullet} {}", style("Rokit in $PATH").bold().green())?;
        } else {
            writeln!(s, "  {bullet} {}", style("Rokit not in $PATH").bold().red())?;
        }

        // Binaries

        writeln!(s, "\nBinaries:")?;
        let longest_spec = tool_specs
            .iter()
            .map(|s| s.to_string().len())
            .max()
            .unwrap_or(0);
        for tool_spec in tool_specs {
            let _tool_path = tool_paths.get(&tool_spec).unwrap();
            let tool_desc = tool_bin_descriptors.get(&tool_spec).copied().flatten();
            let padding = " ".repeat(longest_spec - tool_spec.to_string().len());
            if let Some(tool_desc) = tool_desc {
                writeln!(
                    s,
                    "  {bullet} {tool_spec} {padding}{arrow} {:?} {:?}{}",
                    if tool_desc.os() == current.os() {
                        style(tool_desc.os())
                    } else {
                        style(tool_desc.os()).bold().red()
                    },
                    if tool_desc.arch() == current.arch() {
                        style(tool_desc.arch())
                    } else {
                        style(tool_desc.arch()).bold().yellow()
                    },
                    if let Some(tc) = tool_desc.toolchain() {
                        if current.toolchain() == Some(tc) {
                            format!(" ({})", style(tc.as_str()))
                        } else {
                            format!(" ({})", style(tc.as_str()).bold().yellow())
                        }
                    } else {
                        String::new()
                    }
                )?;
            } else {
                writeln!(
                    s,
                    "  {bullet} {tool_spec} {padding}{arrow} {}",
                    style("UNKNOWN").bold().yellow()
                )?;
            }
        }

        // Links

        writeln!(s, "\nLinks:")?;
        for alias in storage.all_link_paths().await? {
            writeln!(
                s,
                "  {bullet} {}{MAIN_SEPARATOR_STR}{}",
                display_path(alias.parent().expect("Links should have parents")),
                style(
                    alias
                        .file_name()
                        .expect("Links should not have empty file names")
                        .to_string_lossy()
                )
                .bold()
                .cyan()
            )?;
        }

        println!("{s}");

        Ok(())
    }
}

fn display_path(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    if let Some(user_home) = dirs::home_dir() {
        if let Ok(path) = path.strip_prefix(user_home) {
            return format!("~/{}", dunce::simplified(path).display());
        }
    }
    dunce::simplified(path).display().to_string()
}
