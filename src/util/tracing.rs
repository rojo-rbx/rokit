use std::io::stderr;

use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[cfg(debug_assertions)]
const FMT_PRETTY: bool = true;

#[cfg(not(debug_assertions))]
const FMT_PRETTY: bool = false;

pub fn init(default_level_filter: LevelFilter) {
    let tracing_env_filter = EnvFilter::builder()
        .with_default_directive(default_level_filter.into())
        .from_env_lossy()
        // Adding the below extra directives will let us debug
        // Rokit easier using RUST_LOG=debug or RUST_LOG=trace
        .add_directive("reqwest=info".parse().unwrap())
        .add_directive("rustls=info".parse().unwrap())
        .add_directive("tokio_util=info".parse().unwrap())
        .add_directive("goblin=info".parse().unwrap())
        .add_directive("tower=info".parse().unwrap())
        .add_directive("hyper=info".parse().unwrap())
        .add_directive("h2=info".parse().unwrap());

    // Show the target module in the tracing output during development
    // so that we can track down issues and trace origins faster.
    tracing_subscriber::fmt()
        .with_env_filter(tracing_env_filter)
        .with_writer(stderr)
        .with_target(FMT_PRETTY)
        .without_time()
        .init();
}
