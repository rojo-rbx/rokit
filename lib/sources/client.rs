use std::time::Duration;

use reqwest::{header::HeaderMap, Client, Error};
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
*/
pub fn create_client(default_headers: HeaderMap) -> Result<ClientWithMiddleware, Error> {
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
