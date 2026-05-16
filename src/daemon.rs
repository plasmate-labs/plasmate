//! Plasmate daemon: keeps a warm HTTP client and JS runtime for fast repeated fetches.
//!
//! The daemon listens on a local TCP port (default 9224) and accepts JSON requests:
//!   POST /fetch  { "url": "https://...", "no_js": false, "profile": null }
//!   GET  /health
//!   POST /shutdown
//!
//! The `plasmate fetch` command auto-connects to the daemon when it is running,
//! avoiding cold-start overhead on every invocation.

use crate::auth;
use crate::cache::store::{CacheConfig, CacheLookup, CacheSnapshot, SomCache};
use crate::js;
use crate::network;
use crate::som;
use crate::som::filter::apply_selector;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tracing::{debug, info};

const DEFAULT_PORT: u16 = 9224;

#[derive(Deserialize, Serialize)]
struct FetchRequest {
    url: String,
    #[serde(default)]
    no_js: bool,
    #[serde(default)]
    no_external: bool,
    #[serde(default)]
    selector: Option<String>,
    profile: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct FetchResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    som: Option<som::types::Som>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    fetch_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    cache_status: Option<String>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    uptime_seconds: u64,
    requests_served: u64,
    cache: CacheSnapshot,
}

/// Start the daemon server.
pub async fn run_daemon(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let port = if port == 0 { DEFAULT_PORT } else { port };
    let addr = format!("127.0.0.1:{}", port);

    // Build a persistent HTTP client (warm, reusable connections)
    let jar = Arc::new(reqwest::cookie::Jar::default());
    let tls_config = network::tls::global();
    let client = network::fetch::build_client_h1_fallback(None, jar.clone(), tls_config)?;
    let cache = Arc::new(SomCache::new(CacheConfig::default()));

    let listener = TcpListener::bind(&addr).await?;
    eprintln!("Plasmate daemon listening on {}", addr);
    eprintln!("Stop with: plasmate daemon stop");

    // Write PID file so `plasmate fetch` can find us
    let pid_path = daemon_pid_path();
    std::fs::create_dir_all(pid_path.parent().unwrap())?;
    std::fs::write(&pid_path, format!("{}\n{}", std::process::id(), port))?;

    let start = std::time::Instant::now();
    let request_count = Arc::new(std::sync::atomic::AtomicU64::new(0));

    loop {
        let (stream, _) = listener.accept().await?;
        let client = client.clone();
        let jar = jar.clone();
        let cache = cache.clone();
        let count = request_count.clone();
        let start_time = start;

        tokio::spawn(async move {
            if let Err(e) =
                handle_connection(stream, &client, &jar, &cache, &count, start_time).await
            {
                eprintln!("Connection error: {}", e);
            }
        });
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    client: &reqwest::Client,
    _jar: &Arc<reqwest::cookie::Jar>,
    cache: &Arc<SomCache>,
    request_count: &Arc<std::sync::atomic::AtomicU64>,
    start_time: std::time::Instant,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (reader, mut writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);

    // Read HTTP request line
    let mut request_line = String::new();
    buf_reader.read_line(&mut request_line).await?;

    // Read headers (skip them, we just need Content-Length)
    let mut content_length: usize = 0;
    loop {
        let mut header = String::new();
        buf_reader.read_line(&mut header).await?;
        if header.trim().is_empty() {
            break;
        }
        if let Some(val) = header
            .strip_prefix("Content-Length: ")
            .or_else(|| header.strip_prefix("content-length: "))
        {
            content_length = val.trim().parse().unwrap_or(0);
        }
    }

    // Read body
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        tokio::io::AsyncReadExt::read_exact(&mut buf_reader, &mut body).await?;
    }

    let (status, response_body) = if request_line.starts_with("POST /fetch") {
        request_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        handle_fetch(client, cache, &body).await
    } else if request_line.starts_with("GET /health") {
        let uptime = start_time.elapsed().as_secs();
        let count = request_count.load(std::sync::atomic::Ordering::Relaxed);
        let resp = HealthResponse {
            status: "ok".to_string(),
            uptime_seconds: uptime,
            requests_served: count,
            cache: cache.snapshot(),
        };
        ("200 OK".to_string(), serde_json::to_string(&resp).unwrap())
    } else if request_line.starts_with("POST /shutdown") {
        // Clean up PID file and exit
        let _ = std::fs::remove_file(daemon_pid_path());
        let resp = serde_json::json!({"status": "shutting_down"});
        let body = serde_json::to_string(&resp).unwrap();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(), body
        );
        writer.write_all(response.as_bytes()).await?;
        writer.flush().await?;
        // Give the response time to flush, then exit
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        std::process::exit(0);
    } else {
        (
            "404 Not Found".to_string(),
            r#"{"error":"not found"}"#.to_string(),
        )
    };

    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        response_body.len(),
        response_body
    );
    writer.write_all(response.as_bytes()).await?;
    writer.flush().await?;

    Ok(())
}

async fn handle_fetch(client: &reqwest::Client, cache: &SomCache, body: &[u8]) -> (String, String) {
    let req: FetchRequest = match serde_json::from_slice(body) {
        Ok(r) => r,
        Err(e) => {
            let resp = FetchResponse {
                success: false,
                som: None,
                error: Some(format!("Invalid request: {}", e)),
                fetch_ms: 0,
                cache_status: None,
            };
            return (
                "400 Bad Request".to_string(),
                serde_json::to_string(&resp).unwrap(),
            );
        }
    };

    let start = std::time::Instant::now();

    // Load auth profile if specified
    if let Some(ref domain) = req.profile {
        let jar = Arc::new(reqwest::cookie::Jar::default());
        let _ = auth::store::load_into_jar(domain, &jar);
    }

    // Fetch the page
    let result = match network::fetch::fetch_url(client, &req.url, 30000).await {
        Ok(r) => r,
        Err(e) => {
            let resp = FetchResponse {
                success: false,
                som: None,
                error: Some(format!("Fetch failed: {}", e)),
                fetch_ms: start.elapsed().as_millis() as u64,
                cache_status: None,
            };
            return (
                "502 Bad Gateway".to_string(),
                serde_json::to_string(&resp).unwrap(),
            );
        }
    };
    let content_hash = SomCache::content_hash(result.html.as_bytes());
    match cache.lookup_or_filter_selector(&result.url, content_hash, req.selector.as_deref()) {
        CacheLookup::Hit(entry) => {
            if let Ok(som) = serde_json::from_slice::<som::types::Som>(&entry.som_json) {
                debug!(url = %result.url, selector = ?req.selector, "Daemon SOM cache hit");
                let resp = FetchResponse {
                    success: true,
                    som: Some(som),
                    error: None,
                    fetch_ms: start.elapsed().as_millis() as u64,
                    cache_status: Some("hit".to_string()),
                };
                return ("200 OK".to_string(), serde_json::to_string(&resp).unwrap());
            }
        }
        CacheLookup::Stale { .. } | CacheLookup::Miss => {}
    }

    // Process through JS pipeline
    let pipeline_config = js::pipeline::PipelineConfig {
        execute_js: !req.no_js,
        fetch_external_scripts: !req.no_external && !req.no_js,
        ..Default::default()
    };

    let page_result =
        match js::pipeline::process_page_async(&result.html, &result.url, &pipeline_config, client)
            .await
        {
            Ok(r) => r,
            Err(e) => {
                // Graceful degradation: compile without JS
                info!(error = %e, "JS pipeline failed, compiling without JS");
                match som::compiler::compile(&result.html, &result.url) {
                    Ok(som) => {
                        let selected_som = select_and_store_som(
                            cache,
                            &result.url,
                            content_hash,
                            som,
                            result.html_bytes,
                            req.selector.as_deref(),
                        );
                        let resp = FetchResponse {
                            success: true,
                            som: Some(selected_som),
                            error: Some(
                                "JS execution failed, compiled from static HTML".to_string(),
                            ),
                            fetch_ms: start.elapsed().as_millis() as u64,
                            cache_status: None,
                        };
                        return ("200 OK".to_string(), serde_json::to_string(&resp).unwrap());
                    }
                    Err(e2) => {
                        let resp = FetchResponse {
                            success: false,
                            som: None,
                            error: Some(format!(
                                "Both JS and static compilation failed: {}, {}",
                                e, e2
                            )),
                            fetch_ms: start.elapsed().as_millis() as u64,
                            cache_status: None,
                        };
                        return (
                            "500 Internal Server Error".to_string(),
                            serde_json::to_string(&resp).unwrap(),
                        );
                    }
                }
            }
        };

    let som = select_and_store_som(
        cache,
        &result.url,
        content_hash,
        page_result.som,
        result.html_bytes,
        req.selector.as_deref(),
    );

    let resp = FetchResponse {
        success: true,
        som: Some(som),
        error: None,
        fetch_ms: start.elapsed().as_millis() as u64,
        cache_status: Some("miss".to_string()),
    };

    ("200 OK".to_string(), serde_json::to_string(&resp).unwrap())
}

fn select_and_store_som(
    cache: &SomCache,
    url: &str,
    content_hash: u64,
    som: som::types::Som,
    html_bytes: usize,
    selector: Option<&str>,
) -> som::types::Som {
    if let Ok(full_som_json) = serde_json::to_vec(&som) {
        cache.store(url, content_hash, full_som_json, html_bytes);
    }

    if let Some(selector) = selector {
        let selected = apply_selector(&som, selector);
        if let Ok(selected_json) = serde_json::to_vec(&selected) {
            cache.store_with_selector(url, content_hash, Some(selector), selected_json, html_bytes);
        }
        selected
    } else {
        som
    }
}

/// Path to the daemon PID file.
pub fn daemon_pid_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    std::path::PathBuf::from(home)
        .join(".plasmate")
        .join("daemon.pid")
}

/// Check if the daemon is running and return its port.
pub fn daemon_port() -> Option<u16> {
    let pid_path = daemon_pid_path();
    let content = std::fs::read_to_string(&pid_path).ok()?;
    let mut lines = content.lines();
    let pid: u32 = lines.next()?.trim().parse().ok()?;
    let port: u16 = lines.next()?.trim().parse().ok()?;

    // Check if process is still alive by checking /proc or sending signal 0
    let alive = std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !alive {
        // Process is dead, clean up stale PID file
        let _ = std::fs::remove_file(&pid_path);
        return None;
    }

    Some(port)
}

/// Send a fetch request to the running daemon.
pub async fn daemon_fetch(
    port: u16,
    url: &str,
    no_js: bool,
    profile: Option<&str>,
    selector: Option<&str>,
) -> Result<som::types::Som, Box<dyn std::error::Error>> {
    let req = FetchRequest {
        url: url.to_string(),
        no_js,
        no_external: false,
        selector: selector.map(|s| s.to_string()),
        profile: profile.map(|s| s.to_string()),
    };
    let body = serde_json::to_string(&req)?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(35))
        .build()?;

    let resp = client
        .post(format!("http://127.0.0.1:{}/fetch", port))
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await?;

    let resp_text = resp.text().await?;
    let resp_body: FetchResponse = serde_json::from_str(&resp_text)?;

    if resp_body.success {
        resp_body.som.ok_or_else(|| "No SOM in response".into())
    } else {
        Err(resp_body
            .error
            .unwrap_or_else(|| "Unknown error".to_string())
            .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::som::types::{Element, ElementRole, Region, RegionRole, Som, SomMeta};

    fn test_som() -> Som {
        Som {
            som_version: "0.1".to_string(),
            url: "https://example.com/app".to_string(),
            title: "App".to_string(),
            lang: "en".to_string(),
            regions: vec![Region {
                id: "r1".to_string(),
                role: RegionRole::Main,
                label: None,
                action: None,
                method: None,
                target: None,
                enctype: None,
                novalidate: None,
                accept_charset: None,
                autocomplete: None,
                elements: vec![
                    Element {
                        id: "e1".to_string(),
                        role: ElementRole::Button,
                        html_id: Some("save".to_string()),
                        text: Some("Save".to_string()),
                        label: None,
                        actions: Some(vec!["click".to_string()]),
                        attrs: None,
                        children: None,
                        hints: None,
                        shadow: None,
                    },
                    Element {
                        id: "e2".to_string(),
                        role: ElementRole::Paragraph,
                        html_id: None,
                        text: Some("Body".to_string()),
                        label: None,
                        actions: None,
                        attrs: None,
                        children: None,
                        hints: None,
                        shadow: None,
                    },
                ],
            }],
            meta: SomMeta {
                html_bytes: 200,
                som_bytes: 100,
                element_count: 2,
                interactive_count: 1,
            },
            structured_data: None,
        }
    }

    #[test]
    fn fetch_request_round_trips_selector() {
        let req = FetchRequest {
            url: "https://example.com".to_string(),
            no_js: true,
            no_external: true,
            selector: Some("action:click".to_string()),
            profile: Some("example.com".to_string()),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"selector\":\"action:click\""));

        let parsed: FetchRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.selector.as_deref(), Some("action:click"));
        assert!(parsed.no_js);
        assert!(parsed.no_external);
    }

    #[test]
    fn select_and_store_som_materializes_selector_cache() {
        let cache = SomCache::new(CacheConfig::default());
        let content_hash = 42;

        let selected = select_and_store_som(
            &cache,
            "https://example.com/app",
            content_hash,
            test_som(),
            200,
            Some("ACTION:CLICK"),
        );

        assert_eq!(selected.regions.len(), 1);
        assert_eq!(selected.regions[0].elements.len(), 1);
        assert_eq!(selected.regions[0].elements[0].id, "e1");
        assert!(matches!(
            cache.lookup("https://example.com/app", content_hash),
            CacheLookup::Hit(_)
        ));
        assert!(matches!(
            cache.lookup_with_selector(
                "https://example.com/app",
                content_hash,
                Some("action:click")
            ),
            CacheLookup::Hit(_)
        ));
    }

    #[test]
    fn fetch_response_accepts_cache_status() {
        let response = serde_json::json!({
            "success": true,
            "som": test_som(),
            "fetch_ms": 3,
            "cache_status": "hit"
        });

        let parsed: FetchResponse = serde_json::from_value(response).unwrap();
        assert_eq!(parsed.cache_status.as_deref(), Some("hit"));
        assert!(parsed.success);
    }

    #[test]
    fn health_response_includes_cache_snapshot() {
        let cache = SomCache::new(CacheConfig::default());
        cache.store("https://example.com/app", 42, b"som".to_vec(), 200);
        cache.store_with_selector(
            "https://example.com/app",
            42,
            Some("interactive"),
            b"sel".to_vec(),
            200,
        );
        let _ = cache.lookup_with_selector("https://example.com/app", 42, Some("interactive"));

        let response = HealthResponse {
            status: "ok".to_string(),
            uptime_seconds: 5,
            requests_served: 2,
            cache: cache.snapshot(),
        };
        let json = serde_json::to_value(response).unwrap();

        assert_eq!(json["cache"]["entries"], 2);
        assert_eq!(json["cache"]["full_entries"], 1);
        assert_eq!(json["cache"]["selector_entries"], 1);
        assert_eq!(json["cache"]["hits"], 1);
    }
}
