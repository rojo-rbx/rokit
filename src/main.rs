use std::process::exit;

use clap::Parser;
use tracing::{error, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

mod cli;
mod util;
use cli::Cli;

#[tokio::main]
async fn main() {
    let tracing_env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy()
        // Adding the below extra directives will let us debug
        // aftman easier using RUST_LOG=debug or RUST_LOG=trace
        .add_directive("reqwest=info".parse().unwrap())
        .add_directive("rustls=info".parse().unwrap())
        .add_directive("hyper=info".parse().unwrap())
        .add_directive("h2=info".parse().unwrap());

    tracing_subscriber::fmt()
        .with_env_filter(tracing_env_filter)
        .with_target(false)
        .without_time()
        .init();

    if let Err(e) = Cli::parse().run().await {
        // NOTE: We use tracing for errors here for consistent
        // output between returned errors, and errors that
        // may be logged while the program is running.
        error!("{e}");
        exit(1);
    }
}
