use std::io::stderr;

use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[cfg(debug_assertions)]
const FMT_PRETTY: bool = true;

#[cfg(not(debug_assertions))]
const FMT_PRETTY: bool = false;

pub fn init() {
    let tracing_env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
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

    // Use the excessively verbose and pretty tracing-subscriber during
    // development, and a more concise and less pretty output in production.
    if FMT_PRETTY {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_env_filter)
            .with_writer(stderr)
            .pretty()
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_env_filter)
            .with_writer(stderr)
            .with_target(false)
            .without_time()
            .init();
    }
}
