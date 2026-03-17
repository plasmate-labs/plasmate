//! Integration tests for the AWP server protocol.
//!
//! These test the full pipeline: WebSocket -> handler -> fetch -> JS -> SOM -> response.

use plasmate::awp::handler::{handle_request, ConnectionState};
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

async fn start_test_server() -> (String, oneshot::Sender<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base_url = format!("http://{}", addr);

    let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

    let base_for_server = base_url.clone();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut shutdown_rx => {
                    break;
                }
                accept = listener.accept() => {
                    let Ok((mut stream, _peer)) = accept else { continue; };
                    let base = base_for_server.clone();

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

                        let (status, body, content_type): (u16, String, &str) = match path {
                            "/" => (200, HOME_HTML_TEMPLATE.replace("__BASE__", &base), "text/html"),
                            "/next" => (200, NEXT_HTML_TEMPLATE.replace("__BASE__", &base), "text/html"),
                            "/js" => (200, JS_HTML.to_string(), "text/html"),
                            _ => (404, NOT_FOUND_HTML.to_string(), "text/html"),
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

const HOME_HTML_TEMPLATE: &str = r#"<!doctype html>
<html>
  <head>
    <title>AWP Test Home</title>
    <meta property="og:title" content="AWP Test Home" />
    <script type="application/ld+json">{
      "@context": "https://schema.org",
      "@type": "WebSite",
      "name": "AWP Test",
      "url": "__BASE__/"
    }</script>
  </head>
  <body>
    <h1>Hello from Plasmate AWP</h1>
    <p>This is a local fixture page used for integration tests.</p>
    <a href="__BASE__/next">Next</a>
    <form>
      <label>Query <input type="text" name="q" aria-label="Query" /></label>
      <button type="submit">Submit</button>
    </form>
  </body>
</html>"#;

const NEXT_HTML_TEMPLATE: &str = r#"<!doctype html>
<html>
  <head><title>AWP Test Next</title></head>
  <body>
    <h1>Next page</h1>
    <a href="__BASE__/">Home</a>
  </body>
</html>"#;

const JS_HTML: &str = r#"<!doctype html>
<html>
  <head><title>AWP Test JS</title></head>
  <body>
    <h1>JS page</h1>
    <script>window.__plasmate_smoke = 1;</script>
  </body>
</html>"#;

const NOT_FOUND_HTML: &str =
    r#"<!doctype html><html><head><title>Not Found</title></head><body>404</body></html>"#;

/// Helper: run a request through the handler.
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

// ============================================================
// HANDSHAKE
// ============================================================

#[tokio::test]
async fn test_hello_handshake() {
    let mut state = ConnectionState::new();
    let resp = send(
        &mut state,
        "awp.hello",
        json!({
            "awp_version": "0.1",
            "client_name": "test"
        }),
    )
    .await;
    assert!(is_success(&resp));
    let result = get_result(&resp);
    assert_eq!(result["awp_version"], "0.1");
    assert_eq!(result["server_name"], "plasmate");
}

#[tokio::test]
async fn test_method_before_hello_rejected() {
    let mut state = ConnectionState::new();
    let resp = send(&mut state, "session.create", json!({})).await;
    assert!(!is_success(&resp), "Should reject methods before hello");
}

// ============================================================
// SESSION LIFECYCLE
// ============================================================

#[tokio::test]
async fn test_session_create_and_close() {
    let mut state = ConnectionState::new();
    send(&mut state, "awp.hello", json!({"awp_version": "0.1"})).await;

    let resp = send(&mut state, "session.create", json!({})).await;
    assert!(is_success(&resp));
    let session_id = get_result(&resp)["session_id"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(session_id.starts_with("s_"));

    let resp = send(
        &mut state,
        "session.close",
        json!({"session_id": session_id}),
    )
    .await;
    assert!(is_success(&resp));
    assert_eq!(get_result(&resp)["closed"], true);
}

// ============================================================
// NAVIGATION (full pipeline)
// ============================================================

#[tokio::test]
async fn test_navigate_real_page() {
    let (base_url, shutdown) = start_test_server().await;
    let url = format!("{}/", base_url);

    let mut state = ConnectionState::new();
    send(&mut state, "awp.hello", json!({"awp_version": "0.1"})).await;

    let resp = send(&mut state, "session.create", json!({})).await;
    let session_id = get_result(&resp)["session_id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = send(
        &mut state,
        "page.navigate",
        json!({
            "session_id": session_id,
            "url": url
        }),
    )
    .await;

    let _ = shutdown.send(());

    assert!(is_success(&resp), "Navigation should succeed: {:?}", resp);
    let result = get_result(&resp);
    assert_eq!(result["status"], 200);
    assert!(result["som_ready"].as_bool().unwrap());
    assert!(result["html_bytes"].as_u64().unwrap() > 0);
    assert!(result["som_bytes"].as_u64().unwrap() > 0);
    assert!(result["pipeline_ms"].as_u64().is_some());
}

#[tokio::test]
async fn test_navigate_with_js_execution() {
    let (base_url, shutdown) = start_test_server().await;
    let url = format!("{}/js", base_url);

    let mut state = ConnectionState::new();
    send(&mut state, "awp.hello", json!({"awp_version": "0.1"})).await;

    let resp = send(&mut state, "session.create", json!({})).await;
    let session_id = get_result(&resp)["session_id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = send(
        &mut state,
        "page.navigate",
        json!({
            "session_id": session_id,
            "url": url
        }),
    )
    .await;

    let _ = shutdown.send(());

    assert!(is_success(&resp));
    let result = get_result(&resp);
    if let Some(js) = result.get("js") {
        assert!(js["scripts_total"].as_u64().unwrap() > 0);
    }
}

// ============================================================
// OBSERVE (SOM output)
// ============================================================

#[tokio::test]
async fn test_observe_after_navigate() {
    let (base_url, shutdown) = start_test_server().await;
    let url = format!("{}/", base_url);

    let mut state = ConnectionState::new();
    send(&mut state, "awp.hello", json!({"awp_version": "0.1"})).await;

    let resp = send(&mut state, "session.create", json!({})).await;
    let session_id = get_result(&resp)["session_id"]
        .as_str()
        .unwrap()
        .to_string();

    send(
        &mut state,
        "page.navigate",
        json!({
            "session_id": session_id,
            "url": url
        }),
    )
    .await;

    let resp = send(
        &mut state,
        "page.observe",
        json!({
            "session_id": session_id
        }),
    )
    .await;

    let _ = shutdown.send(());

    assert!(is_success(&resp));
    let result = get_result(&resp);
    let som = &result["som"];
    assert_eq!(som["som_version"], "0.1");
    assert!(som["regions"].as_array().unwrap().len() > 0);
}

#[tokio::test]
async fn test_observe_before_navigate_fails() {
    let mut state = ConnectionState::new();
    send(&mut state, "awp.hello", json!({"awp_version": "0.1"})).await;

    let resp = send(&mut state, "session.create", json!({})).await;
    let session_id = get_result(&resp)["session_id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = send(
        &mut state,
        "page.observe",
        json!({
            "session_id": session_id
        }),
    )
    .await;

    assert!(!is_success(&resp), "Should fail with no page loaded");
}

// ============================================================
// EXTRACT (structured data + interactive elements + field queries)
// ============================================================

#[tokio::test]
async fn test_extract_structured_data() {
    let (base_url, shutdown) = start_test_server().await;
    let url = format!("{}/", base_url);

    let mut state = ConnectionState::new();
    send(&mut state, "awp.hello", json!({"awp_version": "0.1"})).await;

    let resp = send(&mut state, "session.create", json!({})).await;
    let session_id = get_result(&resp)["session_id"]
        .as_str()
        .unwrap()
        .to_string();

    send(
        &mut state,
        "page.navigate",
        json!({
            "session_id": session_id,
            "url": url
        }),
    )
    .await;

    let resp = send(
        &mut state,
        "page.extract",
        json!({
            "session_id": session_id,
            "structured_data": true
        }),
    )
    .await;

    let _ = shutdown.send(());

    assert!(
        is_success(&resp),
        "Structured data extraction should work: {:?}",
        resp
    );
    let result = get_result(&resp);
    if let Some(sd) = result.get("structured_data") {
        if !sd.is_null() {
            assert!(
                sd.get("json_ld").is_some()
                    || sd.get("open_graph").is_some()
                    || sd.get("meta").is_some(),
                "Should have some structured data"
            );
        }
    }
}

#[tokio::test]
async fn test_extract_interactive_elements() {
    let (base_url, shutdown) = start_test_server().await;
    let url = format!("{}/", base_url);

    let mut state = ConnectionState::new();
    send(&mut state, "awp.hello", json!({"awp_version": "0.1"})).await;

    let resp = send(&mut state, "session.create", json!({})).await;
    let session_id = get_result(&resp)["session_id"]
        .as_str()
        .unwrap()
        .to_string();

    send(
        &mut state,
        "page.navigate",
        json!({
            "session_id": session_id,
            "url": url
        }),
    )
    .await;

    let resp = send(
        &mut state,
        "page.extract",
        json!({
            "session_id": session_id,
            "interactive_elements": true
        }),
    )
    .await;

    let _ = shutdown.send(());

    assert!(is_success(&resp));
    let result = get_result(&resp);
    assert!(result["count"].as_u64().unwrap() > 0);
    let elements = result["interactive_elements"].as_array().unwrap();
    assert!(elements.iter().any(|e| e["role"] == "link"));
}

// ============================================================
// ACT (click navigation)
// ============================================================

#[tokio::test]
async fn test_act_click_link() {
    let (base_url, shutdown) = start_test_server().await;
    let url = format!("{}/", base_url);

    let mut state = ConnectionState::new();
    send(&mut state, "awp.hello", json!({"awp_version": "0.1"})).await;

    let resp = send(&mut state, "session.create", json!({})).await;
    let session_id = get_result(&resp)["session_id"]
        .as_str()
        .unwrap()
        .to_string();

    send(
        &mut state,
        "page.navigate",
        json!({
            "session_id": session_id,
            "url": url
        }),
    )
    .await;

    let resp = send(
        &mut state,
        "page.act",
        json!({
            "session_id": session_id,
            "intent": {
                "action": "click",
                "target": {
                    "role": "link"
                }
            }
        }),
    )
    .await;

    let _ = shutdown.send(());

    assert!(is_success(&resp), "Click should succeed: {:?}", resp);
    let result = get_result(&resp);
    assert_eq!(result["status"], "ok");
    assert_eq!(result["effects"]["navigated"], true);
    assert_eq!(result["effects"]["som_changed"], true);
}

#[tokio::test]
async fn test_act_type_into_field() {
    let (base_url, shutdown) = start_test_server().await;
    let url = format!("{}/", base_url);

    let mut state = ConnectionState::new();
    send(&mut state, "awp.hello", json!({"awp_version": "0.1"})).await;

    let resp = send(&mut state, "session.create", json!({})).await;
    let session_id = get_result(&resp)["session_id"]
        .as_str()
        .unwrap()
        .to_string();

    send(
        &mut state,
        "page.navigate",
        json!({
            "session_id": session_id,
            "url": url
        }),
    )
    .await;

    let extract_resp = send(
        &mut state,
        "page.extract",
        json!({
            "session_id": session_id,
            "interactive_elements": true
        }),
    )
    .await;

    let elements = get_result(&extract_resp)["interactive_elements"]
        .as_array()
        .unwrap();
    let text_input = elements.iter().find(|e| e["role"] == "text_input");

    if let Some(input) = text_input {
        let input_id = input["id"].as_str().unwrap();
        let resp = send(
            &mut state,
            "page.act",
            json!({
                "session_id": session_id,
                "intent": {
                    "action": "type",
                    "target": {"ref": input_id},
                    "value": "hello world"
                }
            }),
        )
        .await;

        assert!(is_success(&resp));
        assert_eq!(get_result(&resp)["effects"]["som_changed"], true);
    }

    let _ = shutdown.send(());
}

// ============================================================
// FULL WORKFLOW: navigate -> observe -> extract -> act -> observe
// ============================================================

#[tokio::test]
async fn test_full_agent_workflow() {
    let (base_url, shutdown) = start_test_server().await;
    let url = format!("{}/", base_url);

    let mut state = ConnectionState::new();

    // 1. Handshake
    let resp = send(
        &mut state,
        "awp.hello",
        json!({
            "awp_version": "0.1",
            "client_name": "integration-test"
        }),
    )
    .await;
    assert!(is_success(&resp));

    // 2. Create session
    let resp = send(&mut state, "session.create", json!({})).await;
    let session_id = get_result(&resp)["session_id"]
        .as_str()
        .unwrap()
        .to_string();

    // 3. Navigate
    let resp = send(
        &mut state,
        "page.navigate",
        json!({
            "session_id": &session_id,
            "url": url
        }),
    )
    .await;
    assert!(is_success(&resp));
    assert_eq!(get_result(&resp)["status"], 200);

    // 4. Observe SOM
    let resp = send(
        &mut state,
        "page.observe",
        json!({
            "session_id": &session_id
        }),
    )
    .await;
    assert!(is_success(&resp));
    let som = &get_result(&resp)["som"];
    assert!(som["regions"].as_array().unwrap().len() > 0);

    // 5. Extract interactive elements
    let resp = send(
        &mut state,
        "page.extract",
        json!({
            "session_id": &session_id,
            "interactive_elements": true
        }),
    )
    .await;
    assert!(is_success(&resp));
    let count = get_result(&resp)["count"].as_u64().unwrap();
    assert!(count > 0);

    // 6. Click a link (navigates to new page)
    let resp = send(
        &mut state,
        "page.act",
        json!({
            "session_id": &session_id,
            "intent": {
                "action": "click",
                "target": {"role": "link"}
            }
        }),
    )
    .await;
    assert!(is_success(&resp));

    // 7. Observe new page
    let resp = send(
        &mut state,
        "page.observe",
        json!({
            "session_id": &session_id
        }),
    )
    .await;
    assert!(is_success(&resp));

    // 8. Close session
    let resp = send(
        &mut state,
        "session.close",
        json!({
            "session_id": &session_id
        }),
    )
    .await;
    assert!(is_success(&resp));

    let _ = shutdown.send(());
}
