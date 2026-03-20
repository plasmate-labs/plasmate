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
pub const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";

/// Build an HTTP client optimized for high-throughput agent browsing.
///
/// This client:
/// - Reuses TCP/TLS connections across requests (keep-alive)
/// - Negotiates HTTP/2 for multiplexed requests to the same host
/// - Accepts compressed responses (gzip, brotli, deflate)
/// - Skips unnecessary resources (we only want HTML)
/// - Uses rustls (no OpenSSL dependency)
pub fn build_client(user_agent: Option<&str>, cookie_jar: Arc<Jar>) -> Result<Client, FetchError> {
    Client::builder()
        .user_agent(user_agent.unwrap_or(DEFAULT_USER_AGENT))
        .cookie_provider(cookie_jar)
        .redirect(reqwest::redirect::Policy::limited(10))
        // Connection pooling: keep idle connections alive
        .pool_max_idle_per_host(16)
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        // Compression: smaller payloads = faster transfers
        .gzip(true)
        .brotli(true)
        .deflate(true)
        // TCP optimizations
        .tcp_nodelay(true)
        .tcp_keepalive(std::time::Duration::from_secs(60))
        // HTTP/1.1 quirks: some servers (e.g., eBay) send malformed chunked responses
        .http1_allow_obsolete_multiline_headers_in_responses(true)
        // HTTP/2: allow negotiation via ALPN (do not force prior knowledge)
        .build()
        .map_err(|e| FetchError::NavigationFailed(format!("{e:?}")))
}

/// Build a client that allows HTTP/1.1 fallback (for servers that don't support h2).
pub fn build_client_h1_fallback(
    user_agent: Option<&str>,
    cookie_jar: Arc<Jar>,
) -> Result<Client, FetchError> {
    Client::builder()
        .user_agent(user_agent.unwrap_or(DEFAULT_USER_AGENT))
        .cookie_provider(cookie_jar)
        .redirect(reqwest::redirect::Policy::limited(10))
        .pool_max_idle_per_host(16)
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .gzip(true)
        .brotli(true)
        .deflate(true)
        .tcp_nodelay(true)
        .tcp_keepalive(std::time::Duration::from_secs(60))
        // HTTP/1.1 quirks: some servers (e.g., eBay) send malformed chunked responses
        .http1_allow_obsolete_multiline_headers_in_responses(true)
        .build()
        .map_err(|e| FetchError::NavigationFailed(format!("{e:?}")))
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
        client
            .get(url)
            // Browser-realistic headers to avoid anti-bot blocking
            .header(
                "Accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
            )
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("Sec-Fetch-Dest", "document")
            .header("Sec-Fetch-Mode", "navigate")
            .header("Sec-Fetch-Site", "none")
            .header("Sec-Fetch-User", "?1")
            .header("Upgrade-Insecure-Requests", "1")
            .header("Cache-Control", "max-age=0")
            // Client hints
            .header("sec-ch-ua", "\"Chromium\";v=\"128\", \"Not;A=Brand\";v=\"24\"")
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", "\"macOS\"")
            .send(),
    )
    .await
    .map_err(|_| FetchError::Timeout(timeout_ms))?
    .map_err(|e| FetchError::NavigationFailed(format!("{e:?}")))?;

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

    // Use bytes() instead of text() to handle chunked encoding errors gracefully.
    // Some servers (e.g., eBay) send malformed chunked responses with "extra bytes after body".
    // We accept whatever bytes we received and convert to string, rather than failing entirely.
    let html = match response.bytes().await {
        Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(e) => {
            // If we got a decode error but partial body is available, that's still useful.
            // Unfortunately reqwest doesn't expose partial bytes on error, so we fail here.
            return Err(FetchError::NavigationFailed(format!(
                "Body decode error: {e:?}"
            )));
        }
    };

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

/// Fetch multiple URLs concurrently using a shared client (connection reuse).
pub async fn fetch_urls_parallel(
    client: &Client,
    urls: &[String],
    timeout_ms: u64,
    max_concurrent: usize,
) -> Vec<Result<FetchResult, FetchError>> {
    use futures_util::stream::{self, StreamExt};

    stream::iter(urls.iter())
        .map(|url| {
            let client = client.clone();
            let url = url.clone();
            async move { fetch_url(&client, &url, timeout_ms).await }
        })
        .buffer_unordered(max_concurrent)
        .collect()
        .await
}
