use anyhow::Result;
use clap::Parser;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

mod cli;
use cli::Args;

#[tokio::main]
async fn main() -> Result<()> {
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

    Args::parse().run().await
}
