//! CDP domain handlers.
//!
//! Each function handles one CDP method, translating it to our SOM pipeline.
//! The goal: Puppeteer/Playwright connect and work. Under the hood, everything
//! goes through Plasmate's engine - agents get speed + token efficiency for free.

use std::collections::HashMap;

use serde_json::json;
use tracing::info;

use super::cookies::cookie_from_cdp_params;
use super::session::{CdpTarget, NodeEntry};
use super::types::*;
use crate::som::types::{Element, Som};

// ============================================================
// Browser domain
// ============================================================

pub fn browser_get_version(id: u64) -> CdpResponse {
    CdpResponse::success(
        id,
        json!({
            "protocolVersion": "1.3",
            "product": concat!("Plasmate/", env!("CARGO_PKG_VERSION")),
            "revision": "plasmate",
            "userAgent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36",
            "jsVersion": "12.0"
        }),
    )
}

pub fn browser_close(id: u64) -> CdpResponse {
    CdpResponse::success(id, json!({}))
}

// ============================================================
// Target domain
// ============================================================

pub fn target_get_targets(id: u64, target: &CdpTarget) -> CdpResponse {
    CdpResponse::success(
        id,
        json!({
            "targetInfos": [{
                "targetId": target.target_id,
                "type": "page",
                "title": target.current_som.as_ref().map(|s| s.title.as_str()).unwrap_or(""),
                "url": target.current_url.as_deref().unwrap_or("about:blank"),
                "attached": true,
                "browserContextId": "default",
            }]
        }),
    )
}

pub fn target_create_target(id: u64, target: &CdpTarget) -> CdpResponse {
    CdpResponse::success(
        id,
        json!({
            "targetId": target.target_id,
        }),
    )
}

pub fn target_attach_to_target(id: u64, target: &CdpTarget) -> CdpResponse {
    CdpResponse::success(
        id,
        json!({
            "sessionId": target.session_id,
        }),
    )
}

pub fn target_set_discover_targets(id: u64) -> CdpResponse {
    CdpResponse::success(id, json!({}))
}

// ============================================================
// Page domain
// ============================================================

pub async fn page_navigate(
    id: u64,
    params: &serde_json::Value,
    target: &mut CdpTarget,
) -> (CdpResponse, Vec<CdpEvent>) {
    let url = match params.get("url").and_then(|v| v.as_str()) {
        Some(u) => u,
        None => {
            return (
                CdpResponse::error(id, CDP_ERR_INVALID_PARAMS, "Missing url"),
                vec![],
            )
        }
    };

    info!(url, "CDP: Page.navigate");

    match target.navigate(url).await {
        Ok(result) => {
            let url = result.url.clone();
            let loader_id = result.loader_id.clone();
            let frame_id = result.frame_id.clone();
            let mime_type = result.mime_type.clone();
            let status = result.status;
            let encoded_data_length = result.encoded_data_length;
            let request_id = format!("req_{}", loader_id);

            let events = vec![
                // Minimal Network events so Puppeteer can resolve navigationResponse()
                CdpEvent::new(
                    "Network.requestWillBeSent",
                    json!({
                        "requestId": request_id.clone(),
                        "loaderId": loader_id.clone(),
                        "frameId": frame_id.clone(),
                        "documentURL": url.clone(),
                        "request": {
                            "url": url.clone(),
                            "method": "GET",
                            "headers": {},
                            "initialPriority": "High",
                            "referrerPolicy": "strict-origin-when-cross-origin"
                        },
                        "timestamp": timestamp_sec(),
                        "type": "Document",
                        "initiator": {"type": "other"}
                    }),
                ),
                CdpEvent::new(
                    "Network.responseReceived",
                    json!({
                        "requestId": request_id.clone(),
                        "loaderId": loader_id.clone(),
                        "frameId": frame_id.clone(),
                        "timestamp": timestamp_sec(),
                        "type": "Document",
                        "response": {
                            "url": url.clone(),
                            "status": status,
                            "statusText": "OK",
                            "headers": {},
                            "mimeType": mime_type.clone(),
                            "connectionReused": false,
                            "connectionId": 0,
                            "fromDiskCache": false,
                            "fromServiceWorker": false,
                            "encodedDataLength": encoded_data_length
                        }
                    }),
                ),
                CdpEvent::new(
                    "Network.loadingFinished",
                    json!({
                        "requestId": request_id.clone(),
                        "frameId": frame_id.clone(),
                        "timestamp": timestamp_sec(),
                        "encodedDataLength": encoded_data_length
                    }),
                ),
                CdpEvent::new(
                    "Page.frameStartedLoading",
                    json!({"frameId": frame_id.clone()}),
                ),
                // We intentionally do NOT send executionContextsCleared here.
                // Plasmate owns the engine - context references remain valid
                // across navigations because we resolve evaluate() from SOM,
                // not from a real JS isolate. Clearing would force Puppeteer to
                // wait for new context creation, which is fragile to get right.
                CdpEvent::new(
                    "Page.frameNavigated",
                    json!({
                        "frame": {
                            "id": frame_id.clone(),
                            "loaderId": loader_id.clone(),
                            "url": url.clone(),
                            "securityOrigin": url.clone(),
                            "mimeType": "text/html",
                        }
                    }),
                ),
                // Full lifecycle sequence that Puppeteer's LifecycleWatcher expects
                CdpEvent::new(
                    "Page.lifecycleEvent",
                    json!({
                        "frameId": frame_id.clone(),
                        "loaderId": loader_id.clone(),
                        "name": "init",
                        "timestamp": timestamp_sec(),
                    }),
                ),
                CdpEvent::new(
                    "Page.lifecycleEvent",
                    json!({
                        "frameId": frame_id.clone(),
                        "loaderId": loader_id.clone(),
                        "name": "commit",
                        "timestamp": timestamp_sec(),
                    }),
                ),
                CdpEvent::new(
                    "Page.domContentEventFired",
                    json!({"timestamp": timestamp_sec()}),
                ),
                CdpEvent::new(
                    "Page.lifecycleEvent",
                    json!({
                        "frameId": frame_id.clone(),
                        "loaderId": loader_id.clone(),
                        "name": "DOMContentLoaded",
                        "timestamp": timestamp_sec(),
                    }),
                ),
                CdpEvent::new("Page.loadEventFired", json!({"timestamp": timestamp_sec()})),
                CdpEvent::new(
                    "Page.lifecycleEvent",
                    json!({
                        "frameId": frame_id.clone(),
                        "loaderId": loader_id.clone(),
                        "name": "load",
                        "timestamp": timestamp_sec(),
                    }),
                ),
                CdpEvent::new(
                    "Page.lifecycleEvent",
                    json!({
                        "frameId": frame_id.clone(),
                        "loaderId": loader_id.clone(),
                        "name": "networkAlmostIdle",
                        "timestamp": timestamp_sec(),
                    }),
                ),
                CdpEvent::new(
                    "Page.lifecycleEvent",
                    json!({
                        "frameId": frame_id.clone(),
                        "loaderId": loader_id.clone(),
                        "name": "networkIdle",
                        "timestamp": timestamp_sec(),
                    }),
                ),
                CdpEvent::new(
                    "Page.frameStoppedLoading",
                    json!({"frameId": frame_id.clone()}),
                ),
                CdpEvent::new(
                    "Target.targetInfoChanged",
                    json!({
                        "targetInfo": {
                            "targetId": frame_id.clone(),
                            "type": "page",
                            "title": "",
                            "url": url.clone(),
                            "attached": true,
                            "browserContextId": "default",
                        }
                    }),
                ),
            ];

            (
                CdpResponse::success(
                    id,
                    json!({
                        "frameId": frame_id,
                        "loaderId": loader_id,
                    }),
                ),
                events,
            )
        }
        Err(e) => (CdpResponse::error(id, CDP_ERR_SERVER, &e), vec![]),
    }
}

pub async fn page_set_content(
    id: u64,
    params: &serde_json::Value,
    target: &mut CdpTarget,
) -> (CdpResponse, Vec<CdpEvent>) {
    let html = match params.get("html").and_then(|v| v.as_str()) {
        Some(h) => h,
        None => {
            return (
                CdpResponse::error(id, CDP_ERR_INVALID_PARAMS, "Missing html"),
                vec![],
            )
        }
    };

    info!("CDP: Page.setContent ({} bytes)", html.len());

    match target.set_content(html).await {
        Ok(result) => {
            let frame_id = result.frame_id;
            let loader_id = result.loader_id;

            // Emit lifecycle events (no Network events since there's no fetch)
            let events = vec![
                CdpEvent::new(
                    "Page.frameStartedLoading",
                    json!({"frameId": frame_id.clone()}),
                ),
                CdpEvent::new(
                    "Page.frameNavigated",
                    json!({
                        "frame": {
                            "id": frame_id.clone(),
                            "loaderId": loader_id.clone(),
                            "url": target.current_url.as_deref().unwrap_or("about:blank"),
                            "securityOrigin": target.current_url.as_deref().unwrap_or("about:blank"),
                            "mimeType": "text/html",
                        }
                    }),
                ),
                CdpEvent::new(
                    "Page.lifecycleEvent",
                    json!({
                        "frameId": frame_id.clone(),
                        "loaderId": loader_id.clone(),
                        "name": "init",
                        "timestamp": timestamp_sec(),
                    }),
                ),
                CdpEvent::new(
                    "Page.lifecycleEvent",
                    json!({
                        "frameId": frame_id.clone(),
                        "loaderId": loader_id.clone(),
                        "name": "commit",
                        "timestamp": timestamp_sec(),
                    }),
                ),
                CdpEvent::new(
                    "Page.domContentEventFired",
                    json!({"timestamp": timestamp_sec()}),
                ),
                CdpEvent::new(
                    "Page.lifecycleEvent",
                    json!({
                        "frameId": frame_id.clone(),
                        "loaderId": loader_id.clone(),
                        "name": "DOMContentLoaded",
                        "timestamp": timestamp_sec(),
                    }),
                ),
                CdpEvent::new("Page.loadEventFired", json!({"timestamp": timestamp_sec()})),
                CdpEvent::new(
                    "Page.lifecycleEvent",
                    json!({
                        "frameId": frame_id.clone(),
                        "loaderId": loader_id.clone(),
                        "name": "load",
                        "timestamp": timestamp_sec(),
                    }),
                ),
                CdpEvent::new(
                    "Page.lifecycleEvent",
                    json!({
                        "frameId": frame_id.clone(),
                        "loaderId": loader_id.clone(),
                        "name": "networkAlmostIdle",
                        "timestamp": timestamp_sec(),
                    }),
                ),
                CdpEvent::new(
                    "Page.lifecycleEvent",
                    json!({
                        "frameId": frame_id.clone(),
                        "loaderId": loader_id.clone(),
                        "name": "networkIdle",
                        "timestamp": timestamp_sec(),
                    }),
                ),
                CdpEvent::new(
                    "Page.frameStoppedLoading",
                    json!({"frameId": frame_id.clone()}),
                ),
            ];

            (CdpResponse::success(id, json!({})), events)
        }
        Err(e) => (CdpResponse::error(id, CDP_ERR_SERVER, &e), vec![]),
    }
}

/// Return the current page HTML (effective_html after JS, or raw HTML, or empty).
/// Used by Playwright's page.content() via Page.getResourceContent.
pub fn page_get_content(id: u64, target: &CdpTarget) -> CdpResponse {
    let html = target
        .effective_html
        .as_deref()
        .or(target.current_html.as_deref())
        .unwrap_or("");
    CdpResponse::success(id, json!({"content": html, "base64Encoded": false}))
}

pub fn page_enable(id: u64) -> CdpResponse {
    CdpResponse::success(id, json!({}))
}

pub fn page_get_frame_tree(id: u64, target: &CdpTarget) -> CdpResponse {
    CdpResponse::success(
        id,
        json!({
            "frameTree": {
                "frame": {
                    "id": target.frame_id,
                    "loaderId": target.loader_id,
                    "url": target.current_url.as_deref().unwrap_or("about:blank"),
                    "securityOrigin": target.current_url.as_deref().unwrap_or("about:blank"),
                    "mimeType": "text/html",
                },
                "childFrames": []
            }
        }),
    )
}

pub fn page_set_lifecycle_events_enabled(id: u64) -> CdpResponse {
    CdpResponse::success(id, json!({}))
}

pub fn page_create_isolated_world(id: u64) -> CdpResponse {
    CdpResponse::success(
        id,
        json!({
            "executionContextId": 2,
        }),
    )
}

// ============================================================
// Runtime domain
// ============================================================

pub fn runtime_enable(id: u64, target: &CdpTarget) -> (CdpResponse, Vec<CdpEvent>) {
    let events = vec![CdpEvent::new(
        "Runtime.executionContextCreated",
        json!({
            "context": {
                "id": 1,
                "origin": target.current_url.as_deref().unwrap_or("about:blank"),
                "name": "",
                "uniqueId": "1",
                "auxData": {
                    "isDefault": true,
                    "type": "default",
                    "frameId": target.frame_id,
                }
            }
        }),
    )];
    (CdpResponse::success(id, json!({})), events)
}

pub fn runtime_evaluate(id: u64, params: &serde_json::Value, target: &CdpTarget) -> CdpResponse {
    let expression = params
        .get("expression")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let return_by_value = params
        .get("returnByValue")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let _await_promise = params
        .get("awaitPromise")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Check if we have effective HTML for real JS evaluation
    if target.effective_html.is_some() {
        // Use real V8 evaluation
        match target.evaluate_js(expression) {
            Ok(value) => {
                // Convert the result to CDP format
                let result = value_to_cdp_result(&value, return_by_value);
                CdpResponse::success(id, json!({"result": result}))
            }
            Err(e) => {
                // Return exception details for JS errors
                info!("Runtime.evaluate error: {}", e);
                CdpResponse::success(
                    id,
                    json!({
                        "result": {
                            "type": "undefined",
                        },
                        "exceptionDetails": {
                            "exceptionId": 1,
                            "text": e,
                            "lineNumber": 0,
                            "columnNumber": 0,
                        }
                    }),
                )
            }
        }
    } else {
        // Fallback to pattern matching for common expressions (no page loaded)
        match expression {
            e if e.contains("document.title") => {
                let title = target
                    .current_som
                    .as_ref()
                    .map(|s| s.title.clone())
                    .unwrap_or_default();
                CdpResponse::success(
                    id,
                    json!({
                        "result": {
                            "type": "string",
                            "value": title,
                        }
                    }),
                )
            }
            e if e.contains("document.URL") || e.contains("location.href") => {
                let url = target.current_url.as_deref().unwrap_or("about:blank");
                CdpResponse::success(
                    id,
                    json!({
                        "result": {
                            "type": "string",
                            "value": url,
                        }
                    }),
                )
            }
            e if e.contains("outerHTML") || e.contains("innerHTML") => {
                // Puppeteer calls document.documentElement.outerHTML for page.content()
                let html = target
                    .effective_html
                    .as_deref()
                    .or(target.current_html.as_deref())
                    .unwrap_or("<html></html>");
                CdpResponse::success(
                    id,
                    json!({
                        "result": {
                            "type": "string",
                            "value": html,
                        }
                    }),
                )
            }
            _ => CdpResponse::success(
                id,
                json!({
                    "result": {
                        "type": "undefined",
                    }
                }),
            ),
        }
    }
}

/// Convert a serde_json::Value to CDP result format.
pub fn value_to_cdp_result(value: &serde_json::Value, return_by_value: bool) -> serde_json::Value {
    match value {
        serde_json::Value::Null => json!({"type": "undefined"}),
        serde_json::Value::Bool(b) => json!({"type": "boolean", "value": b}),
        serde_json::Value::Number(n) => json!({"type": "number", "value": n}),
        serde_json::Value::String(s) => {
            // Check for special string values
            if s == "undefined" {
                json!({"type": "undefined"})
            } else if s == "true" {
                json!({"type": "boolean", "value": true})
            } else if s == "false" {
                json!({"type": "boolean", "value": false})
            } else if let Ok(n) = s.parse::<f64>() {
                json!({"type": "number", "value": n})
            } else {
                json!({"type": "string", "value": s})
            }
        }
        serde_json::Value::Array(arr) => {
            if return_by_value {
                json!({
                    "type": "object",
                    "subtype": "array",
                    "value": arr,
                })
            } else {
                json!({
                    "type": "object",
                    "subtype": "array",
                    "description": format!("Array({})", arr.len()),
                    "objectId": format!("arr-{}", arr.len()),
                })
            }
        }
        serde_json::Value::Object(obj) => {
            if return_by_value {
                json!({
                    "type": "object",
                    "value": obj,
                })
            } else {
                json!({
                    "type": "object",
                    "description": "Object",
                    "objectId": "obj-result",
                })
            }
        }
    }
}

// ============================================================
// DOM domain
// ============================================================

pub fn dom_get_document(id: u64, target: &CdpTarget) -> CdpResponse {
    let doc_node = target.node_map.get(&target.document_node_id);

    let children: Vec<serde_json::Value> = if let Some(doc) = doc_node {
        doc.children_ids
            .iter()
            .filter_map(|cid| target.node_map.get(cid))
            .map(|n| node_to_cdp(n, target, 2))
            .collect()
    } else {
        vec![]
    };

    CdpResponse::success(
        id,
        json!({
            "root": {
                "nodeId": target.document_node_id,
                "backendNodeId": target.document_node_id,
                "nodeType": 9,
                "nodeName": "#document",
                "localName": "",
                "nodeValue": "",
                "childNodeCount": children.len(),
                "children": children,
                "documentURL": target.current_url.as_deref().unwrap_or("about:blank"),
                "baseURL": target.current_url.as_deref().unwrap_or("about:blank"),
                "xmlVersion": "",
            }
        }),
    )
}

pub fn dom_query_selector(id: u64, params: &serde_json::Value, target: &CdpTarget) -> CdpResponse {
    let selector = params
        .get("selector")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    match target.query_selector(selector) {
        Some(node_id) => CdpResponse::success(id, json!({"nodeId": node_id})),
        None => CdpResponse::success(id, json!({"nodeId": 0})),
    }
}

pub fn dom_query_selector_all(
    id: u64,
    params: &serde_json::Value,
    target: &CdpTarget,
) -> CdpResponse {
    let selector = params
        .get("selector")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let node_ids = target.query_selector_all(selector);
    CdpResponse::success(id, json!({"nodeIds": node_ids}))
}

pub fn dom_describe_node(id: u64, params: &serde_json::Value, target: &CdpTarget) -> CdpResponse {
    let node_id = params.get("nodeId").and_then(|v| v.as_u64()).unwrap_or(0);

    match target.node_map.get(&node_id) {
        Some(entry) => CdpResponse::success(
            id,
            json!({
                "node": node_to_cdp(entry, target, 0),
            }),
        ),
        None => CdpResponse::error(id, CDP_ERR_NOT_FOUND, "Node not found"),
    }
}

pub fn dom_resolve_node(id: u64, params: &serde_json::Value, target: &CdpTarget) -> CdpResponse {
    let node_id = params
        .get("nodeId")
        .and_then(|v| v.as_u64())
        .or_else(|| params.get("backendNodeId").and_then(|v| v.as_u64()))
        .unwrap_or(0);

    match target.node_map.get(&node_id) {
        Some(entry) => CdpResponse::success(
            id,
            json!({
                "object": {
                    "type": "object",
                    "subtype": "node",
                    "className": entry.node_name,
                    "description": format!("{}#{}", entry.node_name, node_id),
                    "objectId": format!("node-{}", node_id),
                }
            }),
        ),
        None => CdpResponse::error(id, CDP_ERR_NOT_FOUND, "Node not found"),
    }
}

/// Return a stub box model (consistent with the DOM shim's getBoundingClientRect).
/// Plasmate has no layout engine, so all elements report 100x100 at (0,0).
pub fn dom_get_box_model(id: u64, params: &serde_json::Value, target: &CdpTarget) -> CdpResponse {
    let node_id = params
        .get("nodeId")
        .and_then(|v| v.as_u64())
        .or_else(|| params.get("backendNodeId").and_then(|v| v.as_u64()))
        .unwrap_or(0);

    if node_id == 0 || !target.node_map.contains_key(&node_id) {
        return CdpResponse::error(id, CDP_ERR_NOT_FOUND, "Node not found");
    }

    // Quad format: [x1,y1, x2,y2, x3,y3, x4,y4] (four corners of rectangle)
    let quad = vec![0, 0, 100, 0, 100, 100, 0, 100];
    CdpResponse::success(
        id,
        json!({
            "model": {
                "content": quad,
                "padding": quad,
                "border": quad,
                "margin": quad,
                "width": 100,
                "height": 100,
            }
        }),
    )
}

// ============================================================
// Input domain
// ============================================================

pub async fn input_dispatch_mouse_event(
    id: u64,
    params: &serde_json::Value,
    _target: &mut CdpTarget,
) -> (CdpResponse, Vec<CdpEvent>) {
    let event_type = params.get("type").and_then(|v| v.as_str()).unwrap_or("");

    if event_type != "mousePressed" && event_type != "mouseReleased" {
        // Just acknowledge move/other events
        return (CdpResponse::success(id, json!({})), vec![]);
    }

    // For mousePressed on a link, navigate
    // CDP click usually comes as mousePressed + mouseReleased
    if event_type == "mousePressed" {
        // In a real implementation, we'd map x,y coordinates to elements.
        // For now, this is acknowledged. The actual clicking happens through
        // Runtime.evaluate or DOM interaction in Puppeteer's higher-level API.
        info!("CDP: mouse click event");
    }

    (CdpResponse::success(id, json!({})), vec![])
}

pub fn input_dispatch_key_event(id: u64, _params: &serde_json::Value) -> CdpResponse {
    CdpResponse::success(id, json!({}))
}

/// Input.insertText — insert text at the current focus point.
/// Puppeteer uses this for page.type() after focusing an element.
pub fn input_insert_text(id: u64, params: &serde_json::Value, target: &CdpTarget) -> CdpResponse {
    let text = params.get("text").and_then(|v| v.as_str()).unwrap_or("");

    if text.is_empty() {
        return CdpResponse::success(id, json!({}));
    }

    // If we have effective HTML, use JS evaluation to set the value
    // of the currently focused/active element.
    if target.effective_html.is_some() {
        let escaped = text
            .replace('\\', "\\\\")
            .replace('\'', "\\'")
            .replace('\n', "\\n");
        let script = format!(
            "(function() {{ \
                var el = document.activeElement || document.querySelector('input, textarea'); \
                if (el && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA')) {{ \
                    el.value = (el.value || '') + '{}'; \
                }} \
            }})()",
            escaped
        );
        let _ = target.evaluate_js(&script);
    }

    CdpResponse::success(id, json!({}))
}

// ============================================================
// Network domain
// ============================================================

pub fn network_enable(id: u64) -> CdpResponse {
    CdpResponse::success(id, json!({}))
}

pub fn network_set_extra_http_headers(
    id: u64,
    params: &serde_json::Value,
    target: &mut CdpTarget,
) -> CdpResponse {
    if let Some(headers) = params.get("headers").and_then(|v| v.as_object()) {
        for (k, v) in headers {
            if let Some(val) = v.as_str() {
                target.extra_headers.insert(k.clone(), val.to_string());
            }
        }
    }
    CdpResponse::success(id, json!({}))
}

pub fn network_get_cookies(id: u64, params: &serde_json::Value, target: &CdpTarget) -> CdpResponse {
    // Get URLs to filter by (optional)
    let urls: Vec<String> = params
        .get("urls")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let cookies = if urls.is_empty() {
        // No URLs specified - return all cookies for current page URL
        if let Some(ref url) = target.current_url {
            target.cookie_jar.get_cookies(url)
        } else {
            target.cookie_jar.get_all_cookies()
        }
    } else {
        // Filter by specified URLs
        let mut all_cookies = Vec::new();
        for url in &urls {
            all_cookies.extend(target.cookie_jar.get_cookies(url));
        }
        // Deduplicate by (name, domain, path)
        let mut seen = std::collections::HashSet::new();
        all_cookies.retain(|c| {
            let key = (c.name.clone(), c.domain.clone(), c.path.clone());
            seen.insert(key)
        });
        all_cookies
    };

    let cookies_json: Vec<serde_json::Value> = cookies.iter().map(|c| c.to_cdp_json()).collect();

    CdpResponse::success(id, json!({"cookies": cookies_json}))
}

pub fn network_get_all_cookies(id: u64, target: &CdpTarget) -> CdpResponse {
    let cookies = target.cookie_jar.get_all_cookies();
    let cookies_json: Vec<serde_json::Value> = cookies.iter().map(|c| c.to_cdp_json()).collect();
    CdpResponse::success(id, json!({"cookies": cookies_json}))
}

pub fn network_set_cookies(
    id: u64,
    params: &serde_json::Value,
    target: &mut CdpTarget,
) -> CdpResponse {
    if let Some(cookies_array) = params.get("cookies").and_then(|v| v.as_array()) {
        for cookie_params in cookies_array {
            if let Some(cookie) = cookie_from_cdp_params(cookie_params) {
                target.cookie_jar.set_cookie(cookie);
            }
        }
    }
    CdpResponse::success(id, json!({}))
}

pub fn network_set_cookie(
    id: u64,
    params: &serde_json::Value,
    target: &mut CdpTarget,
) -> CdpResponse {
    // Network.setCookie takes cookie parameters directly (not wrapped in "cookies" array)
    if let Some(cookie) = cookie_from_cdp_params(params) {
        target.cookie_jar.set_cookie(cookie);
        CdpResponse::success(id, json!({"success": true}))
    } else {
        CdpResponse::error(id, CDP_ERR_INVALID_PARAMS, "Invalid cookie parameters")
    }
}

pub fn network_delete_cookies(
    id: u64,
    params: &serde_json::Value,
    target: &mut CdpTarget,
) -> CdpResponse {
    let name = match params.get("name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return CdpResponse::error(id, CDP_ERR_INVALID_PARAMS, "Missing cookie name"),
    };

    let url = params.get("url").and_then(|v| v.as_str());
    let domain = params.get("domain").and_then(|v| v.as_str());
    let path = params.get("path").and_then(|v| v.as_str());

    target.cookie_jar.delete_cookies(name, url, domain, path);
    CdpResponse::success(id, json!({}))
}

pub fn network_clear_browser_cookies(id: u64, target: &mut CdpTarget) -> CdpResponse {
    target.cookie_jar.clear();
    CdpResponse::success(id, json!({}))
}

// ============================================================
// Fetch domain (network interception)
// ============================================================

use crate::network::intercept::{
    ErrorReason, FulfillParams, InterceptAction, InterceptRule, RequestOverrides, RequestPattern,
    RequestStage, ResourceType, ResponseOverrides, ResponseRule,
};

/// Fetch.enable — enable interception with URL patterns.
pub fn fetch_enable(id: u64, params: &serde_json::Value, target: &mut CdpTarget) -> CdpResponse {
    let patterns = params
        .get("patterns")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|p| {
                    let url_pattern = p
                        .get("urlPattern")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    let resource_type = p
                        .get("resourceType")
                        .and_then(|v| v.as_str())
                        .map(ResourceType::from_cdp_str);
                    let request_stage = p
                        .get("requestStage")
                        .and_then(|v| v.as_str())
                        .map(|s| match s {
                            "Response" => RequestStage::Response,
                            _ => RequestStage::Request,
                        })
                        .unwrap_or(RequestStage::Request);

                    RequestPattern {
                        url_pattern,
                        resource_type,
                        request_stage,
                    }
                })
                .collect()
        })
        .unwrap_or_else(|| {
            // No patterns = match all requests
            vec![RequestPattern {
                url_pattern: Some("*".to_string()),
                resource_type: None,
                request_stage: RequestStage::Request,
            }]
        });

    target.interceptor.enable(patterns);
    info!("Fetch.enable: interception enabled");
    CdpResponse::success(id, json!({}))
}

/// Fetch.disable — disable interception.
pub fn fetch_disable(id: u64, target: &mut CdpTarget) -> CdpResponse {
    target.interceptor.disable();
    info!("Fetch.disable: interception disabled");
    CdpResponse::success(id, json!({}))
}

/// Fetch.fulfillRequest — register a rule to fulfill matching requests with a mock response.
///
/// CDP normally uses this to resolve a paused request. In Plasmate, we register
/// it as a rule that applies to future matching requests.
pub fn fetch_fulfill_request(
    id: u64,
    params: &serde_json::Value,
    target: &mut CdpTarget,
) -> CdpResponse {
    let url_pattern = params
        .get("urlPattern")
        .and_then(|v| v.as_str())
        .or_else(|| params.get("requestId").and_then(|v| v.as_str()))
        .unwrap_or("*")
        .to_string();

    let status = params
        .get("responseCode")
        .and_then(|v| v.as_u64())
        .unwrap_or(200) as u16;

    let headers: Vec<(String, String)> = params
        .get("responseHeaders")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|h| {
                    let name = h.get("name")?.as_str()?;
                    let value = h.get("value")?.as_str()?;
                    Some((name.to_string(), value.to_string()))
                })
                .collect()
        })
        .unwrap_or_default();

    let body = params
        .get("body")
        .and_then(|v| v.as_str())
        .map(String::from);

    target.interceptor.add_rule(InterceptRule {
        pattern: RequestPattern {
            url_pattern: Some(url_pattern),
            resource_type: None,
            request_stage: RequestStage::Request,
        },
        action: InterceptAction::Fulfill(FulfillParams {
            status,
            headers,
            body,
        }),
    });

    CdpResponse::success(id, json!({}))
}

/// Fetch.failRequest — register a rule to fail matching requests.
pub fn fetch_fail_request(
    id: u64,
    params: &serde_json::Value,
    target: &mut CdpTarget,
) -> CdpResponse {
    let url_pattern = params
        .get("urlPattern")
        .and_then(|v| v.as_str())
        .or_else(|| params.get("requestId").and_then(|v| v.as_str()))
        .unwrap_or("*")
        .to_string();

    let reason = params
        .get("errorReason")
        .and_then(|v| v.as_str())
        .map(ErrorReason::from_cdp_str)
        .unwrap_or(ErrorReason::Failed);

    target.interceptor.add_rule(InterceptRule {
        pattern: RequestPattern {
            url_pattern: Some(url_pattern),
            resource_type: None,
            request_stage: RequestStage::Request,
        },
        action: InterceptAction::Fail(reason),
    });

    CdpResponse::success(id, json!({}))
}

/// Fetch.continueRequest — register a rule to continue matching requests with optional overrides.
pub fn fetch_continue_request(
    id: u64,
    params: &serde_json::Value,
    target: &mut CdpTarget,
) -> CdpResponse {
    let url_pattern = params
        .get("urlPattern")
        .and_then(|v| v.as_str())
        .or_else(|| params.get("requestId").and_then(|v| v.as_str()))
        .unwrap_or("*")
        .to_string();

    let url_override = params.get("url").and_then(|v| v.as_str()).map(String::from);
    let method_override = params
        .get("method")
        .and_then(|v| v.as_str())
        .map(String::from);
    let post_data_override = params
        .get("postData")
        .and_then(|v| v.as_str())
        .map(String::from);

    let headers_override: Option<HashMap<String, String>> =
        params.get("headers").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|h| {
                    let name = h.get("name")?.as_str()?.to_string();
                    let value = h.get("value")?.as_str()?.to_string();
                    Some((name, value))
                })
                .collect()
        });

    let has_overrides = url_override.is_some()
        || method_override.is_some()
        || headers_override.is_some()
        || post_data_override.is_some();

    let overrides = if has_overrides {
        Some(RequestOverrides {
            url: url_override,
            method: method_override,
            headers: headers_override,
            post_data: post_data_override,
        })
    } else {
        None
    };

    target.interceptor.add_rule(InterceptRule {
        pattern: RequestPattern {
            url_pattern: Some(url_pattern),
            resource_type: None,
            request_stage: RequestStage::Request,
        },
        action: InterceptAction::Continue(overrides),
    });

    CdpResponse::success(id, json!({}))
}

/// Fetch.continueResponse — register a rule to modify responses.
pub fn fetch_continue_response(
    id: u64,
    params: &serde_json::Value,
    target: &mut CdpTarget,
) -> CdpResponse {
    let url_pattern = params
        .get("urlPattern")
        .and_then(|v| v.as_str())
        .or_else(|| params.get("requestId").and_then(|v| v.as_str()))
        .unwrap_or("*")
        .to_string();

    let status = params
        .get("responseCode")
        .and_then(|v| v.as_u64())
        .map(|s| s as u16);
    let body = params
        .get("body")
        .and_then(|v| v.as_str())
        .map(String::from);

    let headers: Option<HashMap<String, String>> = params
        .get("responseHeaders")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|h| {
                    let name = h.get("name")?.as_str()?.to_string();
                    let value = h.get("value")?.as_str()?.to_string();
                    Some((name, value))
                })
                .collect()
        });

    target.interceptor.add_response_rule(ResponseRule {
        pattern: RequestPattern {
            url_pattern: Some(url_pattern),
            resource_type: None,
            request_stage: RequestStage::Response,
        },
        overrides: ResponseOverrides {
            status,
            headers,
            body,
        },
    });

    CdpResponse::success(id, json!({}))
}

/// Fetch.getResponseBody — return the body of the current page (if any).
pub fn fetch_get_response_body(
    id: u64,
    _params: &serde_json::Value,
    target: &CdpTarget,
) -> CdpResponse {
    match &target.current_html {
        Some(html) => CdpResponse::success(
            id,
            json!({
                "body": html,
                "base64Encoded": false,
            }),
        ),
        None => CdpResponse::error(id, CDP_ERR_SERVER, "No response body available"),
    }
}

// ============================================================
// Emulation domain (Puppeteer needs these)
// ============================================================

pub fn emulation_set_device_metrics_override(
    id: u64,
    params: &serde_json::Value,
    target: &mut CdpTarget,
) -> CdpResponse {
    if let Some(w) = params.get("width").and_then(|v| v.as_u64()) {
        target.viewport_width = w as u32;
    }
    if let Some(h) = params.get("height").and_then(|v| v.as_u64()) {
        target.viewport_height = h as u32;
    }
    if let Some(s) = params.get("deviceScaleFactor").and_then(|v| v.as_f64()) {
        target.device_scale_factor = s;
    }
    CdpResponse::success(id, json!({}))
}

pub fn emulation_set_touch_emulation_enabled(id: u64) -> CdpResponse {
    CdpResponse::success(id, json!({}))
}

// ============================================================
// Page.captureScreenshot
// ============================================================

pub fn page_capture_screenshot(
    id: u64,
    params: &serde_json::Value,
    target: &CdpTarget,
) -> (CdpResponse, Vec<CdpEvent>) {
    use crate::screenshot;

    let url = target.current_url.as_deref().unwrap_or("about:blank");
    let format_str = params
        .get("format")
        .and_then(|v| v.as_str())
        .unwrap_or("png");
    let quality = params
        .get("quality")
        .and_then(|v| v.as_u64())
        .map(|q| q as u32);

    let opts = screenshot::ScreenshotOptions {
        width: target.viewport_width,
        height: target.viewport_height,
        format: screenshot::Format::from_str(format_str),
        quality,
        ..Default::default()
    };

    match screenshot::capture_url(url, &opts) {
        Ok(data) => {
            let base64 = base64_encode_simple(&data);
            (CdpResponse::success(id, json!({"data": base64})), vec![])
        }
        Err(screenshot::ScreenshotError::ChromeNotFound) => {
            // Fall back to SOM if Chrome not available
            if let Some(ref som) = target.current_som {
                let fallback = screenshot::som_fallback(som);
                (CdpResponse::success(id, fallback), vec![])
            } else {
                (
                    CdpResponse::error(
                        id,
                        CDP_ERR_SERVER,
                        "Screenshot requires Chrome/Chromium. Install Chrome for screenshot support.",
                    ),
                    vec![],
                )
            }
        }
        Err(e) => (
            CdpResponse::error(id, CDP_ERR_SERVER, &e.to_string()),
            vec![],
        ),
    }
}

/// Simple base64 encoding for image data (CDP module).
fn base64_encode_simple(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

// ============================================================
// Plasmate custom domain (our SOM-native API over CDP)
// ============================================================

pub fn plasmate_get_som(id: u64, target: &CdpTarget) -> CdpResponse {
    match &target.current_som {
        Some(som) => {
            let som_json = serde_json::to_value(som).unwrap_or(json!(null));
            CdpResponse::success(
                id,
                json!({
                    "som": som_json,
                    "url": target.current_url,
                }),
            )
        }
        None => CdpResponse::error(id, CDP_ERR_SERVER, "No page loaded"),
    }
}

pub fn plasmate_get_structured_data(id: u64, target: &CdpTarget) -> CdpResponse {
    match &target.current_structured_data {
        Some(sd) => {
            let sd_json = serde_json::to_value(sd).unwrap_or(json!(null));
            CdpResponse::success(
                id,
                json!({
                    "structured_data": sd_json,
                    "url": target.current_url,
                }),
            )
        }
        None => CdpResponse::success(
            id,
            json!({
                "structured_data": null,
                "url": target.current_url,
            }),
        ),
    }
}

pub fn plasmate_get_interactive_elements(
    id: u64,
    params: &serde_json::Value,
    target: &CdpTarget,
) -> CdpResponse {
    let id_filter = params.get("id").and_then(|v| v.as_str());
    let cache_key_filter = params
        .get("cacheKey")
        .or_else(|| params.get("cache_key"))
        .and_then(|v| v.as_str());
    let html_id_filter = params
        .get("htmlId")
        .or_else(|| params.get("html_id"))
        .and_then(|v| v.as_str());
    let test_id_filter = params
        .get("testId")
        .or_else(|| params.get("test_id"))
        .and_then(|v| v.as_str());
    let lookup_value = params.get("value").and_then(|v| v.as_str());
    let lookup_by = params.get("by").and_then(|v| v.as_str()).unwrap_or("auto");
    let role_filter = params.get("role").and_then(|v| v.as_str());
    let action_filter = params.get("action").and_then(|v| v.as_str());
    let label_filter = params.get("label").and_then(|v| v.as_str());
    let exact_label = params
        .get("exact")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let enabled_only = params
        .get("enabledOnly")
        .or_else(|| params.get("enabled_only"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let offset = params.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let limit = params
        .get("limit")
        .and_then(|v| v.as_u64())
        .map(|value| value as usize);

    let all_elements: Vec<&Element> = target
        .current_som
        .as_ref()
        .map(interactive_elements)
        .unwrap_or_default();

    let filtered_elements: Vec<&Element> = all_elements
        .iter()
        .copied()
        .filter(|e| id_filter.map(|target_id| e.id == target_id).unwrap_or(true))
        .filter(|e| {
            cache_key_filter
                .map(|cache_key| action_plan_cache_key(e) == cache_key)
                .unwrap_or(true)
        })
        .filter(|e| {
            html_id_filter
                .map(|html_id| e.html_id.as_deref() == Some(html_id))
                .unwrap_or(true)
        })
        .filter(|e| {
            test_id_filter
                .map(|test_id| element_test_id(e) == Some(test_id))
                .unwrap_or(true)
        })
        .filter(|e| {
            lookup_value
                .map(|value| element_matches_lookup(e, value, lookup_by, exact_label))
                .unwrap_or(true)
        })
        .filter(|e| {
            role_filter
                .map(|role| element_matches_role_filter(e, role))
                .unwrap_or(true)
        })
        .filter(|e| {
            action_filter
                .map(|action| {
                    e.actions
                        .as_ref()
                        .map(|actions| actions.iter().any(|a| a == action))
                        .unwrap_or(false)
                })
                .unwrap_or(true)
        })
        .filter(|e| {
            label_filter
                .map(|label| {
                    let element_label = e.label.as_deref().or(e.text.as_deref()).unwrap_or("");
                    if exact_label {
                        element_label == label
                    } else {
                        element_label.to_lowercase().contains(&label.to_lowercase())
                    }
                })
                .unwrap_or(true)
        })
        .filter(|e| !enabled_only || interactive_enabled_state(e).0)
        .collect();

    let filtered_count = filtered_elements.len();
    let mut paged_elements: Vec<&Element> = filtered_elements.into_iter().skip(offset).collect();
    if let Some(limit) = limit {
        paged_elements.truncate(limit);
    }
    let elements: Vec<serde_json::Value> = paged_elements
        .into_iter()
        .map(interactive_element_to_json)
        .collect();
    let count = elements.len();

    CdpResponse::success(
        id,
        json!({
            "elements": elements,
            "count": count,
            "filtered_count": filtered_count,
            "total_interactive_count": all_elements.len(),
            "offset": offset,
            "limit": limit,
        }),
    )
}

pub fn plasmate_get_markdown(id: u64, target: &CdpTarget) -> CdpResponse {
    // Generate markdown from SOM for compatibility with tools expecting markdown
    let md = if let Some(som) = &target.current_som {
        som_to_markdown(som)
    } else {
        String::new()
    };

    CdpResponse::success(
        id,
        json!({
            "markdown": md,
            "url": target.current_url,
        }),
    )
}

pub async fn plasmate_list_plugins(id: u64, target: &CdpTarget) -> CdpResponse {
    let manifests = if let Some(ref pm) = target.plugins {
        let pm = pm.lock().await;
        pm.manifests()
            .iter()
            .map(|m| {
                json!({
                    "name": m.name,
                    "version": m.version,
                    "hooks": m.hooks,
                })
            })
            .collect::<Vec<_>>()
    } else {
        vec![]
    };

    CdpResponse::success(
        id,
        json!({
            "plugins": manifests,
            "count": manifests.len(),
        }),
    )
}

// ============================================================
// Accessibility domain
// ============================================================

/// Return a basic a11y tree derived from the SOM.
/// Each SOM region becomes a landmark node; each element becomes an AX node.
pub fn accessibility_get_full_ax_tree(id: u64, target: &CdpTarget) -> CdpResponse {
    use crate::som::types::RegionRole;

    let som = match &target.current_som {
        Some(s) => s,
        None => {
            return CdpResponse::success(id, json!({"nodes": []}));
        }
    };

    let mut nodes: Vec<serde_json::Value> = Vec::new();
    let mut next_id = 1u64;

    // Root node
    let root_id = next_id;
    next_id += 1;

    let mut root_children = Vec::new();

    for region in &som.regions {
        let region_ax_id = next_id;
        next_id += 1;
        root_children.push(json!({"nodeId": format!("{}", region_ax_id)}));

        let landmark_role = match region.role {
            RegionRole::Navigation => "navigation",
            RegionRole::Main => "main",
            RegionRole::Aside => "complementary",
            RegionRole::Header => "banner",
            RegionRole::Footer => "contentinfo",
            RegionRole::Form => "form",
            RegionRole::Dialog => "dialog",
            RegionRole::Content => "group",
        };

        let mut region_children = Vec::new();

        for element in &region.elements {
            let el_ax_id =
                push_ax_element_nodes(element, region_ax_id, target, &mut nodes, &mut next_id);
            region_children.push(json!({"nodeId": format!("{}", el_ax_id)}));
        }

        nodes.push(json!({
            "nodeId": format!("{}", region_ax_id),
            "ignored": false,
            "role": {"type": "role", "value": landmark_role},
            "name": {"type": "computedString", "value": region.label.as_deref().unwrap_or("")},
            "parentId": format!("{}", root_id),
            "childIds": region_children,
        }));
    }

    // Insert root node at the beginning
    nodes.insert(
        0,
        json!({
            "nodeId": format!("{}", root_id),
            "ignored": false,
            "role": {"type": "role", "value": "RootWebArea"},
            "name": {"type": "computedString", "value": som.title},
            "childIds": root_children,
        }),
    );

    CdpResponse::success(id, json!({"nodes": nodes}))
}

fn push_ax_element_nodes(
    element: &Element,
    parent_ax_id: u64,
    target: &CdpTarget,
    nodes: &mut Vec<serde_json::Value>,
    next_id: &mut u64,
) -> u64 {
    let el_ax_id = *next_id;
    *next_id += 1;

    let mut child_ids = Vec::new();
    if let Some(children) = &element.children {
        for child in children {
            let child_ax_id = push_ax_element_nodes(child, el_ax_id, target, nodes, next_id);
            child_ids.push(json!({"nodeId": format!("{}", child_ax_id)}));
        }
    }
    if let Some(shadow) = &element.shadow {
        for child in &shadow.elements {
            let child_ax_id = push_ax_element_nodes(child, el_ax_id, target, nodes, next_id);
            child_ids.push(json!({"nodeId": format!("{}", child_ax_id)}));
        }
    }

    let name = element
        .label
        .as_deref()
        .or(element.text.as_deref())
        .unwrap_or("");
    let (enabled, blocked_reason) = interactive_enabled_state(element);
    let mut properties = Vec::new();
    if !enabled {
        properties.push(json!({
            "name": "disabled",
            "value": {"type": "boolean", "value": true},
        }));
    }
    if blocked_reason == Some("readonly") {
        properties.push(json!({
            "name": "readonly",
            "value": {"type": "boolean", "value": true},
        }));
    }

    let mut node = json!({
        "nodeId": format!("{}", el_ax_id),
        "ignored": false,
        "role": {"type": "role", "value": element_role_to_ax_role(element)},
        "name": {"type": "computedString", "value": name},
        "parentId": format!("{}", parent_ax_id),
        "backendDOMNodeId": backend_node_id_for_element(target, element),
    });

    if !child_ids.is_empty() {
        node["childIds"] = json!(child_ids);
    }
    if !properties.is_empty() {
        node["properties"] = json!(properties);
    }

    nodes.push(node);
    el_ax_id
}

fn backend_node_id_for_element(target: &CdpTarget, element: &Element) -> u64 {
    target
        .node_map
        .iter()
        .find(|(_, entry)| entry.som_element_id.as_deref() == Some(&element.id))
        .map(|(_, entry)| entry.backend_node_id)
        .unwrap_or(0)
}

fn element_role_to_ax_role(element: &Element) -> &'static str {
    use crate::som::types::ElementRole;

    match element.role {
        ElementRole::Link => "link",
        ElementRole::Button => "button",
        ElementRole::TextInput => "textbox",
        ElementRole::Textarea => "textbox",
        ElementRole::Select => "combobox",
        ElementRole::Checkbox => "checkbox",
        ElementRole::Radio => "radio",
        ElementRole::Heading => "heading",
        ElementRole::Image => "img",
        ElementRole::List => "list",
        ElementRole::Table => "table",
        ElementRole::Paragraph => "paragraph",
        ElementRole::Section => "Section",
        ElementRole::Group => "group",
        ElementRole::Separator => "separator",
        ElementRole::Details => "group",
        ElementRole::Iframe => "Iframe",
    }
}

// ============================================================
// Helpers
// ============================================================

fn node_to_cdp(entry: &NodeEntry, target: &CdpTarget, depth: u32) -> serde_json::Value {
    let mut node = json!({
        "nodeId": entry.node_id,
        "backendNodeId": entry.backend_node_id,
        "nodeType": entry.node_type,
        "nodeName": entry.node_name,
        "localName": entry.node_name.to_lowercase(),
        "nodeValue": entry.node_value,
    });

    if let Some(attributes) = node_attributes(entry, target) {
        node["attributes"] = json!(attributes);
    }

    if depth > 0 && !entry.children_ids.is_empty() {
        let children: Vec<serde_json::Value> = entry
            .children_ids
            .iter()
            .filter_map(|cid| target.node_map.get(cid))
            .map(|c| node_to_cdp(c, target, depth - 1))
            .collect();
        node["childNodeCount"] = json!(children.len());
        node["children"] = json!(children);
    } else {
        node["childNodeCount"] = json!(entry.children_ids.len());
    }

    node
}

fn node_attributes(entry: &NodeEntry, target: &CdpTarget) -> Option<Vec<String>> {
    let som_id = entry.som_element_id.as_deref()?;
    let element = target.find_element_by_som_id(som_id)?;
    let mut attrs = Vec::new();
    let element_attrs = element.attrs.as_ref();

    push_attr(&mut attrs, "data-plasmate-id", Some(element.id.as_str()));
    push_attr(&mut attrs, "data-som-role", Some(element.role.as_str()));
    push_attr(&mut attrs, "id", element.html_id.as_deref());
    push_attr(&mut attrs, "aria-label", element.label.as_deref());
    push_attr(
        &mut attrs,
        "aria-labelledby",
        attr_aria_string(element_attrs, "labelledby"),
    );
    push_attr(
        &mut attrs,
        "aria-describedby",
        attr_aria_string(element_attrs, "describedby"),
    );
    push_attr(&mut attrs, "data-testid", element_test_id(element));

    push_attr(&mut attrs, "href", attr_string(element_attrs, "href"));
    push_attr(&mut attrs, "name", attr_string(element_attrs, "name"));
    push_attr(
        &mut attrs,
        "placeholder",
        attr_string(element_attrs, "placeholder"),
    );
    push_attr(&mut attrs, "title", attr_string(element_attrs, "title"));
    push_attr(
        &mut attrs,
        "type",
        attr_string(element_attrs, "type")
            .or_else(|| attr_string(element_attrs, "input_type"))
            .or_else(|| attr_string(element_attrs, "button_type")),
    );
    push_bool_attr(
        &mut attrs,
        "disabled",
        attr_bool_opt(element_attrs, "disabled"),
    );
    push_bool_attr(
        &mut attrs,
        "readonly",
        attr_bool_opt(element_attrs, "readonly"),
    );
    push_bool_attr(
        &mut attrs,
        "required",
        attr_bool_opt(element_attrs, "required"),
    );

    if attrs.is_empty() {
        None
    } else {
        Some(attrs)
    }
}

fn push_attr(out: &mut Vec<String>, name: &str, value: Option<&str>) {
    if let Some(value) = value.filter(|value| !value.is_empty()) {
        out.push(name.to_string());
        out.push(value.to_string());
    }
}

fn push_bool_attr(out: &mut Vec<String>, name: &str, value: bool) {
    if value {
        out.push(name.to_string());
        out.push("true".to_string());
    }
}

fn attr_bool_opt(attrs: Option<&serde_json::Value>, key: &str) -> bool {
    attrs
        .and_then(|attrs| attrs.get(key))
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

fn interactive_elements(som: &Som) -> Vec<&Element> {
    let mut elements = Vec::new();
    for region in &som.regions {
        collect_interactive_elements(&region.elements, &mut elements);
    }
    elements
}

fn collect_interactive_elements<'a>(source: &'a [Element], out: &mut Vec<&'a Element>) {
    for element in source {
        if element.role.is_interactive() {
            out.push(element);
        }
        if let Some(children) = &element.children {
            collect_interactive_elements(children, out);
        }
        if let Some(shadow) = &element.shadow {
            collect_interactive_elements(&shadow.elements, out);
        }
    }
}

fn interactive_element_to_json(element: &Element) -> serde_json::Value {
    let (enabled, blocked_reason) = interactive_enabled_state(element);
    let test_id = element_test_id(element);

    json!({
        "id": element.id,
        "role": element.role.as_str(),
        "cache_key": action_plan_cache_key(element),
        "html_id": element.html_id,
        "test_id": test_id,
        "text": element.text,
        "label": element.label,
        "actions": element.actions,
        "attrs": element.attrs,
        "hints": element.hints,
        "enabled": enabled,
        "blocked_reason": blocked_reason,
    })
}

fn element_matches_lookup(element: &Element, value: &str, by: &str, exact_label: bool) -> bool {
    match by {
        "id" => element.id == value,
        "cache_key" | "cacheKey" => action_plan_cache_key(element) == value,
        "html_id" | "htmlId" => element.html_id.as_deref() == Some(value),
        "test_id" | "testId" => element_test_id(element) == Some(value),
        "label" => element_label_matches(element, value, exact_label || by == "label"),
        _ => {
            element.id == value
                || action_plan_cache_key(element) == value
                || element.html_id.as_deref() == Some(value)
                || element_test_id(element) == Some(value)
        }
    }
}

fn element_matches_role_filter(element: &Element, role: &str) -> bool {
    let normalized = normalize_role_filter(role);
    match normalized.as_str() {
        "textbox" => matches!(
            element.role,
            crate::som::types::ElementRole::TextInput | crate::som::types::ElementRole::Textarea
        ),
        "combobox" | "listbox" => element.role.as_str() == "select",
        "img" => element.role.as_str() == "image",
        "input" => matches!(
            element.role,
            crate::som::types::ElementRole::TextInput
                | crate::som::types::ElementRole::Checkbox
                | crate::som::types::ElementRole::Radio
        ),
        other => element.role.as_str() == other,
    }
}

fn normalize_role_filter(role: &str) -> String {
    role.trim().to_ascii_lowercase().replace(['-', ' '], "_")
}

fn element_label_matches(element: &Element, label: &str, exact: bool) -> bool {
    let element_label = element
        .label
        .as_deref()
        .or(element.text.as_deref())
        .unwrap_or("");
    if exact {
        element_label == label
    } else {
        element_label.to_lowercase().contains(&label.to_lowercase())
    }
}

fn element_test_id(element: &Element) -> Option<&str> {
    element
        .attrs
        .as_ref()
        .and_then(|attrs| attrs.get("test_id"))
        .and_then(|value| value.as_str())
}

fn action_plan_cache_key(element: &Element) -> String {
    let mut actions = element.actions.clone().unwrap_or_default();
    actions.sort();
    let action_value = if actions.is_empty() {
        None
    } else {
        Some(actions.join(","))
    };
    let attrs = element.attrs.as_ref();
    let parts = vec![
        Some(element.id.clone()),
        Some(element.role.as_str().to_string()),
        element.label.clone(),
        action_value,
        attr_string(attrs, "name").map(str::to_string),
        attr_string(attrs, "href").map(str::to_string),
        attr_string(attrs, "input_type").map(str::to_string),
        attr_string(attrs, "group").map(str::to_string),
        attr_string(attrs, "placeholder").map(str::to_string),
    ];
    let encoded = serde_json::to_string(&parts).unwrap_or_else(|_| "[]".to_string());
    format!("plasmate-action:v1:{}", fnv1a32(&encoded))
}

fn attr_string<'a>(attrs: Option<&'a serde_json::Value>, key: &str) -> Option<&'a str> {
    attrs
        .and_then(|attrs| attrs.get(key))
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())
}

fn attr_aria_string<'a>(attrs: Option<&'a serde_json::Value>, key: &str) -> Option<&'a str> {
    attrs
        .and_then(|attrs| attrs.get("aria"))
        .and_then(|aria| aria.get(key))
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())
}

fn fnv1a32(value: &str) -> String {
    let mut hash = 0x811c9dc5u32;
    for byte in value.as_bytes() {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(0x01000193);
    }
    format!("{hash:08x}")
}

fn interactive_enabled_state(element: &Element) -> (bool, Option<&'static str>) {
    let Some(attrs) = &element.attrs else {
        return (true, None);
    };

    if attr_bool(attrs, "disabled") {
        return (false, Some("disabled"));
    }
    if attr_bool(attrs, "inert") {
        return (false, Some("inert"));
    }
    if attr_bool(attrs, "readonly") {
        return (false, Some("readonly"));
    }

    (true, None)
}

fn attr_bool(attrs: &serde_json::Value, key: &str) -> bool {
    attrs
        .get(key)
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

fn timestamp_sec() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

/// Convert SOM to markdown for Lightpanda-compatible output.
fn som_to_markdown(som: &crate::som::types::Som) -> String {
    use crate::som::types::ElementRole;
    let mut md = String::new();

    if !som.title.is_empty() {
        md.push_str(&format!("# {}\n\n", som.title));
    }

    for region in &som.regions {
        for element in &region.elements {
            match &element.role {
                ElementRole::Heading => {
                    let text = element.text.as_deref().unwrap_or("");
                    md.push_str(&format!("## {}\n\n", text));
                }
                ElementRole::Paragraph => {
                    if let Some(text) = &element.text {
                        md.push_str(text);
                        md.push_str("\n\n");
                    }
                }
                ElementRole::Link => {
                    let text = element.text.as_deref().unwrap_or("link");
                    let href = element
                        .attrs
                        .as_ref()
                        .and_then(|a| a.get("href"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("#");
                    md.push_str(&format!("[{}]({})\n", text, href));
                }
                ElementRole::List => {
                    let text = element.text.as_deref().unwrap_or("");
                    md.push_str(&format!("- {}\n", text));
                }
                ElementRole::Image => {
                    let alt = element.label.as_deref().unwrap_or("image");
                    let src = element
                        .attrs
                        .as_ref()
                        .and_then(|a| a.get("src"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    md.push_str(&format!("![{}]({})\n", alt, src));
                }
                _ => {
                    if let Some(text) = &element.text {
                        if !text.is_empty() {
                            md.push_str(text);
                            md.push('\n');
                        }
                    }
                }
            }
        }
    }

    md
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::som::types::{ElementRole, Region, RegionRole, ShadowRoot, SomMeta};

    fn element(id: &str, role: ElementRole, label: Option<&str>) -> Element {
        Element {
            id: id.to_string(),
            role,
            html_id: None,
            text: None,
            label: label.map(str::to_string),
            actions: None,
            attrs: None,
            children: None,
            hints: None,
            shadow: None,
        }
    }

    fn cdp_target_with_som(som: Som) -> CdpTarget {
        let mut target = CdpTarget::new().unwrap();
        target.current_som = Some(som);
        target.rebuild_node_map();
        target
    }

    fn fixture_som() -> Som {
        let mut top_button = element("top-button", ElementRole::Button, Some("Save"));
        top_button.actions = Some(vec!["click".to_string()]);
        top_button.html_id = Some("save-button".to_string());
        top_button.attrs = Some(json!({
            "test_id": "settings-save",
            "disabled": true,
            "aria": {
                "labelledby": "save-label",
                "describedby": "save-help"
            },
        }));

        let mut text_input = element("email-input", ElementRole::TextInput, Some("Email"));
        text_input.actions = Some(vec!["type".to_string(), "clear".to_string()]);

        let mut nested_link = element("nested-link", ElementRole::Link, Some("Docs"));
        nested_link.actions = Some(vec!["click".to_string()]);

        let mut shadow_button = element("shadow-button", ElementRole::Button, Some("Shadow save"));
        shadow_button.actions = Some(vec!["click".to_string()]);

        let mut host = element("host", ElementRole::Section, Some("Host"));
        host.children = Some(vec![nested_link]);
        host.shadow = Some(ShadowRoot {
            mode: "open".to_string(),
            elements: vec![shadow_button],
        });

        Som {
            som_version: "0.1".to_string(),
            url: "https://example.com/app".to_string(),
            title: "Settings".to_string(),
            lang: "en".to_string(),
            regions: vec![Region {
                id: "main".to_string(),
                role: RegionRole::Main,
                label: None,
                action: None,
                method: None,
                target: None,
                enctype: None,
                novalidate: None,
                accept_charset: None,
                autocomplete: None,
                elements: vec![top_button, text_input, host],
            }],
            meta: SomMeta {
                html_bytes: 100,
                som_bytes: 100,
                element_count: 5,
                interactive_count: 4,
            },
            structured_data: None,
        }
    }

    #[test]
    fn get_interactive_elements_includes_nested_shadow_and_serialized_roles() {
        let target = cdp_target_with_som(fixture_som());
        let response = plasmate_get_interactive_elements(1, &json!({}), &target);
        let result = response.result.unwrap();
        let elements = result["elements"].as_array().unwrap();
        let ids: Vec<&str> = elements
            .iter()
            .map(|element| element["id"].as_str().unwrap())
            .collect();

        assert_eq!(result["count"], 4);
        assert_eq!(result["filtered_count"], 4);
        assert_eq!(result["total_interactive_count"], 4);
        assert!(ids.contains(&"top-button"));
        assert!(ids.contains(&"email-input"));
        assert!(ids.contains(&"nested-link"));
        assert!(ids.contains(&"shadow-button"));
        assert_eq!(
            elements
                .iter()
                .find(|element| element["id"] == "email-input")
                .unwrap()["role"],
            "text_input"
        );
        assert!(elements.iter().all(|element| element["cache_key"]
            .as_str()
            .is_some_and(|key| key.starts_with("plasmate-action:v1:"))));
    }

    #[test]
    fn dom_nodes_expose_replay_attributes() {
        let target = cdp_target_with_som(fixture_som());
        let node_id = target.query_selector("#save-button").unwrap();
        let entry = target.node_map.get(&node_id).unwrap();
        let node = node_to_cdp(entry, &target, 0);
        let attrs = node["attributes"].as_array().unwrap();
        let attr = |name: &str| {
            attrs
                .chunks(2)
                .find(|pair| pair[0].as_str() == Some(name))
                .and_then(|pair| pair[1].as_str())
        };

        assert_eq!(attr("data-plasmate-id"), Some("top-button"));
        assert_eq!(attr("data-som-role"), Some("button"));
        assert_eq!(attr("id"), Some("save-button"));
        assert_eq!(attr("aria-label"), Some("Save"));
        assert_eq!(attr("aria-labelledby"), Some("save-label"));
        assert_eq!(attr("aria-describedby"), Some("save-help"));
        assert_eq!(attr("data-testid"), Some("settings-save"));
        assert_eq!(attr("disabled"), Some("true"));
    }

    #[test]
    fn get_interactive_elements_filters_by_action_label_and_enabled_state() {
        let target = cdp_target_with_som(fixture_som());
        let response = plasmate_get_interactive_elements(
            1,
            &json!({
                "action": "click",
                "label": "save",
                "enabledOnly": true,
            }),
            &target,
        );
        let result = response.result.unwrap();
        let elements = result["elements"].as_array().unwrap();

        assert_eq!(result["count"], 1);
        assert_eq!(elements[0]["id"], "shadow-button");
        assert_eq!(elements[0]["enabled"], true);
        assert!(elements[0]["blocked_reason"].is_null());

        let blocked = plasmate_get_interactive_elements(
            2,
            &json!({
                "role": "button",
                "label": "Save",
                "exact": true,
            }),
            &target,
        )
        .result
        .unwrap();
        let blocked_elements = blocked["elements"].as_array().unwrap();
        assert_eq!(blocked_elements.len(), 1);
        assert_eq!(blocked_elements[0]["id"], "top-button");
        assert_eq!(blocked_elements[0]["html_id"], "save-button");
        assert_eq!(blocked_elements[0]["test_id"], "settings-save");
        assert_eq!(blocked_elements[0]["enabled"], false);
        assert_eq!(blocked_elements[0]["blocked_reason"], "disabled");

        let textbox = plasmate_get_interactive_elements(
            3,
            &json!({
                "role": "textbox",
            }),
            &target,
        )
        .result
        .unwrap();
        assert_eq!(textbox["count"], 1);
        assert_eq!(textbox["elements"][0]["id"], "email-input");
    }

    #[test]
    fn get_interactive_elements_resolves_replay_identifiers() {
        let target = cdp_target_with_som(fixture_som());
        let all = plasmate_get_interactive_elements(1, &json!({}), &target)
            .result
            .unwrap();
        let elements = all["elements"].as_array().unwrap();
        let save = elements
            .iter()
            .find(|element| element["id"] == "top-button")
            .unwrap();
        let cache_key = save["cache_key"].as_str().unwrap();

        let by_cache_key = plasmate_get_interactive_elements(
            2,
            &json!({
                "value": cache_key,
                "by": "cache_key",
            }),
            &target,
        )
        .result
        .unwrap();
        assert_eq!(by_cache_key["count"], 1);
        assert_eq!(by_cache_key["elements"][0]["id"], "top-button");

        let by_auto = plasmate_get_interactive_elements(
            3,
            &json!({
                "value": "settings-save",
            }),
            &target,
        )
        .result
        .unwrap();
        assert_eq!(by_auto["count"], 1);
        assert_eq!(by_auto["elements"][0]["id"], "top-button");

        let by_label = plasmate_get_interactive_elements(
            4,
            &json!({
                "value": "Shadow save",
                "by": "label",
            }),
            &target,
        )
        .result
        .unwrap();
        assert_eq!(by_label["count"], 1);
        assert_eq!(by_label["elements"][0]["id"], "shadow-button");
    }

    #[test]
    fn get_interactive_elements_pages_filtered_results() {
        let target = cdp_target_with_som(fixture_som());
        let response = plasmate_get_interactive_elements(
            1,
            &json!({
                "action": "click",
                "offset": 1,
                "limit": 1,
            }),
            &target,
        );
        let result = response.result.unwrap();
        let elements = result["elements"].as_array().unwrap();

        assert_eq!(result["count"], 1);
        assert_eq!(result["filtered_count"], 3);
        assert_eq!(result["total_interactive_count"], 4);
        assert_eq!(result["offset"], 1);
        assert_eq!(result["limit"], 1);
        assert_eq!(elements[0]["id"], "nested-link");
    }

    #[test]
    fn get_full_ax_tree_includes_nested_shadow_and_availability_properties() {
        let target = cdp_target_with_som(fixture_som());
        let response = accessibility_get_full_ax_tree(1, &target);
        let result = response.result.unwrap();
        let nodes = result["nodes"].as_array().unwrap();

        let node_by_name = |name: &str| {
            nodes
                .iter()
                .find(|node| node["name"]["value"].as_str() == Some(name))
                .unwrap_or_else(|| panic!("missing AX node named {name}"))
        };

        let host = node_by_name("Host");
        let nested = node_by_name("Docs");
        let shadow = node_by_name("Shadow save");
        let disabled = node_by_name("Save");

        assert_eq!(nested["parentId"], host["nodeId"]);
        assert_eq!(shadow["parentId"], host["nodeId"]);
        assert!(nested["backendDOMNodeId"].as_u64().unwrap() > 0);
        assert!(shadow["backendDOMNodeId"].as_u64().unwrap() > 0);
        assert_eq!(disabled["properties"][0]["name"], "disabled");
    }
}
