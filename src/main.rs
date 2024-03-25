use std::env::args;
use std::process::exit;
use std::str::FromStr;

use anyhow::{Context, Result};
use clap::Parser;
use tracing::{error, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

use aftman::{storage::Home, system::run_interruptible, tool::ToolAlias};

mod cli;
mod util;

use self::cli::Cli;
use self::util::{arg0_file_name, discover_closest_tool_spec};

#[cfg(debug_assertions)]
const FMT_PRETTY: bool = true;

#[cfg(not(debug_assertions))]
const FMT_PRETTY: bool = false;

#[tokio::main]
async fn main() {
    let tracing_env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy()
        // Adding the below extra directives will let us debug
        // aftman easier using RUST_LOG=debug or RUST_LOG=trace
        .add_directive("octocrab=info".parse().unwrap())
        .add_directive("reqwest=info".parse().unwrap())
        .add_directive("rustls=info".parse().unwrap())
        .add_directive("tower=info".parse().unwrap())
        .add_directive("hyper=info".parse().unwrap())
        .add_directive("h2=info".parse().unwrap());

    // Use the excessively verbose and pretty tracing-subscriber during
    // development, and a more concise and less pretty output in production.
    if FMT_PRETTY {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_env_filter)
            .pretty()
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_env_filter)
            .with_target(false)
            .without_time()
            .init();
    }

    let exe_name = arg0_file_name();
    let result = if exe_name == "aftman" {
        run_cli().await
    } else {
        run_tool(exe_name).await
    };

    if let Err(e) = result {
        // NOTE: We use tracing for errors here for consistent
        // output between returned errors, and errors that
        // may be logged while the program is running.
        error!("{e:?}");
        exit(1);
    }
}

async fn run_cli() -> Result<()> {
    Cli::parse().run().await
}

async fn run_tool(alias: String) -> Result<()> {
    let alias = ToolAlias::from_str(&alias)?;

    let home = Home::load_from_env().await?;

    let result = async {
        let spec = discover_closest_tool_spec(&home, &alias)
            .await
            .with_context(|| format!("Failed to find tool '{alias}'"))?;
        let path = home.tool_storage().tool_path(&spec);
        let args = args().skip(1).collect::<Vec<_>>();
        anyhow::Ok(run_interruptible(&path, &args).await?)
    }
    .await;

    home.save().await?;

    exit(result?);
}
