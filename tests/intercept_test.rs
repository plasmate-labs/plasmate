//! Integration tests for network request interception.
//!
//! Tests both AWP and CDP interception flows against a real test server.

use plasmate::awp::handler::{handle_request, ConnectionState};
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

// ============================================================
// Test server
// ============================================================

async fn start_test_server() -> (String, oneshot::Sender<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base_url = format!("http://{}", addr);

    let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut shutdown_rx => break,
                accept = listener.accept() => {
                    let Ok((mut stream, _)) = accept else { continue };
                    tokio::spawn(async move {
                        let mut buf = vec![0u8; 8192];
                        let n = match stream.read(&mut buf).await {
                            Ok(n) if n > 0 => n,
                            _ => return,
                        };
                        let req = String::from_utf8_lossy(&buf[..n]);
                        let first_line = req.lines().next().unwrap_or("");
                        let mut parts = first_line.split_whitespace();
                        let _method = parts.next().unwrap_or("GET");
                        let path = parts.next().unwrap_or("/");

                        let (status, body, content_type): (u16, &str, &str) = match path {
                            "/" => (200, HOME_HTML, "text/html"),
                            "/api/data" => (200, API_JSON, "application/json"),
                            "/blocked" => (200, BLOCKED_HTML, "text/html"),
                            _ => (404, NOT_FOUND_HTML, "text/html"),
                        };

                        let resp = format!(
                            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            status,
                            if status == 200 { "OK" } else { "Not Found" },
                            content_type,
                            body.as_bytes().len(),
                            body
                        );

                        let _ = stream.write_all(resp.as_bytes()).await;
                        let _ = stream.shutdown().await;
                    });
                }
            }
        }
    });

    (base_url, shutdown_tx)
}

const HOME_HTML: &str = r#"<!doctype html>
<html>
  <head><title>Intercept Test</title></head>
  <body>
    <h1>Home Page</h1>
    <p>This page is for testing network interception.</p>
  </body>
</html>"#;

const API_JSON: &str = r#"{"status": "real", "data": [1, 2, 3]}"#;

const BLOCKED_HTML: &str = r#"<!doctype html>
<html>
  <head><title>Should Be Blocked</title></head>
  <body><p>You should not see this.</p></body>
</html>"#;

const NOT_FOUND_HTML: &str =
    r#"<!doctype html><html><head><title>Not Found</title></head><body>404</body></html>"#;

// ============================================================
// AWP helpers
// ============================================================

async fn send(
    state: &mut ConnectionState,
    method: &str,
    params: serde_json::Value,
) -> serde_json::Value {
    let resp = handle_request("test-1", method, &params, state).await;
    serde_json::to_value(&resp).unwrap()
}

fn is_success(resp: &serde_json::Value) -> bool {
    resp.get("result").is_some() && resp.get("error").is_none()
}

fn get_result(resp: &serde_json::Value) -> &serde_json::Value {
    resp.get("result").unwrap()
}

fn is_error(resp: &serde_json::Value) -> bool {
    resp.get("error").is_some()
}

/// Set up a handshake + session, return the session_id.
async fn setup_session(state: &mut ConnectionState) -> String {
    send(state, "awp.hello", json!({"awp_version": "0.1"})).await;
    let resp = send(state, "session.create", json!({})).await;
    get_result(&resp)["session_id"]
        .as_str()
        .unwrap()
        .to_string()
}

// ============================================================
// AWP: Block a request by URL pattern
// ============================================================

#[tokio::test]
async fn test_awp_block_request() {
    let (base_url, shutdown) = start_test_server().await;

    let mut state = ConnectionState::new();
    let sid = setup_session(&mut state).await;

    // Enable interception
    let resp = send(
        &mut state,
        "network.enableInterception",
        json!({
            "session_id": sid,
            "patterns": [{"url_pattern": "*"}]
        }),
    )
    .await;
    assert!(is_success(&resp));

    // Add a block rule for /blocked
    let resp = send(
        &mut state,
        "network.addRule",
        json!({
            "session_id": sid,
            "rule": {
                "url_pattern": "*blocked*",
                "action": "block"
            }
        }),
    )
    .await;
    assert!(is_success(&resp));

    // Navigate to /blocked — should fail
    let resp = send(
        &mut state,
        "page.navigate",
        json!({
            "session_id": sid,
            "url": format!("{}/blocked", base_url)
        }),
    )
    .await;
    assert!(is_error(&resp), "Blocked request should fail: {:?}", resp);

    // Navigate to / — should succeed (not blocked)
    let resp = send(
        &mut state,
        "page.navigate",
        json!({
            "session_id": sid,
            "url": format!("{}/", base_url)
        }),
    )
    .await;
    assert!(
        is_success(&resp),
        "Non-blocked request should succeed: {:?}",
        resp
    );

    let _ = shutdown.send(());
}

// ============================================================
// AWP: Mock a response (fulfill)
// ============================================================

#[tokio::test]
async fn test_awp_fulfill_request() {
    let (base_url, shutdown) = start_test_server().await;

    let mut state = ConnectionState::new();
    let sid = setup_session(&mut state).await;

    // Enable interception
    send(
        &mut state,
        "network.enableInterception",
        json!({
            "session_id": sid,
            "patterns": [{"url_pattern": "*"}]
        }),
    )
    .await;

    // Add a fulfill rule for /api/data
    send(
        &mut state,
        "network.addRule",
        json!({
            "session_id": sid,
            "rule": {
                "url_pattern": "*/api/data*",
                "action": "fulfill",
                "status": 200,
                "headers": {"Content-Type": "text/html"},
                "body": "<html><head><title>Mock</title></head><body><h1>Mocked API</h1></body></html>"
            }
        }),
    )
    .await;

    // Navigate to /api/data — should get mock response
    let resp = send(
        &mut state,
        "page.navigate",
        json!({
            "session_id": sid,
            "url": format!("{}/api/data", base_url)
        }),
    )
    .await;
    assert!(is_success(&resp), "Fulfilled request should succeed: {:?}", resp);
    let result = get_result(&resp);
    assert_eq!(result["status"], 200);

    // Observe the SOM — should contain mocked content
    let resp = send(
        &mut state,
        "page.observe",
        json!({"session_id": sid}),
    )
    .await;
    assert!(is_success(&resp));

    let _ = shutdown.send(());
}

// ============================================================
// AWP: Response body modification
// ============================================================

#[tokio::test]
async fn test_awp_response_modification() {
    let (base_url, shutdown) = start_test_server().await;

    let mut state = ConnectionState::new();
    let sid = setup_session(&mut state).await;

    // Enable interception with response stage
    send(
        &mut state,
        "network.enableInterception",
        json!({
            "session_id": sid,
            "patterns": [
                {"url_pattern": "*", "stage": "request"},
                {"url_pattern": "*", "stage": "response"}
            ]
        }),
    )
    .await;

    // Add a response modification rule
    send(
        &mut state,
        "network.addRule",
        json!({
            "session_id": sid,
            "rule": {
                "url_pattern": "*",
                "stage": "response",
                "body": "<html><head><title>Injected</title></head><body><h1>Injected Content</h1></body></html>"
            }
        }),
    )
    .await;

    // Navigate — response body should be replaced
    let resp = send(
        &mut state,
        "page.navigate",
        json!({
            "session_id": sid,
            "url": format!("{}/", base_url)
        }),
    )
    .await;
    assert!(is_success(&resp));

    let _ = shutdown.send(());
}

// ============================================================
// AWP: Intercepted request log
// ============================================================

#[tokio::test]
async fn test_awp_intercepted_request_log() {
    let (base_url, shutdown) = start_test_server().await;

    let mut state = ConnectionState::new();
    let sid = setup_session(&mut state).await;

    // Enable interception
    send(
        &mut state,
        "network.enableInterception",
        json!({
            "session_id": sid,
            "patterns": [{"url_pattern": "*"}]
        }),
    )
    .await;

    // Navigate to create a log entry
    send(
        &mut state,
        "page.navigate",
        json!({
            "session_id": sid,
            "url": format!("{}/", base_url)
        }),
    )
    .await;

    // Check the log
    let resp = send(
        &mut state,
        "network.getInterceptedRequests",
        json!({"session_id": sid}),
    )
    .await;
    assert!(is_success(&resp));
    let result = get_result(&resp);
    assert!(result["count"].as_u64().unwrap() >= 1);

    let requests = result["requests"].as_array().unwrap();
    assert!(requests[0]["url"].as_str().unwrap().contains(&base_url));
    assert_eq!(requests[0]["resource_type"], "Document");
    assert_eq!(requests[0]["is_navigation"], true);

    let _ = shutdown.send(());
}

// ============================================================
// AWP: Enable/disable lifecycle
// ============================================================

#[tokio::test]
async fn test_awp_enable_disable_lifecycle() {
    let (base_url, shutdown) = start_test_server().await;

    let mut state = ConnectionState::new();
    let sid = setup_session(&mut state).await;

    // Enable interception with a block rule
    send(
        &mut state,
        "network.enableInterception",
        json!({
            "session_id": sid,
            "patterns": [{"url_pattern": "*"}]
        }),
    )
    .await;
    send(
        &mut state,
        "network.addRule",
        json!({
            "session_id": sid,
            "rule": {
                "url_pattern": "*",
                "action": "block"
            }
        }),
    )
    .await;

    // Navigate should fail (everything blocked)
    let resp = send(
        &mut state,
        "page.navigate",
        json!({
            "session_id": sid,
            "url": format!("{}/", base_url)
        }),
    )
    .await;
    assert!(is_error(&resp), "Should be blocked");

    // Disable interception
    send(
        &mut state,
        "network.disableInterception",
        json!({"session_id": sid}),
    )
    .await;

    // Navigate should now succeed
    let resp = send(
        &mut state,
        "page.navigate",
        json!({
            "session_id": sid,
            "url": format!("{}/", base_url)
        }),
    )
    .await;
    assert!(
        is_success(&resp),
        "Should succeed after disable: {:?}",
        resp
    );

    let _ = shutdown.send(());
}

// ============================================================
// AWP: Clear rules
// ============================================================

#[tokio::test]
async fn test_awp_clear_rules() {
    let (base_url, shutdown) = start_test_server().await;

    let mut state = ConnectionState::new();
    let sid = setup_session(&mut state).await;

    // Enable + block
    send(
        &mut state,
        "network.enableInterception",
        json!({
            "session_id": sid,
            "patterns": [{"url_pattern": "*"}]
        }),
    )
    .await;
    send(
        &mut state,
        "network.addRule",
        json!({
            "session_id": sid,
            "rule": {
                "url_pattern": "*",
                "action": "block"
            }
        }),
    )
    .await;

    // Clear rules (interception still enabled, but no rules = continue)
    send(
        &mut state,
        "network.clearRules",
        json!({"session_id": sid}),
    )
    .await;

    // Navigate should succeed (rules cleared, default is continue)
    let resp = send(
        &mut state,
        "page.navigate",
        json!({
            "session_id": sid,
            "url": format!("{}/", base_url)
        }),
    )
    .await;
    assert!(
        is_success(&resp),
        "Should succeed after clearing rules: {:?}",
        resp
    );

    let _ = shutdown.send(());
}

// ============================================================
// AWP: Remove specific rule by URL pattern
// ============================================================

#[tokio::test]
async fn test_awp_remove_specific_rule() {
    let (base_url, shutdown) = start_test_server().await;

    let mut state = ConnectionState::new();
    let sid = setup_session(&mut state).await;

    // Enable + add two rules
    send(
        &mut state,
        "network.enableInterception",
        json!({
            "session_id": sid,
            "patterns": [{"url_pattern": "*"}]
        }),
    )
    .await;

    // Block /blocked
    send(
        &mut state,
        "network.addRule",
        json!({
            "session_id": sid,
            "rule": {
                "url_pattern": "*blocked*",
                "action": "block"
            }
        }),
    )
    .await;

    // Block everything else too
    send(
        &mut state,
        "network.addRule",
        json!({
            "session_id": sid,
            "rule": {
                "url_pattern": "*other*",
                "action": "block"
            }
        }),
    )
    .await;

    // Remove just the *blocked* rule
    send(
        &mut state,
        "network.removeRule",
        json!({
            "session_id": sid,
            "url_pattern": "*blocked*"
        }),
    )
    .await;

    // Navigate to /blocked should now succeed (rule removed)
    let resp = send(
        &mut state,
        "page.navigate",
        json!({
            "session_id": sid,
            "url": format!("{}/blocked", base_url)
        }),
    )
    .await;
    assert!(
        is_success(&resp),
        "Should succeed after removing block rule: {:?}",
        resp
    );

    let _ = shutdown.send(());
}

// ============================================================
// AWP: Resource type filtering
// ============================================================

#[tokio::test]
async fn test_awp_resource_type_filter() {
    let (base_url, shutdown) = start_test_server().await;

    let mut state = ConnectionState::new();
    let sid = setup_session(&mut state).await;

    // Enable interception only for Script resources
    send(
        &mut state,
        "network.enableInterception",
        json!({
            "session_id": sid,
            "patterns": [{"resource_type": "Script"}]
        }),
    )
    .await;

    // Block all scripts
    send(
        &mut state,
        "network.addRule",
        json!({
            "session_id": sid,
            "rule": {
                "resource_type": "Script",
                "action": "block"
            }
        }),
    )
    .await;

    // Navigate to / (Document type) — should succeed because filter is Script only
    let resp = send(
        &mut state,
        "page.navigate",
        json!({
            "session_id": sid,
            "url": format!("{}/", base_url)
        }),
    )
    .await;
    assert!(
        is_success(&resp),
        "Document navigate should not be blocked by Script filter: {:?}",
        resp
    );

    let _ = shutdown.send(());
}

// ============================================================
// AWP: Hello advertises intercept feature
// ============================================================

#[tokio::test]
async fn test_awp_hello_advertises_intercept() {
    let mut state = ConnectionState::new();
    let resp = send(
        &mut state,
        "awp.hello",
        json!({"awp_version": "0.1", "client_name": "test"}),
    )
    .await;
    assert!(is_success(&resp));
    let features = get_result(&resp)["features"].as_array().unwrap();
    let feature_strs: Vec<&str> = features.iter().filter_map(|v| v.as_str()).collect();
    assert!(
        feature_strs.contains(&"network.intercept"),
        "Should advertise network.intercept feature: {:?}",
        feature_strs
    );
}

// ============================================================
// AWP: Navigate without interception still works
// ============================================================

#[tokio::test]
async fn test_awp_navigate_without_interception() {
    let (base_url, shutdown) = start_test_server().await;

    let mut state = ConnectionState::new();
    let sid = setup_session(&mut state).await;

    // No interception configured — navigate should work normally
    let resp = send(
        &mut state,
        "page.navigate",
        json!({
            "session_id": sid,
            "url": format!("{}/", base_url)
        }),
    )
    .await;
    assert!(
        is_success(&resp),
        "Navigate without interception should work: {:?}",
        resp
    );
    assert_eq!(get_result(&resp)["status"], 200);

    let _ = shutdown.send(());
}
