use regex::Regex;
use serde_json::json;
use tracing::{info, warn};

use super::messages::{ErrorCode, Response};
use super::session::Session;
use crate::som::types::{Element, ElementRole};

/// Connection state tracked per WebSocket connection.
pub struct ConnectionState {
    pub handshake_done: bool,
    pub session: Option<Session>,
}

impl ConnectionState {
    pub fn new() -> Self {
        ConnectionState {
            handshake_done: false,
            session: None,
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
        "page.navigate" => handle_page_navigate(id, params, state).await,
        "page.observe" => handle_page_observe(id, params, state),
        "page.act" => handle_page_act(id, params, state).await,
        "page.extract" => handle_page_extract(id, params, state),
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

    Response::success(
        id,
        json!({
            "awp_version": "0.1",
            "server_name": "plasmate",
            "server_version": "0.1.0",
            "features": ["som.snapshot", "act.primitive", "extract"]
        }),
    )
}

fn handle_session_create(
    id: &str,
    params: &serde_json::Value,
    state: &mut ConnectionState,
) -> Response {
    // Close existing session if any (v0.1: one session per connection)
    if state.session.is_some() {
        info!("Closing existing session for new session creation");
        state.session = None;
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

    match Session::new(session_id.clone(), user_agent, locale, timeout_ms) {
        Ok(session) => {
            info!(session_id = %session.id, "Session created");
            state.session = Some(session);
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

async fn handle_page_navigate(
    id: &str,
    params: &serde_json::Value,
    state: &mut ConnectionState,
) -> Response {
    let session_id = params
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let session = match &mut state.session {
        Some(s) if s.id == session_id => s,
        _ => {
            return Response::error(
                id,
                ErrorCode::NotFound,
                &format!("Session not found: {}", session_id),
            )
        }
    };

    let url = match params.get("url").and_then(|v| v.as_str()) {
        Some(u) => u,
        None => {
            return Response::error(id, ErrorCode::InvalidRequest, "Missing required param: url")
        }
    };

    info!(url, "Navigating (full pipeline)");

    // Use the full async pipeline: fetch -> external scripts -> V8 -> SOM
    match session.navigate(url).await {
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

    let session = match &state.session {
        Some(s) if s.id == session_id => s,
        _ => {
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

    let session = match &mut state.session {
        Some(s) if s.id == session_id => s,
        _ => {
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
            // No-op in v0.1
            Response::success(
                id,
                json!({
                    "status": "ok",
                    "resolved": null,
                    "effects": {
                        "navigated": false,
                        "som_changed": false
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

    let session = match &state.session {
        Some(s) if s.id == session_id => s,
        _ => {
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
