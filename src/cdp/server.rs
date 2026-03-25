//! CDP WebSocket server.
//!
//! Implements the CDP discovery endpoint (HTTP GET /json/version and /json/list)
//! plus the WebSocket connection for CDP messages. This is what Puppeteer and
//! Playwright connect to.

use futures_util::{sink::SinkExt, StreamExt};
use std::time::Duration;
use tokio::net::TcpListener;
use tracing::{debug, error, info, warn};

use super::handler::handle_cdp_request;
use super::session::{CdpTarget, SharedPlugins};
use super::types::CdpRequest;

/// Start the CDP-compatible server.
///
/// This serves both the HTTP discovery endpoints and WebSocket connections.
pub async fn start(
    host: &str,
    port: u16,
    plugins: SharedPlugins,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr).await?;
    info!("CDP server listening on {}", addr);
    info!("  WebSocket: ws://{}:{}", host, port);
    info!("  Discovery: http://{}:{}/json/version", host, port);

    loop {
        let (stream, peer) = listener.accept().await?;
        let listen_port = port;
        let listen_host = host.to_string();
        let plugins = plugins.clone();
        info!(%peer, "New connection");

        tokio::spawn(async move {
            // Peek at the first bytes to determine if this is HTTP or WebSocket
            let mut buf = [0u8; 4];
            match stream.peek(&mut buf).await {
                Ok(n) if n >= 3 => {
                    let start = String::from_utf8_lossy(&buf[..n]);
                    if start.starts_with("GET") {
                        handle_http_or_upgrade(stream, peer, &listen_host, listen_port, plugins)
                            .await;
                    } else {
                        // Direct WebSocket (unlikely but handle it)
                        handle_websocket_connection(stream, peer, plugins).await;
                    }
                }
                _ => {
                    error!(%peer, "Failed to peek connection");
                }
            }
        });
    }
}

/// Handle HTTP requests (discovery endpoints) or upgrade to WebSocket.
async fn handle_http_or_upgrade(
    stream: tokio::net::TcpStream,
    peer: std::net::SocketAddr,
    listen_host: &str,
    listen_port: u16,
    plugins: SharedPlugins,
) {
    // Read the HTTP request line
    let mut buf = vec![0u8; 4096];
    let n = match stream.peek(&mut buf).await {
        Ok(n) => n,
        Err(e) => {
            error!(%peer, "Failed to read: {}", e);
            return;
        }
    };

    let request = String::from_utf8_lossy(&buf[..n]);
    let first_line = request.lines().next().unwrap_or("");

    // Check if this is a WebSocket upgrade
    let is_upgrade = request.to_lowercase().contains("upgrade: websocket");

    if is_upgrade {
        handle_websocket_connection(stream, peer, plugins).await;
        return;
    }

    // HTTP discovery endpoints
    let (path, _) = first_line.split_once(" HTTP").unwrap_or((first_line, ""));
    let path = path.trim_start_matches("GET ");

    let advertise_host = if listen_host == "0.0.0.0" || listen_host == "::" {
        "127.0.0.1"
    } else {
        listen_host
    };

    let response_body = match path {
        "/json/version" | "/json/version/" => {
            serde_json::json!({
                "Browser": "Plasmate/0.1.0",
                "Protocol-Version": "1.3",
                "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36",
                "V8-Version": "12.0",
                "WebKit-Version": "537.36",
                "webSocketDebuggerUrl": format!("ws://{}:{}/devtools/browser/plasmate", advertise_host, listen_port),
            })
        }
        "/json" | "/json/" | "/json/list" | "/json/list/" => {
            serde_json::json!([{
                "description": "Plasmate CDP compatibility target",
                "devtoolsFrontendUrl": "",
                "id": "plasmate-default",
                "title": "Plasmate",
                "type": "page",
                "url": "about:blank",
                "webSocketDebuggerUrl": format!("ws://{}:{}/devtools/page/plasmate-default", advertise_host, listen_port),
            }])
        }
        "/json/protocol" | "/json/protocol/" => {
            // Minimal protocol descriptor
            serde_json::json!({
                "domains": [
                    {"domain": "Browser", "description": "Browser domain"},
                    {"domain": "Page", "description": "Page domain"},
                    {"domain": "Runtime", "description": "Runtime domain"},
                    {"domain": "DOM", "description": "DOM domain"},
                    {"domain": "Network", "description": "Network domain"},
                    {"domain": "Input", "description": "Input domain"},
                    {"domain": "Plasmate", "description": "Plasmate SOM-native domain"},
                ]
            })
        }
        _ => {
            // 404
            let body = "Not Found";
            let resp = format!(
                "HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{}",
                body.len(), body
            );
            let _ = stream.try_write(resp.as_bytes());
            return;
        }
    };

    let body = serde_json::to_string_pretty(&response_body).unwrap_or_default();
    let resp = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.try_write(resp.as_bytes());
}

/// Handle a WebSocket connection with CDP messages.
async fn handle_websocket_connection(
    stream: tokio::net::TcpStream,
    peer: std::net::SocketAddr,
    plugins: SharedPlugins,
) {
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!(%peer, "WebSocket handshake failed: {}", e);
            return;
        }
    };

    info!(%peer, "CDP WebSocket connected");

    let (mut sink, mut stream) = ws_stream.split();

    // Default target (always present)
    let mut default_target = match CdpTarget::new_with_plugins(plugins.clone()) {
        Ok(t) => t,
        Err(e) => {
            error!(%peer, "Failed to create target: {}", e);
            return;
        }
    };

    // Additional targets keyed by session_id (for multi-target support)
    let mut extra_targets: std::collections::HashMap<String, CdpTarget> =
        std::collections::HashMap::new();

    while let Some(msg) = stream.next().await {
        match msg {
            Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                let req: CdpRequest = match serde_json::from_str(&text) {
                    Ok(r) => r,
                    Err(e) => {
                        warn!(%peer, "Invalid CDP JSON: {}", e);
                        continue;
                    }
                };

                let can_create = extra_targets.len() < 49;

                // Intercept Target.attachToTarget before mutable routing
                // (needs immutable access to all targets to find by targetId)
                if req.method == "Target.attachToTarget" {
                    let attach_target_id = req.params.get("targetId").and_then(|v| v.as_str()).unwrap_or("");
                    // Find the session_id for the target that matches this targetId
                    let found = if default_target.target_id == attach_target_id {
                        Some((default_target.session_id.clone(), default_target.target_id.clone(), default_target.current_url.as_deref().unwrap_or("about:blank").to_string()))
                    } else {
                        extra_targets.values().find(|t| t.target_id == attach_target_id).map(|t| {
                            (t.session_id.clone(), t.target_id.clone(), t.current_url.as_deref().unwrap_or("about:blank").to_string())
                        })
                    };
                    if let Some((sid, tid, url)) = found {
                        let events = vec![
                            super::types::CdpEvent::new(
                                "Target.attachedToTarget",
                                serde_json::json!({
                                    "sessionId": sid,
                                    "targetInfo": {
                                        "targetId": tid,
                                        "type": "page",
                                        "title": "",
                                        "url": url,
                                        "attached": true,
                                        "browserContextId": "default",
                                    },
                                    "waitingForDebugger": false,
                                }),
                            ),
                        ];
                        let response = super::types::CdpResponse::success(
                            req.id,
                            serde_json::json!({"sessionId": sid}),
                        );
                        let response_json = serde_json::to_string(&response).unwrap_or_default();
                        for event in events {
                            let event_json = serde_json::to_string(&event).unwrap_or_default();
                            if let Err(e) = sink.send(tokio_tungstenite::tungstenite::Message::Text(event_json)).await {
                                error!(%peer, "Failed to send event: {}", e);
                                return;
                            }
                        }
                        if let Err(e) = sink.send(tokio_tungstenite::tungstenite::Message::Text(response_json)).await {
                            error!(%peer, "Failed to send response: {}", e);
                            return;
                        }
                        continue;
                    }
                    // If not found, fall through to normal handler
                }

                // Route by sessionId to the correct target
                let target = if let Some(ref sid) = req.session_id {
                    if default_target.session_ids.contains(sid) {
                        &mut default_target
                    } else if let Some(t) = extra_targets.get_mut(sid) {
                        t
                    } else {
                        &mut default_target // fallback
                    }
                } else {
                    &mut default_target
                };

                let (response, events) = if req.method == "Target.createTarget" && can_create {
                    match CdpTarget::new_with_plugins(plugins.clone()) {
                        Ok(new_target) => {
                            let tid = new_target.target_id.clone();
                            let sid = new_target.session_id.clone();
                            extra_targets.insert(sid.clone(), new_target);
                            let events = vec![
                                super::types::CdpEvent::new(
                                    "Target.targetCreated",
                                    serde_json::json!({
                                        "targetInfo": {"targetId": tid, "type": "page", "title": "", "url": "about:blank", "attached": true, "browserContextId": "default"}
                                    }),
                                ),
                                super::types::CdpEvent::new(
                                    "Target.attachedToTarget",
                                    serde_json::json!({
                                        "sessionId": sid, "targetInfo": {"targetId": tid, "type": "page", "title": "", "url": "about:blank", "attached": true, "browserContextId": "default"}, "waitingForDebugger": false
                                    }),
                                ),
                            ];
                            (
                                super::types::CdpResponse::success(
                                    req.id,
                                    serde_json::json!({"targetId": tid}),
                                ),
                                events,
                            )
                        }
                        Err(e) => (super::types::CdpResponse::error(req.id, -32000, &e), vec![]),
                    }
                } else {
                    handle_cdp_request(&req, target).await
                };

                // Important: Puppeteer waits for the Page.navigate response (loaderId)
                // before it starts matching lifecycle events. So for navigation,
                // send the response first, then the events.
                let send_response_first =
                    req.method == "Page.navigate" || req.method == "Target.createTarget";

                let response_json = serde_json::to_string(&response).unwrap_or_default();

                if send_response_first {
                    if let Err(e) = sink
                        .send(tokio_tungstenite::tungstenite::Message::Text(
                            response_json.clone(),
                        ))
                        .await
                    {
                        error!(%peer, "Failed to send response: {}", e);
                        return;
                    }

                    // Flush the response to the network before sending events.
                    // Puppeteer must process Page.navigate's loaderId before
                    // lifecycle events arrive - they're matched by loaderId.
                    if req.method == "Page.navigate" {
                        let _ = sink.flush().await;
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }
                }

                for event in events {
                    let event_json = serde_json::to_string(&event).unwrap_or_default();
                    debug!(%peer, method = %event.method, "Sending CDP event");
                    if let Err(e) = sink
                        .send(tokio_tungstenite::tungstenite::Message::Text(event_json))
                        .await
                    {
                        error!(%peer, "Failed to send event: {}", e);
                        return;
                    }
                }

                if !send_response_first {
                    if let Err(e) = sink
                        .send(tokio_tungstenite::tungstenite::Message::Text(response_json))
                        .await
                    {
                        error!(%peer, "Failed to send response: {}", e);
                        return;
                    }
                }
            }
            Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => {
                info!(%peer, "CDP connection closed");
                break;
            }
            Ok(tokio_tungstenite::tungstenite::Message::Ping(data)) => {
                let _ = sink
                    .send(tokio_tungstenite::tungstenite::Message::Pong(data))
                    .await;
            }
            Err(e) => {
                error!(%peer, "WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}
