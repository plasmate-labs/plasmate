use std::sync::Arc;
use std::time::Instant;

use futures_util::StreamExt;
use reqwest::cookie::Jar;
use reqwest::Client;
use url::Url;

use super::proxy::ProxyConfig;
use super::security::{OutboundUrlPolicy, PolicyDnsResolver};
use super::tls::TlsConfig;

/// Result of fetching a URL.
pub struct FetchResult {
    pub url: String,
    pub status: u16,
    pub content_type: String,
    pub html: String,
    pub html_bytes: usize,
    pub load_ms: u64,
    /// Set-Cookie headers from the response (for CDP cookie jar sync).
    pub set_cookies: Vec<String>,
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
    #[error("Outbound URL blocked: {0}")]
    UrlBlocked(String),
    #[error("Too many redirects (maximum {0})")]
    TooManyRedirects(usize),
    #[error("Response body exceeds the configured {limit} byte limit")]
    BodyTooLarge { limit: usize },
}

#[derive(Debug, Clone, Copy)]
pub struct FetchLimits {
    /// Maximum declared wire size. This rejects oversized Content-Length before reading.
    pub max_compressed_bytes: usize,
    /// Maximum bytes after bounded gzip/brotli/deflate decoding.
    pub max_body_bytes: usize,
    pub max_redirects: usize,
}

impl Default for FetchLimits {
    fn default() -> Self {
        Self {
            max_compressed_bytes: env_usize("PLASMATE_MAX_COMPRESSED_BYTES", 8 * 1024 * 1024),
            max_body_bytes: env_usize("PLASMATE_MAX_BODY_BYTES", 16 * 1024 * 1024),
            max_redirects: env_usize("PLASMATE_MAX_REDIRECTS", 5).min(20),
        }
    }
}

fn env_usize(name: &str, fallback: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(fallback)
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
pub fn build_client(
    user_agent: Option<&str>,
    cookie_jar: Arc<Jar>,
    tls_config: Option<&TlsConfig>,
) -> Result<Client, FetchError> {
    build_client_with_policy(
        user_agent,
        cookie_jar,
        tls_config,
        OutboundUrlPolicy::from_environment(),
    )
}

/// Build a client for the deterministic benchmark's ephemeral loopback server.
/// This remains crate-private so product call surfaces cannot opt out of SSRF policy.
pub(crate) fn build_client_for_local_fixture(cookie_jar: Arc<Jar>) -> Result<Client, FetchError> {
    build_client_with_policy(
        None,
        cookie_jar,
        None,
        OutboundUrlPolicy::for_local_fixtures(),
    )
}

/// Build a client whose DNS resolver is permanently restricted to public
/// destinations. Unlike the general browser client, this never reads the
/// process-wide private-network development escape hatch.
#[derive(Debug)]
pub(crate) struct PublicOnlyClient(Client);

pub(crate) fn build_client_public_only(
    cookie_jar: Arc<Jar>,
) -> Result<PublicOnlyClient, FetchError> {
    build_client_public_only_with_user_agent(cookie_jar, None)
}

/// Build a permanently public-network-only client with an explicit product
/// token as its HTTP User-Agent. This is used by policy discovery surfaces
/// whose declared crawler identity must match the evaluated robots group.
pub(crate) fn build_client_public_only_with_user_agent(
    cookie_jar: Arc<Jar>,
    user_agent: Option<&str>,
) -> Result<PublicOnlyClient, FetchError> {
    build_client_with_policy(
        user_agent,
        cookie_jar,
        None,
        OutboundUrlPolicy::public_network_only(),
    )
    .map(PublicOnlyClient)
}

fn build_client_with_policy(
    user_agent: Option<&str>,
    cookie_jar: Arc<Jar>,
    tls_config: Option<&TlsConfig>,
    policy: OutboundUrlPolicy,
) -> Result<Client, FetchError> {
    let mut builder = Client::builder()
        .user_agent(user_agent.unwrap_or(DEFAULT_USER_AGENT))
        .cookie_provider(cookie_jar)
        .redirect(reqwest::redirect::Policy::none())
        .dns_resolver(Arc::new(PolicyDnsResolver::with_policy(policy)))
        // Connection pooling: keep idle connections alive
        .pool_max_idle_per_host(16)
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        // Compression: smaller payloads = faster transfers
        // Keep wire bytes visible so both compressed and decoded limits are hard.
        .gzip(false)
        .brotli(false)
        .deflate(false)
        // TCP optimizations
        .tcp_nodelay(true)
        .tcp_keepalive(std::time::Duration::from_secs(60))
        // HTTP/1.1 quirks: some servers (e.g., eBay) send malformed chunked responses
        .http1_allow_obsolete_multiline_headers_in_responses(true);
    // HTTP/2: allow negotiation via ALPN (do not force prior knowledge)

    builder = apply_tls_config(builder, tls_config)?;

    builder
        .build()
        .map_err(|e| FetchError::NavigationFailed(format!("{e:?}")))
}

/// Build a client that allows HTTP/1.1 fallback (for servers that don't support h2).
pub fn build_client_h1_fallback(
    user_agent: Option<&str>,
    cookie_jar: Arc<Jar>,
    tls_config: Option<&TlsConfig>,
) -> Result<Client, FetchError> {
    build_client_with_proxy(user_agent, cookie_jar, tls_config, None, None)
}

/// Build an HTTP/1.1 client with optional extra default headers.
pub fn build_client_h1_fallback_with_headers(
    user_agent: Option<&str>,
    cookie_jar: Arc<Jar>,
    tls_config: Option<&TlsConfig>,
    extra_headers: Option<&std::collections::HashMap<String, String>>,
) -> Result<Client, FetchError> {
    build_client_with_proxy(user_agent, cookie_jar, tls_config, None, extra_headers)
}

/// Build an HTTP client with optional proxy support.
///
/// This is the main client builder that supports all configuration options:
/// - User agent
/// - Cookie jar
/// - TLS configuration (fingerprinting, certs)
/// - Proxy configuration (HTTP, HTTPS, SOCKS5)
/// - Extra headers
pub fn build_client_with_proxy(
    user_agent: Option<&str>,
    cookie_jar: Arc<Jar>,
    tls_config: Option<&TlsConfig>,
    proxy_config: Option<&ProxyConfig>,
    extra_headers: Option<&std::collections::HashMap<String, String>>,
) -> Result<Client, FetchError> {
    let mut headers = reqwest::header::HeaderMap::new();
    if let Some(eh) = extra_headers {
        for (k, v) in eh {
            if let (Ok(name), Ok(val)) = (
                reqwest::header::HeaderName::from_bytes(k.as_bytes()),
                reqwest::header::HeaderValue::from_str(v),
            ) {
                headers.insert(name, val);
            }
        }
    }

    let mut builder = Client::builder()
        .user_agent(user_agent.unwrap_or(DEFAULT_USER_AGENT))
        .default_headers(headers)
        .cookie_provider(cookie_jar)
        .redirect(reqwest::redirect::Policy::none())
        .dns_resolver(Arc::new(PolicyDnsResolver::from_environment()))
        .pool_max_idle_per_host(16)
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        // Keep wire bytes visible so both compressed and decoded limits are hard.
        .gzip(false)
        .brotli(false)
        .deflate(false)
        .tcp_nodelay(true)
        .tcp_keepalive(std::time::Duration::from_secs(60))
        // HTTP/1.1 quirks: some servers (e.g., eBay) send malformed chunked responses
        .http1_allow_obsolete_multiline_headers_in_responses(true);

    // Apply TLS configuration
    builder = apply_tls_config(builder, tls_config)?;

    // Apply proxy configuration
    if let Some(proxy) = proxy_config {
        builder = proxy
            .apply_to_builder(builder)
            .map_err(FetchError::NavigationFailed)?;
    }

    builder
        .build()
        .map_err(|e| FetchError::NavigationFailed(format!("{e:?}")))
}

/// Apply TLS configuration to a reqwest ClientBuilder.
///
/// Three paths:
/// - Default (no config): Use Chrome fingerprint to avoid JA3/JA4 bot detection
/// - Simple: uses reqwest's built-in TLS methods (min/max version, insecure, CA certs)
/// - Advanced: builds a custom rustls::ClientConfig for cipher suite / ALPN / group control
fn apply_tls_config(
    builder: reqwest::ClientBuilder,
    tls_config: Option<&TlsConfig>,
) -> Result<reqwest::ClientBuilder, FetchError> {
    // Use Chrome fingerprint by default to defeat TLS fingerprinting (JA3/JA4).
    // Sites like stackoverflow.com block based on TLS fingerprint even when
    // HTTP headers are browser-realistic.
    let chrome_default = TlsConfig::chrome();
    let tls = match tls_config {
        Some(c) if !c.is_default() => c,
        _ => &chrome_default,
    };

    if tls.needs_custom_rustls() {
        // Advanced path: build rustls::ClientConfig directly
        let rustls_config = tls
            .build_rustls_config()
            .map_err(FetchError::NavigationFailed)?;
        Ok(builder.use_preconfigured_tls(rustls_config))
    } else {
        // Simple path: use reqwest's built-in TLS methods
        tls.apply_to_reqwest(builder)
            .map_err(FetchError::NavigationFailed)
    }
}

/// Fetch a URL and return the HTML content.
pub async fn fetch_url(
    client: &Client,
    url: &str,
    timeout_ms: u64,
) -> Result<FetchResult, FetchError> {
    fetch_url_inner(client, url, timeout_ms, None, FetchLimits::default()).await
}

/// Fetch with caller-owned resource limits and a policy that cannot be relaxed
/// by `PLASMATE_UNSAFE_ALLOW_PRIVATE_NETWORK`.
pub(crate) async fn fetch_url_public_only_with_limits(
    client: &PublicOnlyClient,
    url: &str,
    timeout_ms: u64,
    limits: FetchLimits,
) -> Result<FetchResult, FetchError> {
    fetch_url_inner_with_policy(
        &client.0,
        url,
        timeout_ms,
        None,
        limits,
        OutboundUrlPolicy::public_network_only(),
        false,
    )
    .await
}

/// Public-only fetch whose redirect chain is additionally pinned to the
/// original origin. Redirect destinations are revalidated before any request.
pub(crate) async fn fetch_url_public_only_same_origin_with_limits(
    client: &PublicOnlyClient,
    url: &str,
    timeout_ms: u64,
    limits: FetchLimits,
) -> Result<FetchResult, FetchError> {
    fetch_url_inner_with_policy(
        &client.0,
        url,
        timeout_ms,
        None,
        limits,
        OutboundUrlPolicy::public_network_only(),
        true,
    )
    .await
}

/// Fetch from a deterministic local fixture with an explicit private-network policy.
/// The ordinary `fetch_url` path remains fail-closed.
pub(crate) async fn fetch_url_for_local_fixture(
    client: &Client,
    url: &str,
    timeout_ms: u64,
) -> Result<FetchResult, FetchError> {
    fetch_url_inner_with_policy(
        client,
        url,
        timeout_ms,
        None,
        FetchLimits::default(),
        OutboundUrlPolicy::for_local_fixtures(),
        false,
    )
    .await
}

/// Test-only counterpart for deterministic loopback discovery fixtures.
#[cfg(test)]
pub(crate) async fn fetch_url_for_local_fixture_with_limits(
    client: &Client,
    url: &str,
    timeout_ms: u64,
    limits: FetchLimits,
) -> Result<FetchResult, FetchError> {
    fetch_url_inner_with_policy(
        client,
        url,
        timeout_ms,
        None,
        limits,
        OutboundUrlPolicy::for_local_fixtures(),
        false,
    )
    .await
}

/// Test-only same-origin counterpart for deterministic loopback fixtures.
#[cfg(test)]
pub(crate) async fn fetch_url_for_local_fixture_same_origin_with_limits(
    client: &Client,
    url: &str,
    timeout_ms: u64,
    limits: FetchLimits,
) -> Result<FetchResult, FetchError> {
    fetch_url_inner_with_policy(
        client,
        url,
        timeout_ms,
        None,
        limits,
        OutboundUrlPolicy::for_local_fixtures(),
        true,
    )
    .await
}

/// Fetch a URL with additional headers (for interception overrides).
pub async fn fetch_url_with_headers(
    client: &Client,
    url: &str,
    timeout_ms: u64,
    extra_headers: &std::collections::HashMap<String, String>,
) -> Result<FetchResult, FetchError> {
    fetch_url_inner(
        client,
        url,
        timeout_ms,
        Some(extra_headers),
        FetchLimits::default(),
    )
    .await
}

async fn fetch_url_inner(
    client: &Client,
    url: &str,
    timeout_ms: u64,
    extra_headers: Option<&std::collections::HashMap<String, String>>,
    limits: FetchLimits,
) -> Result<FetchResult, FetchError> {
    let policy = OutboundUrlPolicy::from_environment();
    fetch_url_inner_with_policy(
        client,
        url,
        timeout_ms,
        extra_headers,
        limits,
        policy,
        false,
    )
    .await
}

async fn fetch_url_inner_with_policy(
    client: &Client,
    url: &str,
    timeout_ms: u64,
    extra_headers: Option<&std::collections::HashMap<String, String>>,
    limits: FetchLimits,
    policy: OutboundUrlPolicy,
    same_origin_only: bool,
) -> Result<FetchResult, FetchError> {
    let start = Instant::now();
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
    let mut current = policy
        .validate_url(url)
        .await
        .map_err(FetchError::UrlBlocked)?;
    let initial_origin = origin_key(&current);

    for redirect_count in 0..=limits.max_redirects {
        let remaining = deadline
            .checked_duration_since(tokio::time::Instant::now())
            .ok_or(FetchError::Timeout(timeout_ms))?;
        let mut request = document_request(client, current.as_str());
        if let Some(headers) = extra_headers {
            let same_origin = origin_key(&current) == initial_origin;
            for (name, value) in headers {
                if !same_origin && is_sensitive_header(name) {
                    continue;
                }
                request = request.header(name.as_str(), value.as_str());
            }
        }

        let response = tokio::time::timeout(remaining, request.send())
            .await
            .map_err(|_| FetchError::Timeout(timeout_ms))?
            .map_err(|e| FetchError::NavigationFailed(format!("{e:?}")))?;

        if response.status().is_redirection() {
            if redirect_count == limits.max_redirects {
                return Err(FetchError::TooManyRedirects(limits.max_redirects));
            }
            let location = response
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|value| value.to_str().ok())
                .ok_or_else(|| {
                    FetchError::NavigationFailed(
                        "redirect response missing a valid Location".into(),
                    )
                })?;
            current = validated_redirect_target(
                &current,
                location,
                &initial_origin,
                same_origin_only,
                policy,
            )
            .await?;
            continue;
        }

        let remaining = deadline
            .checked_duration_since(tokio::time::Instant::now())
            .ok_or(FetchError::Timeout(timeout_ms))?;
        return tokio::time::timeout(remaining, response_to_result(response, start, limits))
            .await
            .map_err(|_| FetchError::Timeout(timeout_ms))?;
    }

    Err(FetchError::TooManyRedirects(limits.max_redirects))
}

async fn validated_redirect_target(
    current: &Url,
    location: &str,
    initial_origin: &(String, String, Option<u16>),
    same_origin_only: bool,
    policy: OutboundUrlPolicy,
) -> Result<Url, FetchError> {
    let next = current
        .join(location)
        .map_err(|e| FetchError::NavigationFailed(format!("invalid redirect Location: {e}")))?;
    if same_origin_only && origin_key(&next) != *initial_origin {
        return Err(FetchError::UrlBlocked(
            "cross-origin redirect is not allowed for this operation".to_string(),
        ));
    }
    policy
        .validate_url(next.as_str())
        .await
        .map_err(FetchError::UrlBlocked)
}

fn document_request(client: &Client, url: &str) -> reqwest::RequestBuilder {
    client
        .get(url)
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
        .header("sec-ch-ua", "\"Chromium\";v=\"128\", \"Not;A=Brand\";v=\"24\"")
        .header("sec-ch-ua-mobile", "?0")
        .header("sec-ch-ua-platform", "\"macOS\"")
}

async fn response_to_result(
    response: reqwest::Response,
    start: Instant,
    limits: FetchLimits,
) -> Result<FetchResult, FetchError> {
    let status = response.status().as_u16();
    let final_url = response.url().to_string();
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("text/html")
        .to_string();
    let set_cookies = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|value| value.to_str().ok())
        .map(str::to_string)
        .collect();
    let content_encoding = response
        .headers()
        .get(reqwest::header::CONTENT_ENCODING)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);

    if status >= 400 {
        return Err(FetchError::HttpError {
            status,
            url: final_url,
        });
    }
    if response
        .content_length()
        .is_some_and(|length| length > limits.max_compressed_bytes as u64)
    {
        return Err(FetchError::BodyTooLarge {
            limit: limits.max_compressed_bytes,
        });
    }

    let mut wire_body = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk =
            chunk.map_err(|e| FetchError::NavigationFailed(format!("Body decode error: {e:?}")))?;
        let next_len = wire_body.len().saturating_add(chunk.len());
        if next_len > limits.max_compressed_bytes {
            return Err(FetchError::BodyTooLarge {
                limit: limits.max_compressed_bytes,
            });
        }
        wire_body.extend_from_slice(&chunk);
    }
    let body =
        decode_limited_body_async(wire_body, content_encoding, limits.max_body_bytes).await?;
    let html = String::from_utf8_lossy(&body).into_owned();
    Ok(FetchResult {
        url: final_url,
        status,
        content_type,
        html_bytes: html.len(),
        html,
        load_ms: start.elapsed().as_millis() as u64,
        set_cookies,
    })
}

pub(crate) fn decode_limited_body(
    wire_body: &[u8],
    content_encoding: Option<&str>,
    max_decoded_bytes: usize,
) -> Result<Vec<u8>, FetchError> {
    use std::io::{Cursor, Read};

    let encoding = content_encoding
        .unwrap_or("identity")
        .trim()
        .to_ascii_lowercase();
    let reader: Box<dyn Read> = match encoding.as_str() {
        "" | "identity" => Box::new(Cursor::new(wire_body)),
        "gzip" | "x-gzip" => Box::new(flate2::read::GzDecoder::new(Cursor::new(wire_body))),
        "deflate" => Box::new(flate2::read::DeflateDecoder::new(Cursor::new(wire_body))),
        "br" => Box::new(brotli::Decompressor::new(Cursor::new(wire_body), 4096)),
        other => {
            return Err(FetchError::NavigationFailed(format!(
                "unsupported Content-Encoding '{other}'"
            )))
        }
    };
    let mut decoded = Vec::new();
    reader
        .take(max_decoded_bytes.saturating_add(1) as u64)
        .read_to_end(&mut decoded)
        .map_err(|e| FetchError::NavigationFailed(format!("Body decode error: {e}")))?;
    if decoded.len() > max_decoded_bytes {
        return Err(FetchError::BodyTooLarge {
            limit: max_decoded_bytes,
        });
    }
    Ok(decoded)
}

/// Decode a bounded response without running attacker-controlled compression
/// work on a Tokio worker. Ownership is moved into the blocking task to avoid
/// duplicating the already-bounded wire buffer.
pub(crate) async fn decode_limited_body_async(
    wire_body: Vec<u8>,
    content_encoding: Option<String>,
    max_decoded_bytes: usize,
) -> Result<Vec<u8>, FetchError> {
    tokio::task::spawn_blocking(move || {
        decode_limited_body(&wire_body, content_encoding.as_deref(), max_decoded_bytes)
    })
    .await
    .map_err(|error| FetchError::NavigationFailed(format!("Body decode worker failed: {error}")))?
}

fn origin_key(url: &Url) -> (String, String, Option<u16>) {
    (
        url.scheme().to_string(),
        url.host_str().unwrap_or_default().to_ascii_lowercase(),
        url.port_or_known_default(),
    )
}

fn is_sensitive_header(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "authorization" | "proxy-authorization" | "cookie"
    )
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

#[cfg(test)]
mod security_tests {
    use super::*;
    use std::io::Write;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn fixture_server(response: &'static [u8]) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0u8; 1024];
            let _ = stream.read(&mut request).await;
            stream.write_all(response).await.unwrap();
        });
        format!("http://{address}/")
    }

    async fn owned_fixture_server(response: Vec<u8>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0u8; 1024];
            let _ = stream.read(&mut request).await;
            stream.write_all(&response).await.unwrap();
        });
        format!("http://{address}/")
    }

    async fn redirect_loop_fixture_server(requests: usize) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            for _ in 0..requests {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut request = [0u8; 1024];
                let _ = stream.read(&mut request).await;
                stream
                    .write_all(
                        b"HTTP/1.1 302 Found\r\nLocation: /loop\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    )
                    .await
                    .unwrap();
            }
        });
        format!("http://{address}/loop")
    }

    fn fixture_client() -> Client {
        Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn ordinary_fetch_policy_still_rejects_loopback_fixtures() {
        let url = fixture_server(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 2\r\n\r\nok",
        )
        .await;
        let result = fetch_url_inner_with_policy(
            &fixture_client(),
            &url,
            1_000,
            None,
            FetchLimits::default(),
            OutboundUrlPolicy::deny_private_network(),
            false,
        )
        .await;
        assert!(matches!(result, Err(FetchError::UrlBlocked(_))));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn public_only_client_ignores_private_network_environment_opt_in() {
        let previous = std::env::var_os(super::super::security::UNSAFE_PRIVATE_NETWORK_ENV);
        std::env::set_var(super::super::security::UNSAFE_PRIVATE_NETWORK_ENV, "1");
        let client = build_client_public_only(Arc::new(Jar::default())).unwrap();
        let url = fixture_server(
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}",
        )
        .await;
        let result =
            fetch_url_public_only_with_limits(&client, &url, 1_000, FetchLimits::default()).await;
        if let Some(value) = previous {
            std::env::set_var(super::super::security::UNSAFE_PRIVATE_NETWORK_ENV, value);
        } else {
            std::env::remove_var(super::super::security::UNSAFE_PRIVATE_NETWORK_ENV);
        }
        assert!(matches!(result, Err(FetchError::UrlBlocked(_))));
    }

    #[tokio::test]
    async fn redirect_target_is_revalidated() {
        let url = fixture_server(
            b"HTTP/1.1 302 Found\r\nLocation: file:///etc/passwd\r\nContent-Length: 0\r\n\r\n",
        )
        .await;
        let result = fetch_url_inner_with_policy(
            &fixture_client(),
            &url,
            1000,
            None,
            FetchLimits::default(),
            OutboundUrlPolicy::for_test_fixtures(),
            false,
        )
        .await;
        assert!(matches!(result, Err(FetchError::UrlBlocked(_))));
    }

    #[tokio::test]
    async fn public_redirect_to_private_destination_is_blocked_before_request() {
        let current = Url::parse("https://public.example/start").unwrap();
        let initial_origin = origin_key(&current);
        let result = validated_redirect_target(
            &current,
            "http://127.0.0.1/private",
            &initial_origin,
            false,
            OutboundUrlPolicy::deny_private_network(),
        )
        .await;
        assert!(matches!(result, Err(FetchError::UrlBlocked(_))));
    }

    #[tokio::test]
    async fn redirect_loop_stops_at_configured_count() {
        let limits = FetchLimits {
            max_compressed_bytes: 100,
            max_body_bytes: 100,
            max_redirects: 2,
        };
        let url = redirect_loop_fixture_server(limits.max_redirects + 1).await;
        let result = fetch_url_inner_with_policy(
            &fixture_client(),
            &url,
            1_000,
            None,
            limits,
            OutboundUrlPolicy::for_test_fixtures(),
            false,
        )
        .await;
        assert!(matches!(result, Err(FetchError::TooManyRedirects(2))));
    }

    #[tokio::test]
    async fn same_origin_mode_rejects_cross_origin_redirect_before_following() {
        let destination = fixture_server(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\n\r\nok",
        )
        .await;
        let response =
            format!("HTTP/1.1 302 Found\r\nLocation: {destination}\r\nContent-Length: 0\r\n\r\n")
                .into_bytes();
        let source = owned_fixture_server(response).await;
        let result = fetch_url_inner_with_policy(
            &fixture_client(),
            &source,
            1_000,
            None,
            FetchLimits::default(),
            OutboundUrlPolicy::for_test_fixtures(),
            true,
        )
        .await;
        assert!(
            matches!(result, Err(FetchError::UrlBlocked(message)) if message.contains("cross-origin"))
        );
    }

    #[tokio::test]
    async fn decoded_body_stream_stops_at_hard_limit() {
        let url = fixture_server(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 10\r\n\r\n0123456789",
        )
        .await;
        let result = fetch_url_inner_with_policy(
            &fixture_client(),
            &url,
            1000,
            None,
            FetchLimits {
                max_compressed_bytes: 100,
                max_body_bytes: 4,
                max_redirects: 1,
            },
            OutboundUrlPolicy::for_test_fixtures(),
            false,
        )
        .await;
        assert!(matches!(result, Err(FetchError::BodyTooLarge { limit: 4 })));
    }

    #[tokio::test]
    async fn declared_wire_size_is_rejected_before_reading() {
        let url = fixture_server(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 10\r\n\r\n0123456789",
        )
        .await;
        let result = fetch_url_inner_with_policy(
            &fixture_client(),
            &url,
            1000,
            None,
            FetchLimits {
                max_compressed_bytes: 4,
                max_body_bytes: 100,
                max_redirects: 1,
            },
            OutboundUrlPolicy::for_test_fixtures(),
            false,
        )
        .await;
        assert!(matches!(result, Err(FetchError::BodyTooLarge { limit: 4 })));
    }

    #[tokio::test]
    async fn oversized_chunked_wire_body_stops_at_hard_limit() {
        let url = fixture_server(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nTransfer-Encoding: chunked\r\n\r\n4\r\n0123\r\n4\r\n4567\r\n0\r\n\r\n",
        )
        .await;
        let result = fetch_url_inner_with_policy(
            &fixture_client(),
            &url,
            1_000,
            None,
            FetchLimits {
                max_compressed_bytes: 6,
                max_body_bytes: 100,
                max_redirects: 1,
            },
            OutboundUrlPolicy::for_test_fixtures(),
            false,
        )
        .await;
        assert!(matches!(result, Err(FetchError::BodyTooLarge { limit: 6 })));
    }

    #[test]
    fn compressed_body_cannot_expand_past_decoded_limit() {
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        encoder.write_all(&vec![b'a'; 4096]).unwrap();
        let wire = encoder.finish().unwrap();
        assert!(wire.len() < 1024);
        let result = decode_limited_body(&wire, Some("gzip"), 1024);
        assert!(matches!(
            result,
            Err(FetchError::BodyTooLarge { limit: 1024 })
        ));
    }

    #[tokio::test]
    async fn async_decoder_preserves_decoded_limit() {
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        encoder.write_all(&vec![b'a'; 4096]).unwrap();
        let wire = encoder.finish().unwrap();
        let result = decode_limited_body_async(wire, Some("gzip".to_string()), 1024).await;
        assert!(matches!(
            result,
            Err(FetchError::BodyTooLarge { limit: 1024 })
        ));
    }
}
