use std::sync::Arc;
use std::time::Instant;

use reqwest::cookie::Jar;
use reqwest::Client;

/// Result of fetching a URL.
pub struct FetchResult {
    pub url: String,
    pub status: u16,
    pub content_type: String,
    pub html: String,
    pub html_bytes: usize,
    pub load_ms: u64,
}

/// Errors from the fetch layer.
#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("Navigation failed: {0}")]
    NavigationFailed(String),
    #[error("Timeout after {0}ms")]
    Timeout(u64),
    #[error("HTTP error {status}: {url}")]
    HttpError { status: u16, url: String },
}

/// Default User-Agent matching Chrome 128 on macOS.
const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";

/// Build an HTTP client with a shared cookie jar.
pub fn build_client(
    user_agent: Option<&str>,
    cookie_jar: Arc<Jar>,
) -> Result<Client, FetchError> {
    Client::builder()
        .user_agent(user_agent.unwrap_or(DEFAULT_USER_AGENT))
        .cookie_provider(cookie_jar)
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .map_err(|e| FetchError::NavigationFailed(e.to_string()))
}

/// Fetch a URL and return the HTML content.
pub async fn fetch_url(
    client: &Client,
    url: &str,
    timeout_ms: u64,
) -> Result<FetchResult, FetchError> {
    let start = Instant::now();

    let response = tokio::time::timeout(
        std::time::Duration::from_millis(timeout_ms),
        client.get(url).send(),
    )
    .await
    .map_err(|_| FetchError::Timeout(timeout_ms))?
    .map_err(|e| FetchError::NavigationFailed(e.to_string()))?;

    let status = response.status().as_u16();
    let final_url = response.url().to_string();
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("text/html")
        .to_string();

    if status >= 400 {
        return Err(FetchError::HttpError {
            status,
            url: final_url,
        });
    }

    let html = response
        .text()
        .await
        .map_err(|e| FetchError::NavigationFailed(e.to_string()))?;

    let html_bytes = html.len();
    let load_ms = start.elapsed().as_millis() as u64;

    Ok(FetchResult {
        url: final_url,
        status,
        content_type,
        html,
        html_bytes,
        load_ms,
    })
}
