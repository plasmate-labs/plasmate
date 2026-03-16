//! CDP domain handlers.
//!
//! Each function handles one CDP method, translating it to our SOM pipeline.
//! The goal: Puppeteer/Playwright connect and work. Under the hood, everything
//! goes through Plasmate's engine - agents get speed + token efficiency for free.

use serde_json::json;
use tracing::{info, warn};

use super::session::{CdpTarget, NodeEntry};
use super::types::*;
use crate::som::types::ElementRole;

// ============================================================
// Browser domain
// ============================================================

pub fn browser_get_version(id: u64) -> CdpResponse {
    CdpResponse::success(
        id,
        json!({
            "protocolVersion": "1.3",
            "product": "Plasmate/0.1.0",
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
            let events = vec![
                CdpEvent::new(
                    "Page.frameStartedLoading",
                    json!({"frameId": result.frame_id}),
                ),
                CdpEvent::new(
                    "Page.frameNavigated",
                    json!({
                        "frame": {
                            "id": result.frame_id,
                            "loaderId": result.loader_id,
                            "url": result.url,
                            "securityOrigin": result.url,
                            "mimeType": "text/html",
                        }
                    }),
                ),
                CdpEvent::new("Page.loadEventFired", json!({"timestamp": timestamp_sec()})),
                CdpEvent::new(
                    "Page.lifecycleEvent",
                    json!({
                        "frameId": result.frame_id,
                        "loaderId": result.loader_id,
                        "name": "load",
                        "timestamp": timestamp_sec(),
                    }),
                ),
                CdpEvent::new(
                    "Page.frameStoppedLoading",
                    json!({"frameId": result.frame_id}),
                ),
            ];

            (
                CdpResponse::success(
                    id,
                    json!({
                        "frameId": result.frame_id,
                        "loaderId": result.loader_id,
                    }),
                ),
                events,
            )
        }
        Err(e) => (CdpResponse::error(id, CDP_ERR_SERVER, &e), vec![]),
    }
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

    // Common Puppeteer evaluations we can handle
    match expression {
        // document.title
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
        // document.URL or window.location.href
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
        // document.querySelectorAll('a') - return links
        e if e.contains("querySelectorAll") && e.contains("'a'") => {
            let links: Vec<String> = if let Some(som) = &target.current_som {
                som.regions
                    .iter()
                    .flat_map(|r| &r.elements)
                    .filter(|el| el.role == ElementRole::Link)
                    .filter_map(|el| el.attrs.as_ref()?.get("href")?.as_str().map(String::from))
                    .collect()
            } else {
                vec![]
            };

            if return_by_value {
                CdpResponse::success(
                    id,
                    json!({
                        "result": {
                            "type": "object",
                            "subtype": "array",
                            "value": links,
                        }
                    }),
                )
            } else {
                CdpResponse::success(
                    id,
                    json!({
                        "result": {
                            "type": "object",
                            "subtype": "array",
                            "description": format!("Array({})", links.len()),
                            "objectId": "links-result",
                        }
                    }),
                )
            }
        }
        // Generic: try to evaluate in V8 if we have a runtime
        _ => {
            warn!(
                "CDP Runtime.evaluate not fully supported: {}",
                &expression[..expression.len().min(100)]
            );
            CdpResponse::success(
                id,
                json!({
                    "result": {
                        "type": "undefined",
                    }
                }),
            )
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

pub fn network_get_cookies(id: u64) -> CdpResponse {
    // Cookie jar doesn't expose cookies easily; return empty for now
    CdpResponse::success(id, json!({"cookies": []}))
}

pub fn network_set_cookies(id: u64) -> CdpResponse {
    CdpResponse::success(id, json!({}))
}

// ============================================================
// Emulation domain (Puppeteer needs these)
// ============================================================

pub fn emulation_set_device_metrics_override(id: u64) -> CdpResponse {
    CdpResponse::success(id, json!({}))
}

pub fn emulation_set_touch_emulation_enabled(id: u64) -> CdpResponse {
    CdpResponse::success(id, json!({}))
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

pub fn plasmate_get_interactive_elements(id: u64, target: &CdpTarget) -> CdpResponse {
    let elements: Vec<serde_json::Value> = if let Some(som) = &target.current_som {
        som.regions
            .iter()
            .flat_map(|r| &r.elements)
            .filter(|e| e.role.is_interactive())
            .map(|e| {
                json!({
                    "id": e.id,
                    "role": format!("{:?}", e.role).to_lowercase(),
                    "text": e.text,
                    "label": e.label,
                    "actions": e.actions,
                    "attrs": e.attrs,
                    "hints": e.hints,
                })
            })
            .collect()
    } else {
        vec![]
    };

    CdpResponse::success(
        id,
        json!({
            "elements": elements,
            "count": elements.len(),
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
