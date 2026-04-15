use std::sync::Arc;

use tokio::sync::Mutex;

use regex::Regex;
use serde_json::json;
use tracing::{info, warn};

use super::messages::{ErrorCode, Response};
use super::session::Session;
use crate::cdp::cookies::{cookie_from_cdp_params, Cookie};
use crate::network::intercept::{
    ErrorReason, FulfillParams, InterceptAction, InterceptRule, RequestOverrides, RequestPattern,
    RequestStage, ResourceType, ResponseOverrides, ResponseRule,
};
use crate::network::tls::TlsConfig;
use crate::plugin::PluginManager;
use crate::som::types::{Element, ElementRole};

/// Shared plugin manager handle (thread-safe, optional).
pub type SharedPlugins = Option<Arc<Mutex<PluginManager>>>;

/// Connection state tracked per WebSocket connection.
pub struct ConnectionState {
    pub handshake_done: bool,
    /// Active sessions (multiple per connection supported).
    pub sessions: std::collections::HashMap<String, Session>,
    /// Legacy single-session field (backward compatibility).
    pub session: Option<Session>,
    pub plugins: SharedPlugins,
}

impl ConnectionState {
    pub fn new(plugins: SharedPlugins) -> Self {
        ConnectionState {
            handshake_done: false,
            sessions: std::collections::HashMap::new(),
            session: None,
            plugins,
        }
    }

    /// Look up a session by ID (immutable).
    pub fn get_session(&self, session_id: &str) -> Option<&Session> {
        self.sessions
            .get(session_id)
            .or_else(|| self.session.as_ref().filter(|s| s.id == session_id))
    }

    /// Look up a session by ID (mutable).
    pub fn get_session_mut(&mut self, session_id: &str) -> Option<&mut Session> {
        if self.sessions.contains_key(session_id) {
            self.sessions.get_mut(session_id)
        } else {
            self.session.as_mut().filter(|s| s.id == session_id)
        }
    }
}

/// Dispatch a request to the appropriate handler.
pub async fn handle_request(
    id: &str,
    method: &str,
    params: &serde_json::Value,
    state: &mut ConnectionState,
) -> Response {
    // Enforce handshake-first rule
    if !state.handshake_done && method != "awp.hello" {
        return Response::error(
            id,
            ErrorCode::InvalidRequest,
            "Must send awp.hello before any other method",
        );
    }

    match method {
        "awp.hello" => handle_hello(id, params, state),
        "session.create" => handle_session_create(id, params, state),
        "session.close" => handle_session_close(id, params, state),
        "session.list" => handle_session_list(id, state),
        "page.navigate" => handle_page_navigate(id, params, state).await,
        "page.observe" => handle_page_observe(id, params, state),
        "page.act" => handle_page_act(id, params, state).await,
        "page.extract" => handle_page_extract(id, params, state),
        "network.enableInterception" => handle_network_enable_interception(id, params, state),
        "network.disableInterception" => handle_network_disable_interception(id, params, state),
        "network.addRule" => handle_network_add_rule(id, params, state),
        "network.removeRule" => handle_network_remove_rule(id, params, state),
        "network.clearRules" => handle_network_clear_rules(id, params, state),
        "network.getInterceptedRequests" => {
            handle_network_get_intercepted_requests(id, params, state)
        }
        "plugin.list" => handle_plugin_list(id, state).await,
        "session.cookies.get" => handle_cookies_get(id, params, state),
        "session.cookies.set" => handle_cookies_set(id, params, state),
        "session.cookies.clear" => handle_cookies_clear(id, params, state),
        _ => Response::error(
            id,
            ErrorCode::InvalidRequest,
            &format!("Unknown method: {}", method),
        ),
    }
}

fn handle_hello(id: &str, params: &serde_json::Value, state: &mut ConnectionState) -> Response {
    let awp_version = params
        .get("awp_version")
        .and_then(|v| v.as_str())
        .unwrap_or("0.1");

    if awp_version != "0.1" {
        return Response::error(
            id,
            ErrorCode::InvalidRequest,
            &format!("Unsupported AWP version: {}", awp_version),
        );
    }

    state.handshake_done = true;
    let client_name = params
        .get("client_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    info!(client_name, "AWP handshake completed");

    let mut features = vec![
        "som.snapshot",
        "act.primitive",
        "extract",
        "network.intercept",
        "cookies",
    ];
    if state.plugins.is_some() {
        features.push("plugins");
    }

    Response::success(
        id,
        json!({
            "awp_version": "0.1",
            "server_name": "plasmate",
            "server_version": env!("CARGO_PKG_VERSION"),
            "features": features
        }),
    )
}

fn handle_session_create(
    id: &str,
    params: &serde_json::Value,
    state: &mut ConnectionState,
) -> Response {
    // Check session limit (max 50 per connection)
    let total = state.sessions.len() + if state.session.is_some() { 1 } else { 0 };
    if total >= 50 {
        return Response::error(id, ErrorCode::Internal, "Maximum sessions (50) reached");
    }

    let session_id = format!("s_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let user_agent = params
        .get("user_agent")
        .and_then(|v| v.as_str())
        .map(String::from);
    let locale = params
        .get("locale")
        .and_then(|v| v.as_str())
        .map(String::from);
    let timeout_ms = params.get("timeout_ms").and_then(|v| v.as_u64());

    // Parse per-session TLS configuration from params
    let tls_config = parse_tls_params(params);

    match Session::new(
        session_id.clone(),
        user_agent,
        locale,
        timeout_ms,
        tls_config,
    ) {
        Ok(session) => {
            info!(session_id = %session.id, "Session created");
            if state.session.is_none() && state.sessions.is_empty() {
                state.session = Some(session);
            } else {
                state.sessions.insert(session_id.clone(), session);
            }
            Response::success(id, json!({"session_id": session_id}))
        }
        Err(e) => Response::error(id, ErrorCode::Internal, &e),
    }
}

fn handle_session_close(
    id: &str,
    params: &serde_json::Value,
    state: &mut ConnectionState,
) -> Response {
    let requested_id = params
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if state.sessions.remove(requested_id).is_some() {
        info!(session_id = requested_id, "Session closed");
        return Response::success(id, json!({"closed": true}));
    }

    match &state.session {
        Some(session) if session.id == requested_id => {
            info!(session_id = requested_id, "Session closed");
            state.session = None;
            Response::success(id, json!({"closed": true}))
        }
        _ => Response::error(
            id,
            ErrorCode::NotFound,
            &format!("Session not found: {}", requested_id),
        ),
    }
}

fn handle_session_list(id: &str, state: &ConnectionState) -> Response {
    let mut infos = Vec::new();
    if let Some(ref s) = state.session {
        infos.push(
            json!({"session_id": s.id, "url": s.current_url, "page_count": s.page_count,
            "uptime_ms": s.created_at.elapsed().as_millis() as u64}),
        );
    }
    for s in state.sessions.values() {
        infos.push(
            json!({"session_id": s.id, "url": s.current_url, "page_count": s.page_count,
            "uptime_ms": s.created_at.elapsed().as_millis() as u64}),
        );
    }
    Response::success(id, json!({"sessions": infos, "count": infos.len()}))
}

async fn handle_page_navigate(
    id: &str,
    params: &serde_json::Value,
    state: &mut ConnectionState,
) -> Response {
    let session_id = params
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let url = match params.get("url").and_then(|v| v.as_str()) {
        Some(u) => u,
        None => {
            return Response::error(id, ErrorCode::InvalidRequest, "Missing required param: url")
        }
    };

    // Clone plugins before borrowing session mutably (avoids borrow conflict)
    let plugins = state.plugins.clone();
    let effective_url = if let Some(ref pm) = plugins {
        let mut pm = pm.lock().await;
        pm.run_pre_navigate(url).unwrap_or_else(|_| url.to_string())
    } else {
        url.to_string()
    };

    let session = match state.get_session_mut(session_id) {
        Some(s) => s,
        None => {
            return Response::error(
                id,
                ErrorCode::NotFound,
                &format!("Session not found: {}", session_id),
            )
        }
    };

    info!(url = %effective_url, "Navigating (full pipeline)");

    match session
        .navigate_with_plugins(&effective_url, &plugins)
        .await
    {
        Ok(result) => {
            let mut response = json!({
                "url": result.url,
                "status": result.status,
                "content_type": result.content_type,
                "title": result.title,
                "html_bytes": result.html_bytes,
                "som_bytes": result.som_bytes,
                "som_ready": true,
                "fetch_ms": result.fetch_ms,
                "pipeline_ms": result.pipeline_ms,
            });

            if let Some(js) = &result.js_report {
                response["js"] = json!({
                    "scripts_total": js.total,
                    "scripts_ok": js.succeeded,
                    "scripts_err": js.failed,
                });
            }

            Response::success(id, response)
        }
        Err(e) => {
            if e.contains("Timeout") {
                Response::error(id, ErrorCode::Timeout, &e)
            } else {
                Response::error(id, ErrorCode::NavigationFailed, &e)
            }
        }
    }
}

fn handle_page_observe(
    id: &str,
    params: &serde_json::Value,
    state: &mut ConnectionState,
) -> Response {
    let session_id = params
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let session = match state.get_session(session_id) {
        Some(s) => s,
        None => {
            return Response::error(
                id,
                ErrorCode::NotFound,
                &format!("Session not found: {}", session_id),
            )
        }
    };

    match &session.current_som {
        Some(som) => {
            let som_json = serde_json::to_value(som).unwrap_or(json!(null));
            Response::success(id, json!({"som": som_json}))
        }
        None => Response::error(id, ErrorCode::NotFound, "No page loaded yet"),
    }
}

async fn handle_page_act(
    id: &str,
    params: &serde_json::Value,
    state: &mut ConnectionState,
) -> Response {
    let session_id = params
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let session = match state.get_session_mut(session_id) {
        Some(s) => s,
        None => {
            return Response::error(
                id,
                ErrorCode::NotFound,
                &format!("Session not found: {}", session_id),
            )
        }
    };

    let intent = match params.get("intent") {
        Some(i) => i,
        None => {
            return Response::error(
                id,
                ErrorCode::InvalidRequest,
                "Missing required param: intent",
            )
        }
    };

    let action = intent.get("action").and_then(|v| v.as_str()).unwrap_or("");
    let target = match intent.get("target") {
        Some(t) => t,
        None => {
            return Response::error(
                id,
                ErrorCode::InvalidRequest,
                "Missing required param: intent.target",
            )
        }
    };

    let som = match &session.current_som {
        Some(s) => s,
        None => return Response::error(id, ErrorCode::NotFound, "No page loaded yet"),
    };

    // Resolve target
    let resolved = resolve_target(target, som);
    let element = match resolved {
        Some(el) => el,
        None => {
            return Response::error_with_details(
                id,
                ErrorCode::NotFound,
                "Target element not found",
                json!({"target": target}),
            )
        }
    };

    match action {
        "click" => {
            let mut navigated = false;
            let mut som_changed = false;

            // If it's a link, navigate to href using full pipeline
            if element.role == ElementRole::Link {
                if let Some(attrs) = &element.attrs {
                    if let Some(href) = attrs.get("href").and_then(|v| v.as_str()) {
                        // Resolve relative URL
                        let base_url = session.current_url.as_deref().unwrap_or("");
                        let resolved_url = resolve_url(base_url, href);

                        // Use full pipeline (JS + external scripts + structured data)
                        match session.navigate(&resolved_url).await {
                            Ok(_) => {
                                navigated = true;
                                som_changed = true;
                            }
                            Err(e) => {
                                warn!("Click navigation failed: {}", e);
                            }
                        }
                    }
                }
            }

            Response::success(
                id,
                json!({
                    "status": "ok",
                    "resolved": {
                        "element_id": element.id,
                        "role": element.role,
                        "text": element.text
                    },
                    "effects": {
                        "navigated": navigated,
                        "som_changed": som_changed
                    }
                }),
            )
        }
        "type" => {
            let value = intent.get("value").and_then(|v| v.as_str()).unwrap_or("");

            // Update element value in SOM
            if let Some(som) = &mut session.current_som {
                update_element_value(som, &element.id, value);
            }

            Response::success(
                id,
                json!({
                    "status": "ok",
                    "resolved": {
                        "element_id": element.id,
                        "role": element.role,
                        "text": element.text
                    },
                    "effects": {
                        "navigated": false,
                        "som_changed": true
                    }
                }),
            )
        }
        "select" => {
            let value = intent.get("value").and_then(|v| v.as_str()).unwrap_or("");

            if let Some(som) = &mut session.current_som {
                update_element_value(som, &element.id, value);
            }

            Response::success(
                id,
                json!({
                    "status": "ok",
                    "resolved": {
                        "element_id": element.id,
                        "role": element.role,
                        "text": element.text
                    },
                    "effects": {
                        "navigated": false,
                        "som_changed": true
                    }
                }),
            )
        }
        "scroll" => {
            let direction = intent
                .get("value")
                .and_then(|v| v.as_str())
                .unwrap_or("down");

            let delta: i64 = match direction {
                "up" => -300,
                "down" => 300,
                "top" => {
                    // Reset to 0
                    session.scroll_y = Some(0);
                    0
                }
                "bottom" => {
                    // Set a large value to represent bottom
                    session.scroll_y = Some(i64::MAX);
                    0
                }
                _ => 300,
            };

            if direction != "top" && direction != "bottom" {
                let current = session.scroll_y.unwrap_or(0);
                let new_y = (current + delta).max(0);
                session.scroll_y = Some(new_y);
            }

            Response::success(
                id,
                json!({
                    "status": "ok",
                    "resolved": {
                        "element_id": element.id,
                        "role": element.role,
                        "text": element.text
                    },
                    "effects": {
                        "navigated": false,
                        "som_changed": false,
                        "scroll_y": session.scroll_y
                    }
                }),
            )
        }
        "toggle" => {
            // Toggle checkbox/radio checked state or details open state in SOM
            if let Some(som) = &mut session.current_som {
                toggle_element(som, &element.id);
            }

            Response::success(
                id,
                json!({
                    "status": "ok",
                    "resolved": {
                        "element_id": element.id,
                        "role": element.role,
                        "text": element.text
                    },
                    "effects": {
                        "navigated": false,
                        "som_changed": true
                    }
                }),
            )
        }
        "clear" => {
            // Clear the element value in SOM
            if let Some(som) = &mut session.current_som {
                update_element_value(som, &element.id, "");
            }

            Response::success(
                id,
                json!({
                    "status": "ok",
                    "resolved": {
                        "element_id": element.id,
                        "role": element.role,
                        "text": element.text
                    },
                    "effects": {
                        "navigated": false,
                        "som_changed": true
                    }
                }),
            )
        }
        _ => Response::error(
            id,
            ErrorCode::InvalidRequest,
            &format!("Unknown action: {}", action),
        ),
    }
}

fn handle_page_extract(
    id: &str,
    params: &serde_json::Value,
    state: &mut ConnectionState,
) -> Response {
    let session_id = params
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let session = match state.get_session(session_id) {
        Some(s) => s,
        None => {
            return Response::error(
                id,
                ErrorCode::NotFound,
                &format!("Session not found: {}", session_id),
            )
        }
    };

    let som = match &session.current_som {
        Some(s) => s,
        None => return Response::error(id, ErrorCode::NotFound, "No page loaded yet"),
    };

    // If "structured_data" is requested, return the full structured data
    if params
        .get("structured_data")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        let sd = &session.current_structured_data;
        return Response::success(
            id,
            json!({
                "structured_data": sd,
                "url": session.current_url,
            }),
        );
    }

    // If "interactive_elements" is requested, return all interactive elements
    if params
        .get("interactive_elements")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        let all_elements = collect_all_elements(som);
        let interactive: Vec<serde_json::Value> = all_elements
            .iter()
            .filter(|e| e.role.is_interactive())
            .map(|e| {
                json!({
                    "id": e.id,
                    "role": e.role,
                    "text": e.text,
                    "label": e.label,
                    "actions": e.actions,
                    "attrs": e.attrs,
                    "hints": e.hints,
                })
            })
            .collect();
        return Response::success(
            id,
            json!({
                "interactive_elements": interactive,
                "count": interactive.len(),
            }),
        );
    }

    let fields = match params.get("fields") {
        Some(f) if f.is_object() => f.as_object().unwrap(),
        _ => {
            return Response::error(
                id,
                ErrorCode::InvalidRequest,
                "Missing required param: fields (object), or use structured_data:true / interactive_elements:true",
            )
        }
    };

    let all_elements = collect_all_elements(som);

    let mut data = serde_json::Map::new();
    let mut provenance = serde_json::Map::new();

    for (field_name, query) in fields {
        let (value, prov) = execute_field_query(query, &all_elements);
        data.insert(field_name.clone(), value);
        provenance.insert(field_name.clone(), prov);
    }

    Response::success(
        id,
        json!({
            "data": data,
            "provenance": provenance
        }),
    )
}

// ============================================================
// NETWORK INTERCEPTION
// ============================================================

fn handle_network_enable_interception(
    id: &str,
    params: &serde_json::Value,
    state: &mut ConnectionState,
) -> Response {
    let session_id = params
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let session = match state.get_session_mut(session_id) {
        Some(s) => s,
        None => {
            return Response::error(
                id,
                ErrorCode::NotFound,
                &format!("Session not found: {}", session_id),
            )
        }
    };

    let patterns = params
        .get("patterns")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|p| {
                    let url_pattern = p
                        .get("url_pattern")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    let resource_type = p
                        .get("resource_type")
                        .and_then(|v| v.as_str())
                        .map(ResourceType::from_cdp_str);
                    let request_stage = p
                        .get("stage")
                        .and_then(|v| v.as_str())
                        .map(|s| match s {
                            "response" | "Response" => RequestStage::Response,
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
            vec![RequestPattern {
                url_pattern: Some("*".to_string()),
                resource_type: None,
                request_stage: RequestStage::Request,
            }]
        });

    session.interceptor.enable(patterns);
    info!("AWP network interception enabled");
    Response::success(id, json!({"enabled": true}))
}

fn handle_network_disable_interception(
    id: &str,
    params: &serde_json::Value,
    state: &mut ConnectionState,
) -> Response {
    let session_id = params
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let session = match state.get_session_mut(session_id) {
        Some(s) => s,
        None => {
            return Response::error(
                id,
                ErrorCode::NotFound,
                &format!("Session not found: {}", session_id),
            )
        }
    };

    session.interceptor.disable();
    info!("AWP network interception disabled");
    Response::success(id, json!({"enabled": false}))
}

fn handle_network_add_rule(
    id: &str,
    params: &serde_json::Value,
    state: &mut ConnectionState,
) -> Response {
    let session_id = params
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let session = match state.get_session_mut(session_id) {
        Some(s) => s,
        None => {
            return Response::error(
                id,
                ErrorCode::NotFound,
                &format!("Session not found: {}", session_id),
            )
        }
    };

    let rule = match params.get("rule") {
        Some(r) => r,
        None => {
            return Response::error(
                id,
                ErrorCode::InvalidRequest,
                "Missing required param: rule",
            )
        }
    };

    let url_pattern = rule
        .get("url_pattern")
        .and_then(|v| v.as_str())
        .map(String::from);
    let resource_type = rule
        .get("resource_type")
        .and_then(|v| v.as_str())
        .map(ResourceType::from_cdp_str);

    let action_str = rule
        .get("action")
        .and_then(|v| v.as_str())
        .unwrap_or("continue");

    let stage = rule
        .get("stage")
        .and_then(|v| v.as_str())
        .unwrap_or("request");

    if stage == "response" {
        // Response modification rule
        let status = rule
            .get("status")
            .and_then(|v| v.as_u64())
            .map(|s| s as u16);
        let body = rule.get("body").and_then(|v| v.as_str()).map(String::from);

        session.interceptor.add_response_rule(ResponseRule {
            pattern: RequestPattern {
                url_pattern,
                resource_type,
                request_stage: RequestStage::Response,
            },
            overrides: ResponseOverrides {
                status,
                headers: None,
                body,
            },
        });

        return Response::success(id, json!({"added": true, "stage": "response"}));
    }

    // Request-stage rule
    let action = match action_str {
        "block" | "fail" => {
            let reason = rule
                .get("error_reason")
                .and_then(|v| v.as_str())
                .map(ErrorReason::from_cdp_str)
                .unwrap_or(ErrorReason::BlockedByClient);
            InterceptAction::Fail(reason)
        }
        "fulfill" | "mock" => {
            let status = rule.get("status").and_then(|v| v.as_u64()).unwrap_or(200) as u16;
            let body = rule.get("body").and_then(|v| v.as_str()).map(String::from);
            let headers: Vec<(String, String)> = rule
                .get("headers")
                .and_then(|v| v.as_object())
                .map(|obj| {
                    obj.iter()
                        .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string())))
                        .collect()
                })
                .unwrap_or_default();
            InterceptAction::Fulfill(FulfillParams {
                status,
                headers,
                body,
            })
        }
        _ => {
            // "continue" with optional overrides
            let url = rule
                .get("redirect_url")
                .and_then(|v| v.as_str())
                .map(String::from);
            let has_overrides = url.is_some();
            if has_overrides {
                InterceptAction::Continue(Some(RequestOverrides {
                    url,
                    ..Default::default()
                }))
            } else {
                InterceptAction::Continue(None)
            }
        }
    };

    session.interceptor.add_rule(InterceptRule {
        pattern: RequestPattern {
            url_pattern,
            resource_type,
            request_stage: RequestStage::Request,
        },
        action,
    });

    Response::success(id, json!({"added": true, "stage": "request"}))
}

fn handle_network_remove_rule(
    id: &str,
    params: &serde_json::Value,
    state: &mut ConnectionState,
) -> Response {
    let session_id = params
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let session = match state.get_session_mut(session_id) {
        Some(s) => s,
        None => {
            return Response::error(
                id,
                ErrorCode::NotFound,
                &format!("Session not found: {}", session_id),
            )
        }
    };

    let url_pattern = match params.get("url_pattern").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => {
            return Response::error(
                id,
                ErrorCode::InvalidRequest,
                "Missing required param: url_pattern",
            )
        }
    };

    session.interceptor.remove_rules_by_url(url_pattern);
    Response::success(id, json!({"removed": true}))
}

fn handle_network_clear_rules(
    id: &str,
    params: &serde_json::Value,
    state: &mut ConnectionState,
) -> Response {
    let session_id = params
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let session = match state.get_session_mut(session_id) {
        Some(s) => s,
        None => {
            return Response::error(
                id,
                ErrorCode::NotFound,
                &format!("Session not found: {}", session_id),
            )
        }
    };

    session.interceptor.clear_rules();
    Response::success(id, json!({"cleared": true}))
}

fn handle_network_get_intercepted_requests(
    id: &str,
    params: &serde_json::Value,
    state: &mut ConnectionState,
) -> Response {
    let session_id = params
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let session = match state.get_session(session_id) {
        Some(s) => s,
        None => {
            return Response::error(
                id,
                ErrorCode::NotFound,
                &format!("Session not found: {}", session_id),
            )
        }
    };

    let log = session.interceptor.intercepted_log();
    let entries: Vec<serde_json::Value> = log
        .iter()
        .map(|entry| {
            json!({
                "request_id": entry.request_id,
                "url": entry.url,
                "method": entry.method,
                "resource_type": entry.resource_type,
                "is_navigation": entry.is_navigation,
            })
        })
        .collect();

    Response::success(
        id,
        json!({
            "requests": entries,
            "count": entries.len(),
        }),
    )
}

async fn handle_plugin_list(id: &str, state: &ConnectionState) -> Response {
    let manifests = if let Some(ref pm) = state.plugins {
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

    Response::success(
        id,
        json!({
            "plugins": manifests,
            "count": manifests.len(),
        }),
    )
}

// ============================================================================
// Cookie APIs (session.cookies.*)
// ============================================================================

/// Get cookies for a URL or all cookies in the session.
///
/// Params:
/// - session_id (required): Session ID
/// - url (optional): Filter cookies by URL (domain/path matching)
///
/// Returns: { cookies: [...], count: N }
fn handle_cookies_get(
    id: &str,
    params: &serde_json::Value,
    state: &mut ConnectionState,
) -> Response {
    let session_id = params
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let session = match state.get_session(session_id) {
        Some(s) => s,
        None => {
            return Response::error(
                id,
                ErrorCode::NotFound,
                &format!("Session not found: {}", session_id),
            )
        }
    };

    let cookies: Vec<serde_json::Value> = if let Some(url) = params.get("url").and_then(|v| v.as_str()) {
        // Get cookies matching the URL
        session
            .cookies
            .get_cookies(url)
            .iter()
            .map(cookie_to_json)
            .collect()
    } else {
        // Get all cookies
        session
            .cookies
            .get_all_cookies()
            .iter()
            .map(cookie_to_json)
            .collect()
    };

    let count = cookies.len();
    Response::success(id, json!({ "cookies": cookies, "count": count }))
}

/// Set one or more cookies in the session.
///
/// Params:
/// - session_id (required): Session ID
/// - cookies (required): Array of cookie objects with:
///   - name (required)
///   - value (required)
///   - domain (required, or url to derive from)
///   - url (optional, used to derive domain if not specified)
///   - path (optional, defaults to "/")
///   - expires (optional, Unix timestamp in seconds)
///   - httpOnly (optional, defaults to false)
///   - secure (optional, defaults to false)
///   - sameSite (optional, "Strict" | "Lax" | "None", defaults to "Lax")
///
/// Returns: { set: N }
fn handle_cookies_set(
    id: &str,
    params: &serde_json::Value,
    state: &mut ConnectionState,
) -> Response {
    let session_id = params
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let session = match state.get_session_mut(session_id) {
        Some(s) => s,
        None => {
            return Response::error(
                id,
                ErrorCode::NotFound,
                &format!("Session not found: {}", session_id),
            )
        }
    };

    let cookies_param = match params.get("cookies") {
        Some(c) if c.is_array() => c.as_array().unwrap(),
        _ => {
            return Response::error(
                id,
                ErrorCode::InvalidRequest,
                "Missing or invalid 'cookies' array",
            )
        }
    };

    let mut set_count = 0;
    for cookie_params in cookies_param {
        if let Some(cookie) = cookie_from_cdp_params(cookie_params) {
            session.cookies.set_cookie(cookie);
            set_count += 1;
        } else {
            warn!("Skipping invalid cookie: {:?}", cookie_params);
        }
    }

    info!(session_id, set_count, "Cookies set");
    Response::success(id, json!({ "set": set_count }))
}

/// Clear cookies from the session.
///
/// Params:
/// - session_id (required): Session ID
/// - name (optional): Only delete cookies with this name
/// - domain (optional): Only delete cookies for this domain
/// - url (optional): Only delete cookies matching this URL
///
/// If no filters provided, clears all cookies.
///
/// Returns: { cleared: N }
fn handle_cookies_clear(
    id: &str,
    params: &serde_json::Value,
    state: &mut ConnectionState,
) -> Response {
    let session_id = params
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let session = match state.get_session_mut(session_id) {
        Some(s) => s,
        None => {
            return Response::error(
                id,
                ErrorCode::NotFound,
                &format!("Session not found: {}", session_id),
            )
        }
    };

    let name = params.get("name").and_then(|v| v.as_str());
    let domain = params.get("domain").and_then(|v| v.as_str());
    let url = params.get("url").and_then(|v| v.as_str());

    let cleared = if name.is_none() && domain.is_none() && url.is_none() {
        // Clear all cookies
        let count = session.cookies.len();
        session.cookies.clear();
        count
    } else if let Some(cookie_name) = name {
        // Delete specific cookies
        session.cookies.delete_cookies(cookie_name, url, domain, None)
    } else {
        // Domain/URL only filter - need to iterate and clear matching
        let cookies_to_check = if let Some(url_str) = url {
            session.cookies.get_cookies(url_str)
        } else {
            session.cookies.get_all_cookies()
        };

        let mut cleared = 0;
        for cookie in cookies_to_check {
            let domain_matches = domain
                .map(|d| cookie.domain.contains(d) || d.contains(&cookie.domain))
                .unwrap_or(true);

            if domain_matches {
                if session.cookies.remove_cookie(&cookie.name, &cookie.domain, Some(&cookie.path)) {
                    cleared += 1;
                }
            }
        }
        cleared
    };

    info!(session_id, cleared, "Cookies cleared");
    Response::success(id, json!({ "cleared": cleared }))
}

/// Convert a Cookie to JSON for AWP response.
fn cookie_to_json(cookie: &Cookie) -> serde_json::Value {
    json!({
        "name": cookie.name,
        "value": cookie.value,
        "domain": cookie.domain,
        "path": cookie.path,
        "expires": cookie.expires,
        "httpOnly": cookie.http_only,
        "secure": cookie.secure,
        "sameSite": cookie.same_site.as_str(),
        "size": cookie.size,
    })
}

/// Resolve a target specification to an element in the SOM.
fn resolve_target(target: &serde_json::Value, som: &crate::som::types::Som) -> Option<Element> {
    let all_elements = collect_all_elements(som);

    // Try ref first
    if let Some(ref_id) = target.get("ref").and_then(|v| v.as_str()) {
        if let Some(el) = all_elements.iter().find(|e| e.id == ref_id) {
            return Some(el.clone());
        }
    }

    // Try text + role semantic query
    let target_text = target.get("text").and_then(|v| v.as_str());
    let target_role = target.get("role").and_then(|v| v.as_str());

    if target_text.is_some() || target_role.is_some() {
        for el in &all_elements {
            let role_match = target_role.map(|r| el.role.as_str() == r).unwrap_or(true);
            let text_match = target_text
                .map(|t| {
                    el.text
                        .as_ref()
                        .map(|et| et.to_lowercase().contains(&t.to_lowercase()))
                        .unwrap_or(false)
                })
                .unwrap_or(true);
            if role_match && text_match {
                return Some(el.clone());
            }
        }
    }

    // Try CSS selector fallback (basic: tag.class or tag#id)
    if let Some(css) = target.get("css").and_then(|v| v.as_str()) {
        // Basic CSS matching against element IDs
        // (full CSS selector support is out of scope for v0.1)
        warn!(css, "CSS selector fallback is limited in v0.1");
    }

    // Try fallback
    if let Some(fallback) = target.get("fallback") {
        return resolve_target(fallback, som);
    }

    None
}

fn collect_all_elements(som: &crate::som::types::Som) -> Vec<Element> {
    let mut elements = Vec::new();
    for region in &som.regions {
        collect_elements_recursive(&region.elements, &mut elements);
    }
    elements
}

fn collect_elements_recursive(elements: &[Element], out: &mut Vec<Element>) {
    for el in elements {
        out.push(el.clone());
        if let Some(children) = &el.children {
            collect_elements_recursive(children, out);
        }
    }
}

fn execute_field_query(
    query: &serde_json::Value,
    elements: &[Element],
) -> (serde_json::Value, serde_json::Value) {
    let all = query.get("all").and_then(|v| v.as_bool()).unwrap_or(false);
    let props: Vec<String> = query
        .get("props")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    // Match by ref
    if let Some(ref_id) = query.get("ref").and_then(|v| v.as_str()) {
        if let Some(el) = elements.iter().find(|e| e.id == ref_id) {
            let value = element_to_extract_value(el, &props);
            return (value, json!(el.id));
        }
        return (json!(null), json!(null));
    }

    // Match by role (and optional level)
    if let Some(role_str) = query.get("role").and_then(|v| v.as_str()) {
        let level = query.get("level").and_then(|v| v.as_u64());

        let matches: Vec<&Element> = elements
            .iter()
            .filter(|e| {
                if e.role.as_str() != role_str {
                    return false;
                }
                if let Some(lvl) = level {
                    if let Some(attrs) = &e.attrs {
                        if let Some(el_level) = attrs.get("level").and_then(|v| v.as_u64()) {
                            return el_level == lvl;
                        }
                    }
                    return false;
                }
                true
            })
            .collect();

        if all {
            let values: Vec<serde_json::Value> = matches
                .iter()
                .map(|e| element_to_extract_value(e, &props))
                .collect();
            let ids: Vec<serde_json::Value> = matches.iter().map(|e| json!(e.id)).collect();
            return (json!(values), json!(ids));
        } else if let Some(first) = matches.first() {
            let value = if props.is_empty() {
                first.text.as_ref().map(|t| json!(t)).unwrap_or(json!(null))
            } else {
                element_to_extract_value(first, &props)
            };
            return (value, json!(first.id));
        }
        return (json!(null), json!(null));
    }

    // Match by text_match (regex)
    if let Some(pattern) = query.get("text_match").and_then(|v| v.as_str()) {
        if let Ok(re) = Regex::new(pattern) {
            for el in elements {
                if let Some(text) = &el.text {
                    if re.is_match(text) {
                        let value = json!(text);
                        return (value, json!(el.id));
                    }
                }
            }
        }
        return (json!(null), json!(null));
    }

    (json!(null), json!(null))
}

fn element_to_extract_value(el: &Element, props: &[String]) -> serde_json::Value {
    if props.is_empty() {
        return el.text.as_ref().map(|t| json!(t)).unwrap_or(json!(null));
    }

    let mut map = serde_json::Map::new();
    for prop in props {
        match prop.as_str() {
            "text" => {
                if let Some(t) = &el.text {
                    map.insert("text".into(), json!(t));
                }
            }
            "href" => {
                if let Some(attrs) = &el.attrs {
                    if let Some(href) = attrs.get("href") {
                        map.insert("href".into(), href.clone());
                    }
                }
            }
            "value" => {
                if let Some(attrs) = &el.attrs {
                    if let Some(v) = attrs.get("value") {
                        map.insert("value".into(), v.clone());
                    }
                }
            }
            "src" => {
                if let Some(attrs) = &el.attrs {
                    if let Some(v) = attrs.get("src") {
                        map.insert("src".into(), v.clone());
                    }
                }
            }
            "alt" => {
                if let Some(attrs) = &el.attrs {
                    if let Some(v) = attrs.get("alt") {
                        map.insert("alt".into(), v.clone());
                    }
                }
            }
            other => {
                if let Some(attrs) = &el.attrs {
                    if let Some(v) = attrs.get(other) {
                        map.insert(other.to_string(), v.clone());
                    }
                }
            }
        }
    }
    serde_json::Value::Object(map)
}

fn resolve_url(base: &str, href: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        return href.to_string();
    }
    if let Ok(base_url) = url::Url::parse(base) {
        if let Ok(resolved) = base_url.join(href) {
            return resolved.to_string();
        }
    }
    href.to_string()
}

fn update_element_value(som: &mut crate::som::types::Som, element_id: &str, value: &str) {
    for region in &mut som.regions {
        update_element_value_in_list(&mut region.elements, element_id, value);
    }
}

fn update_element_value_in_list(elements: &mut [Element], element_id: &str, value: &str) {
    for el in elements.iter_mut() {
        if el.id == element_id {
            let attrs = el.attrs.get_or_insert_with(|| json!({}));
            if let Some(obj) = attrs.as_object_mut() {
                obj.insert("value".into(), json!(value));
            }
            return;
        }
        if let Some(children) = &mut el.children {
            update_element_value_in_list(children, element_id, value);
        }
    }
}

fn toggle_element(som: &mut crate::som::types::Som, element_id: &str) {
    for region in &mut som.regions {
        toggle_element_in_list(&mut region.elements, element_id);
    }
}

fn toggle_element_in_list(elements: &mut [Element], element_id: &str) {
    for el in elements.iter_mut() {
        if el.id == element_id {
            let attrs = el.attrs.get_or_insert_with(|| json!({}));
            if let Some(obj) = attrs.as_object_mut() {
                // Toggle "checked" for checkboxes/radios, "open" for details
                if let Some(checked) = obj.get("checked").and_then(|v| v.as_bool()) {
                    obj.insert("checked".into(), json!(!checked));
                } else if obj.contains_key("open") {
                    let open = obj.get("open").and_then(|v| v.as_bool()).unwrap_or(false);
                    obj.insert("open".into(), json!(!open));
                } else {
                    // Default: set checked to true (first toggle)
                    obj.insert("checked".into(), json!(true));
                }
            }
            return;
        }
        if let Some(children) = &mut el.children {
            toggle_element_in_list(children, element_id);
        }
    }
}

/// Parse TLS configuration from AWP session.create params.
///
/// Supports:
///   "tls": {
///     "min_version": "1.3",
///     "max_version": "1.3",
///     "insecure": true,
///     "ca_cert": "/path/to/ca.pem",
///     "tls12_ciphers": ["TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256"],
///     "tls13_ciphers": ["TLS_AES_128_GCM_SHA256"],
///     "alpn": ["h2", "http/1.1"],
///     "groups": ["x25519", "secp256r1"],
///     "sni": false
///   }
fn parse_tls_params(params: &serde_json::Value) -> Option<TlsConfig> {
    let tls = params.get("tls")?;

    let min_version = tls
        .get("min_version")
        .and_then(|v| v.as_str())
        .and_then(|s| crate::network::tls::TlsVersion::parse(s).ok());

    let max_version = tls
        .get("max_version")
        .and_then(|v| v.as_str())
        .and_then(|s| crate::network::tls::TlsVersion::parse(s).ok());

    let insecure = tls
        .get("insecure")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let ca_cert_path = tls
        .get("ca_cert")
        .and_then(|v| v.as_str())
        .map(std::path::PathBuf::from);

    let cipher_suites_tls12 = parse_string_array(tls, "tls12_ciphers");
    let cipher_suites_tls13 = parse_string_array(tls, "tls13_ciphers");
    let alpn_protocols = parse_string_array(tls, "alpn");
    let supported_groups = parse_string_array(tls, "groups");

    let enable_sni = tls.get("sni").and_then(|v| v.as_bool());

    let config = TlsConfig {
        min_version,
        max_version,
        danger_accept_invalid_certs: insecure,
        ca_cert_path,
        cipher_suites_tls12,
        cipher_suites_tls13,
        alpn_protocols,
        supported_groups,
        enable_sni,
    };

    if config.is_default() {
        None
    } else {
        info!(tls = %config.summary(), "Per-session TLS configuration");
        Some(config)
    }
}

fn parse_string_array(obj: &serde_json::Value, key: &str) -> Vec<String> {
    obj.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}
