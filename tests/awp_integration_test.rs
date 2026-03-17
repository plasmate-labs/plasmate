//! Integration tests for the AWP server protocol.
//!
//! These test the full pipeline: WebSocket -> handler -> fetch -> JS -> SOM -> response.

use plasmate::awp::handler::{handle_request, ConnectionState};
use serde_json::json;

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
            "url": "https://example.com"
        }),
    )
    .await;

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
            "url": "https://httpbin.org"
        }),
    )
    .await;

    assert!(is_success(&resp));
    let result = get_result(&resp);
    // httpbin has inline JS
    if let Some(js) = result.get("js") {
        assert!(js["scripts_total"].as_u64().unwrap() > 0);
    }
}

// ============================================================
// OBSERVE (SOM output)
// ============================================================

#[tokio::test]
async fn test_observe_after_navigate() {
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
            "url": "https://example.com"
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
            "url": "https://www.bbc.com/news"
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

    assert!(
        is_success(&resp),
        "Structured data extraction should work: {:?}",
        resp
    );
    let result = get_result(&resp);
    // BBC should have structured data
    if let Some(sd) = result.get("structured_data") {
        if !sd.is_null() {
            // Should have at least some metadata
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
            "url": "https://example.com"
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

    assert!(is_success(&resp));
    let result = get_result(&resp);
    assert!(
        result["count"].as_u64().unwrap() > 0,
        "example.com should have at least one link"
    );
    let elements = result["interactive_elements"].as_array().unwrap();
    assert!(elements.iter().any(|e| e["role"] == "link"));
}

// ============================================================
// ACT (click navigation)
// ============================================================

#[tokio::test]
async fn test_act_click_link() {
    let mut state = ConnectionState::new();
    send(&mut state, "awp.hello", json!({"awp_version": "0.1"})).await;

    let resp = send(&mut state, "session.create", json!({})).await;
    let session_id = get_result(&resp)["session_id"]
        .as_str()
        .unwrap()
        .to_string();

    // Navigate to example.com which has a "More information..." link
    send(
        &mut state,
        "page.navigate",
        json!({
            "session_id": session_id,
            "url": "https://example.com"
        }),
    )
    .await;

    // Click the first link (example.com has "More information..." link)
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

    assert!(is_success(&resp), "Click should succeed: {:?}", resp);
    let result = get_result(&resp);
    assert_eq!(result["status"], "ok");
    // The link should have triggered navigation
    assert_eq!(result["effects"]["navigated"], true);
    assert_eq!(result["effects"]["som_changed"], true);
}

#[tokio::test]
async fn test_act_type_into_field() {
    let mut state = ConnectionState::new();
    send(&mut state, "awp.hello", json!({"awp_version": "0.1"})).await;

    let resp = send(&mut state, "session.create", json!({})).await;
    let session_id = get_result(&resp)["session_id"]
        .as_str()
        .unwrap()
        .to_string();

    // Navigate to httpbin which has a form
    send(
        &mut state,
        "page.navigate",
        json!({
            "session_id": session_id,
            "url": "https://httpbin.org"
        }),
    )
    .await;

    // Try to type into any text input (if available)
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
}

// ============================================================
// FULL WORKFLOW: navigate -> observe -> extract -> act -> observe
// ============================================================

#[tokio::test]
async fn test_full_agent_workflow() {
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
            "url": "https://example.com"
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
}
