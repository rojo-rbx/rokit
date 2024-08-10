use std::time::Duration;

use reqwest::{
    header::{HeaderMap, USER_AGENT},
    Client, Error,
};

use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;

/*
    Adds middleware for:

    - Retrying failed requests with exponential backoff
    - Tracing of HTTP requests
*/
fn add_client_middleware(client: Client) -> ClientWithMiddleware {
    ClientBuilder::new(client)
        .with(RetryTransientMiddleware::new_with_policy(
            ExponentialBackoff::builder().build_with_max_retries(3),
        ))
        .with(TracingMiddleware::default())
        .build()
}

/**
    Creates a client with:

    - HTTPS only
    - Timeouts for connection and response
    - All common compression algorithms enabled
    - User agent set to `<crate_name>/<crate_version> (<repository_url>)`
*/
pub fn create_client(mut default_headers: HeaderMap) -> Result<ClientWithMiddleware, Error> {
    let user_agent = format!(
        "{}/{} ({})",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_REPOSITORY"),
    );

    default_headers.insert(USER_AGENT, user_agent.parse().unwrap());

    let client = Client::builder()
        .default_headers(default_headers)
        .https_only(true)
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(60))
        .gzip(true)
        .brotli(true)
        .deflate(true)
        .build()?;

    Ok(add_client_middleware(client))
}
