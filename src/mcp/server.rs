//! MCP stdio server implementation.
//!
//! Reads JSON-RPC 2.0 messages from stdin, processes them, and writes responses to stdout.
//! All log output goes to stderr to keep stdout clean for the protocol.

use std::io::{self, BufRead, Write};
use std::sync::Arc;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, error, info};

use super::protocol::{self, ProtocolAdapter, ProtocolSelectionError, ProtocolState};
use super::sessions::SessionManager;
use super::tools::{self, ToolDefinition};
use crate::cache::store::{CacheConfig, SomCache};
use crate::network::fetch;

/// Server name.
const SERVER_NAME: &str = "plasmate";

/// Server version (matches crate version).
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// JSON-RPC 2.0 request structure.
#[derive(Debug, Deserialize)]
pub(super) struct JsonRpcRequest {
    pub(super) jsonrpc: String,
    #[serde(default)]
    pub(super) id: Option<Value>,
    pub(super) method: String,
    #[serde(default)]
    pub(super) params: Option<Value>,
}

/// JSON-RPC 2.0 response structure.
#[derive(Debug, Serialize)]
pub(super) struct JsonRpcResponse {
    pub(super) jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 error structure.
#[derive(Debug, Serialize)]
pub(super) struct JsonRpcError {
    pub(super) code: i32,
    pub(super) message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) data: Option<Value>,
}

// Standard JSON-RPC error codes
pub(super) const PARSE_ERROR: i32 = -32700;
pub(super) const INVALID_REQUEST: i32 = -32600;
pub(super) const METHOD_NOT_FOUND: i32 = -32601;
pub(super) const INVALID_PARAMS: i32 = -32602;
#[allow(dead_code)]
const INTERNAL_ERROR: i32 = -32603;

/// Run the MCP server, reading from stdin and writing to stdout.
pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting MCP server");

    // Build HTTP client for fetching pages
    let jar = Arc::new(reqwest::cookie::Jar::default());
    let client = fetch::build_client_h1_fallback(None, jar, None)?;

    // Session manager for stateful browser tools
    let sessions = Arc::new(SessionManager::new());
    let cache = Arc::new(SomCache::new(CacheConfig::default()));
    let mut protocol_state = ProtocolState::default();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                error!("Error reading stdin: {}", e);
                break;
            }
        };

        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        debug!("Received: {}", line);

        // Parse JSON-RPC request
        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: PARSE_ERROR,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
                write_response(&mut stdout, &response)?;
                continue;
            }
        };

        // Validate JSON-RPC version
        if request.jsonrpc != "2.0" {
            let response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: INVALID_REQUEST,
                    message: "Invalid JSON-RPC version".to_string(),
                    data: None,
                }),
            };
            write_response(&mut stdout, &response)?;
            continue;
        }

        // Handle the request
        let response =
            handle_request(&request, &client, &sessions, &cache, &mut protocol_state).await;

        // MCP notifications (no id) must not receive a response.
        if request.id.is_none() && request.method.starts_with("notifications/") {
            continue;
        }

        write_response(&mut stdout, &response)?;
    }

    info!("MCP server shutting down");
    Ok(())
}

/// Write a JSON-RPC response to stdout.
fn write_response(
    stdout: &mut io::Stdout,
    response: &JsonRpcResponse,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string(response)?;
    debug!("Sending: {}", json);
    writeln!(stdout, "{}", json)?;
    stdout.flush()?;
    Ok(())
}

/// Handle a JSON-RPC request and return a response.
pub(super) async fn handle_request(
    request: &JsonRpcRequest,
    client: &reqwest::Client,
    sessions: &Arc<SessionManager>,
    cache: &Arc<SomCache>,
    protocol_state: &mut ProtocolState,
) -> JsonRpcResponse {
    match request.method.as_str() {
        // MCP lifecycle methods
        "initialize" => handle_initialize(request, protocol_state),
        "notifications/initialized" => handle_initialized_notification(request, protocol_state),
        "server/discover" => handle_server_discover(request),
        "ping" => handle_ping(request, protocol_state),

        // MCP tool methods
        "tools/list" => match protocol_state.adapter_for_request(request.params.as_ref()) {
            Ok(adapter) => handle_tools_list(request, adapter),
            Err(error) => protocol_selection_error(request, error),
        },
        "tools/call" => match protocol_state.adapter_for_request(request.params.as_ref()) {
            Ok(adapter) => handle_tools_call(request, client, sessions, cache, adapter).await,
            Err(error) => protocol_selection_error(request, error),
        },

        // Unknown method
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: None,
            error: Some(JsonRpcError {
                code: METHOD_NOT_FOUND,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        },
    }
}

/// Handle the 'initialize' method.
fn handle_initialize(
    request: &JsonRpcRequest,
    protocol_state: &mut ProtocolState,
) -> JsonRpcResponse {
    let params = match request.params.as_ref().and_then(Value::as_object) {
        Some(params) => params,
        None => return invalid_params_response(request, "Missing initialize params object"),
    };
    let requested = match params.get("protocolVersion").and_then(Value::as_str) {
        Some(version) => version,
        None => return invalid_params_response(request, "Missing protocolVersion"),
    };
    if !params.get("capabilities").is_some_and(Value::is_object) {
        return invalid_params_response(request, "Missing client capabilities object");
    }
    let client_info = match params.get("clientInfo").and_then(Value::as_object) {
        Some(client_info) => client_info,
        None => return invalid_params_response(request, "Missing clientInfo object"),
    };
    if !client_info.get("name").is_some_and(Value::is_string)
        || !client_info.get("version").is_some_and(Value::is_string)
    {
        return invalid_params_response(
            request,
            "clientInfo must contain string name and version fields",
        );
    }
    debug!("Client protocol version: {}", requested);
    let adapter = protocol_state.negotiate_initialize(Some(requested));

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: Some(protocol::initialize_result(
            adapter,
            SERVER_NAME,
            SERVER_VERSION,
        )),
        error: None,
    }
}

/// Handle the 'notifications/initialized' notification.
fn handle_initialized_notification(
    request: &JsonRpcRequest,
    protocol_state: &mut ProtocolState,
) -> JsonRpcResponse {
    if let Err(message) = protocol_state.mark_initialized() {
        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: None,
            error: Some(JsonRpcError {
                code: INVALID_REQUEST,
                message: message.to_string(),
                data: None,
            }),
        };
    }

    info!("MCP client initialized");
    if request.id.is_some() {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: Some(json!({})),
            error: None,
        }
    } else {
        // For true notifications, we still need to return something to avoid hanging
        // but we won't write it out
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: None,
            result: None,
            error: None,
        }
    }
}

fn handle_server_discover(request: &JsonRpcRequest) -> JsonRpcResponse {
    if let Err(error) = protocol::modern_adapter_for_request(request.params.as_ref()) {
        return protocol_selection_error(request, error);
    }
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: Some(protocol::discover_result(SERVER_NAME, SERVER_VERSION)),
        error: None,
    }
}

fn handle_ping(request: &JsonRpcRequest, protocol_state: &ProtocolState) -> JsonRpcResponse {
    // Initialize-era clients may ping while the lifecycle is still in
    // progress. Modern requests still need their complete per-request meta.
    let adapter = if protocol::request_protocol_version(request.params.as_ref()).is_some() {
        match protocol::modern_adapter_for_request(request.params.as_ref()) {
            Ok(adapter) => adapter,
            Err(error) => return protocol_selection_error(request, error),
        }
    } else {
        protocol_state
            .adapter_for_request(request.params.as_ref())
            .unwrap_or(ProtocolAdapter::Stable2025)
    };
    let result = if adapter.is_modern() {
        json!({ "resultType": "complete" })
    } else {
        json!({})
    };
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: Some(result),
        error: None,
    }
}

/// Handle the 'tools/list' method.
fn handle_tools_list(request: &JsonRpcRequest, adapter: ProtocolAdapter) -> JsonRpcResponse {
    let tools: Vec<ToolDefinition> = vec![
        // Phase 1: Stateless tools
        tools::fetch_page_definition(),
        tools::extract_text_definition(),
        tools::extract_links_definition(),
        tools::ard_discover_definition(),
        tools::crawl_policy_definition(),
        tools::inspect_page_definition(),
        tools::cache_status_definition(),
        tools::session_status_definition(),
        tools::trace_status_definition(),
        tools::trace_export_definition(),
        tools::trace_clear_definition(),
        tools::replay_validate_definition(),
        // Screenshot
        tools::screenshot_page_definition(),
        // Phase 2: Stateful tools
        tools::open_page_definition(),
        tools::evaluate_definition(),
        tools::click_definition(),
        tools::close_page_definition(),
        // Phase 3: Interaction tools
        tools::navigate_to_definition(),
        tools::type_text_definition(),
        tools::select_option_definition(),
        tools::scroll_definition(),
        tools::toggle_definition(),
        tools::clear_definition(),
        // Cookie tools
        tools::get_cookies_definition(),
        tools::set_cookies_definition(),
        tools::clear_cookies_definition(),
    ];

    let definitions = tools
        .into_iter()
        .map(|definition| json!(definition))
        .collect();

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: Some(protocol::adapt_tool_list(adapter, definitions)),
        error: None,
    }
}

/// Handle the 'tools/call' method.
async fn handle_tools_call(
    request: &JsonRpcRequest,
    client: &reqwest::Client,
    sessions: &Arc<SessionManager>,
    cache: &Arc<SomCache>,
    adapter: ProtocolAdapter,
) -> JsonRpcResponse {
    let params = match &request.params {
        Some(p) => p,
        None => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: None,
                error: Some(JsonRpcError {
                    code: INVALID_PARAMS,
                    message: "Missing params".to_string(),
                    data: None,
                }),
            };
        }
    };

    let tool_name = match params.get("name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: None,
                error: Some(JsonRpcError {
                    code: INVALID_PARAMS,
                    message: "Missing tool name".to_string(),
                    data: None,
                }),
            };
        }
    };

    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
    let trace_started = Instant::now();
    let trace_attempt = sessions.prepare_trace(tool_name, &arguments).await;

    let mut result = match tool_name {
        // Phase 1: Stateless tools
        "fetch_page" => tools::handle_fetch_page(&arguments, client, cache).await,
        "extract_text" => tools::handle_extract_text(&arguments, client, cache).await,
        "extract_links" => tools::handle_extract_links(&arguments, client, cache).await,
        "ard_discover" => tools::handle_ard_discover(&arguments).await,
        "crawl_policy" => tools::handle_crawl_policy(&arguments).await,
        "inspect_page" => tools::handle_inspect_page(&arguments, client).await,
        "cache_status" => tools::handle_cache_status(cache),
        "session_status" => tools::handle_session_status(sessions).await,
        "trace_status" => tools::handle_trace_status(&arguments, sessions).await,
        "trace_export" => tools::handle_trace_export(&arguments, sessions).await,
        "trace_clear" => tools::handle_trace_clear(&arguments, sessions).await,
        "replay_validate" => tools::handle_replay_validate(&arguments, sessions).await,
        // Screenshot
        "screenshot_page" => tools::handle_screenshot_page(&arguments, client).await,
        // Phase 2: Stateful tools
        "open_page" => tools::handle_open_page(&arguments, client, sessions, cache).await,
        "evaluate" => tools::handle_evaluate(&arguments, sessions).await,
        "click" => tools::handle_click(&arguments, client, sessions).await,
        "close_page" => tools::handle_close_page(&arguments, sessions).await,
        // Phase 3: Interaction tools
        "navigate_to" => tools::handle_navigate_to(&arguments, client, sessions, cache).await,
        "type_text" => tools::handle_type_text(&arguments, client, sessions).await,
        "select_option" => tools::handle_select_option(&arguments, client, sessions).await,
        "scroll" => tools::handle_scroll(&arguments, client, sessions).await,
        "toggle" => tools::handle_toggle(&arguments, client, sessions).await,
        "clear" => tools::handle_clear(&arguments, client, sessions).await,
        // Cookie tools
        "get_cookies" => tools::handle_get_cookies(&arguments, sessions).await,
        "set_cookies" => tools::handle_set_cookies(&arguments, sessions).await,
        "clear_cookies" => tools::handle_clear_cookies(&arguments, sessions).await,
        _ => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: None,
                error: Some(JsonRpcError {
                    code: INVALID_PARAMS,
                    message: format!("Unknown tool: {}", tool_name),
                    data: None,
                }),
            };
        }
    };

    let trace_duration = trace_started.elapsed();
    if super::trace::open_trace_requested(tool_name, &arguments) {
        if let Some(session_id) = super::trace::session_id_from_tool_result(&result) {
            sessions
                .record_open_trace(&session_id, &arguments, &result, trace_duration)
                .await;
        }
    }
    if let Some(attempt) = trace_attempt {
        if let Some(final_export) = sessions
            .finish_trace(attempt, &result, trace_duration)
            .await
        {
            super::trace::attach_final_trace_export(&mut result, final_export);
        }
    }

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: Some(protocol::adapt_tool_result(adapter, tool_name, result)),
        error: None,
    }
}

fn invalid_params_response(request: &JsonRpcRequest, message: &str) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: None,
        error: Some(JsonRpcError {
            code: INVALID_PARAMS,
            message: message.to_string(),
            data: None,
        }),
    }
}

fn protocol_selection_error(
    request: &JsonRpcRequest,
    error: ProtocolSelectionError,
) -> JsonRpcResponse {
    let (code, message, data) = match error {
        ProtocolSelectionError::NotInitialized => (
            INVALID_REQUEST,
            "MCP connection is not initialized; call initialize or use modern per-request metadata"
                .to_string(),
            None,
        ),
        ProtocolSelectionError::NotReady => (
            INVALID_REQUEST,
            "MCP connection is awaiting notifications/initialized".to_string(),
            None,
        ),
        ProtocolSelectionError::Unsupported(version) => (
            -32022,
            format!("Unsupported protocol version: {}", version),
            Some(json!({
                "supported": [protocol::MODERN_RC_VERSION],
                "requested": version
            })),
        ),
        ProtocolSelectionError::InvalidModernMetadata(reason) => (
            INVALID_PARAMS,
            format!("Invalid modern MCP request metadata: {}", reason),
            Some(json!({
                "required": [
                    "io.modelcontextprotocol/protocolVersion",
                    "io.modelcontextprotocol/clientInfo",
                    "io.modelcontextprotocol/clientCapabilities"
                ]
            })),
        ),
    };
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: None,
        error: Some(JsonRpcError {
            code,
            message,
            data,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(id: Option<u64>, method: &str, params: Option<Value>) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: id.map(|id| json!(id)),
            method: method.to_string(),
            params,
        }
    }

    fn dependencies() -> (reqwest::Client, Arc<SessionManager>, Arc<SomCache>) {
        (
            reqwest::Client::new(),
            Arc::new(SessionManager::new()),
            Arc::new(SomCache::new(CacheConfig::default())),
        )
    }

    async fn route(request: &JsonRpcRequest, state: &mut ProtocolState) -> JsonRpcResponse {
        let (client, sessions, cache) = dependencies();
        handle_request(request, &client, &sessions, &cache, state).await
    }

    #[tokio::test]
    async fn legacy_client_keeps_legacy_tool_shape() {
        let mut state = ProtocolState::default();
        let initialized = route(
            &request(
                Some(1),
                "initialize",
                Some(json!({
                    "protocolVersion": protocol::LEGACY_VERSION,
                    "capabilities": {},
                    "clientInfo": { "name": "legacy-test", "version": "1" }
                })),
            ),
            &mut state,
        )
        .await;
        assert_eq!(
            initialized.result.unwrap()["protocolVersion"],
            protocol::LEGACY_VERSION
        );

        route(
            &request(None, "notifications/initialized", None),
            &mut state,
        )
        .await;
        let listed = route(&request(Some(2), "tools/list", None), &mut state).await;
        let first = &listed.result.unwrap()["tools"][0];
        assert_eq!(first["name"], "fetch_page");
        assert!(first.get("title").is_none());
        assert!(first.get("outputSchema").is_none());
        assert!(first.get("annotations").is_none());
    }

    #[tokio::test]
    async fn stable_client_must_complete_lifecycle_and_gets_modern_tool_metadata() {
        let mut state = ProtocolState::default();
        let initialized = route(
            &request(
                Some(1),
                "initialize",
                Some(json!({
                    "protocolVersion": protocol::STABLE_VERSION,
                    "capabilities": {},
                    "clientInfo": { "name": "stable-test", "version": "1" }
                })),
            ),
            &mut state,
        )
        .await;
        assert_eq!(
            initialized.result.unwrap()["protocolVersion"],
            protocol::STABLE_VERSION
        );

        let too_early = route(&request(Some(2), "tools/list", None), &mut state).await;
        assert_eq!(too_early.error.unwrap().code, INVALID_REQUEST);

        route(
            &request(None, "notifications/initialized", None),
            &mut state,
        )
        .await;
        let listed = route(&request(Some(3), "tools/list", None), &mut state).await;
        let result = listed.result.unwrap();
        assert_eq!(result["tools"].as_array().unwrap().len(), 26);
        for tool in result["tools"].as_array().unwrap() {
            assert!(tool["title"].is_string());
            assert_eq!(tool["outputSchema"]["type"], "object");
            assert!(tool["annotations"]["readOnlyHint"].is_boolean());
            assert!(tool["annotations"]["destructiveHint"].is_boolean());
            assert!(tool["annotations"]["idempotentHint"].is_boolean());
            assert!(tool["annotations"]["openWorldHint"].is_boolean());
            assert!(tool.get("execution").is_none());
        }
    }

    #[tokio::test]
    async fn codex_stdio_version_is_echoed_and_tool_errors_keep_session_healthy() {
        let mut state = ProtocolState::default();
        let initialized = route(
            &request(
                Some(1),
                "initialize",
                Some(json!({
                    "protocolVersion": protocol::CODEX_COMPAT_VERSION,
                    "capabilities": {},
                    "clientInfo": { "name": "codex", "version": "test" }
                })),
            ),
            &mut state,
        )
        .await;
        assert_eq!(
            initialized.result.unwrap()["protocolVersion"],
            protocol::CODEX_COMPAT_VERSION
        );

        route(
            &request(None, "notifications/initialized", None),
            &mut state,
        )
        .await;

        let failed = route(
            &request(
                Some(2),
                "tools/call",
                Some(json!({ "name": "fetch_page", "arguments": {} })),
            ),
            &mut state,
        )
        .await;
        assert_eq!(failed.result.unwrap()["isError"], true);

        for id in 3..13 {
            let status = route(
                &request(
                    Some(id),
                    "tools/call",
                    Some(json!({ "name": "cache_status", "arguments": {} })),
                ),
                &mut state,
            )
            .await;
            let result = status.result.unwrap();
            assert_ne!(result["isError"], true);
            assert!(result["content"].is_array());
        }
    }

    #[tokio::test]
    async fn stable_tool_call_returns_schema_compatible_structured_content() {
        let mut state = ProtocolState::default();
        route(
            &request(
                Some(1),
                "initialize",
                Some(json!({
                    "protocolVersion": protocol::STABLE_VERSION,
                    "capabilities": {},
                    "clientInfo": { "name": "stable-test", "version": "1" }
                })),
            ),
            &mut state,
        )
        .await;
        route(
            &request(None, "notifications/initialized", None),
            &mut state,
        )
        .await;

        let called = route(
            &request(
                Some(2),
                "tools/call",
                Some(json!({ "name": "cache_status", "arguments": {} })),
            ),
            &mut state,
        )
        .await;
        let result = called.result.unwrap();
        assert!(result["content"].is_array());
        assert!(result["structuredContent"]["result"].is_object());
        assert_eq!(
            result["structuredContent"]["contentType"],
            "application/json"
        );
        assert_eq!(result["structuredContent"]["trust"], "local");
        assert!(result.get("resultType").is_none());
    }

    #[tokio::test]
    async fn dispatcher_traces_failures_once_and_close_returns_final_export() {
        let (client, sessions, cache) = dependencies();
        let session_id = sessions.create_session().await.unwrap();
        assert!(sessions.enable_trace(&session_id).await);

        let failed = handle_tools_call(
            &request(
                Some(1),
                "tools/call",
                Some(json!({
                    "name": "evaluate",
                    "arguments": {
                        "session_id": session_id.clone(),
                        "expression": "document.cookie = 'never-export-this'"
                    }
                })),
            ),
            &client,
            &sessions,
            &cache,
            ProtocolAdapter::Stable2025,
        )
        .await;
        assert_eq!(failed.result.as_ref().unwrap()["isError"], true);
        let export = sessions.trace_export(&session_id).await.unwrap();
        assert_eq!(export.retained_events, 1);
        let encoded = serde_json::to_string(&export).unwrap();
        assert!(!encoded.contains("never-export-this"));

        let closed = handle_tools_call(
            &request(
                Some(2),
                "tools/call",
                Some(json!({
                    "name": "close_page",
                    "arguments": {"session_id": session_id}
                })),
            ),
            &client,
            &sessions,
            &cache,
            ProtocolAdapter::Stable2025,
        )
        .await;
        let result = closed.result.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        let payload: Value = serde_json::from_str(text).unwrap();
        assert_eq!(payload["closed"], true);
        assert_eq!(payload["final_trace"]["retained_events"], 2);
        assert_eq!(payload["final_trace"]["events"][1]["action"], "close_page");
        assert_eq!(payload["final_trace"]["events"][1]["outcome"], "success");
        assert!(text.len() <= super::super::trace::MAX_TRACE_EXPORT_BYTES);
    }

    #[tokio::test]
    async fn modern_discovery_and_per_request_metadata_need_no_initialize() {
        let mut state = ProtocolState::default();
        let discovered = route(
            &request(
                Some(1),
                "server/discover",
                Some(json!({
                    "_meta": {
                        "io.modelcontextprotocol/protocolVersion": protocol::MODERN_RC_VERSION,
                        "io.modelcontextprotocol/clientInfo": {
                            "name": "modern-test",
                            "version": "1"
                        },
                        "io.modelcontextprotocol/clientCapabilities": {}
                    }
                })),
            ),
            &mut state,
        )
        .await;
        let result = discovered.result.unwrap();
        assert_eq!(result["resultType"], "complete");
        assert_eq!(result["supportedVersions"][0], protocol::MODERN_RC_VERSION);

        let params = json!({
            "_meta": {
                "io.modelcontextprotocol/protocolVersion": protocol::MODERN_RC_VERSION,
                "io.modelcontextprotocol/clientInfo": {
                    "name": "modern-test",
                    "version": "1"
                },
                "io.modelcontextprotocol/clientCapabilities": {}
            }
        });
        let listed = route(&request(Some(2), "tools/list", Some(params)), &mut state).await;
        let result = listed.result.unwrap();
        assert_eq!(result["resultType"], "complete");
        assert_eq!(result["cacheScope"], "public");
        assert!(result["ttlMs"].as_u64().unwrap() > 0);
    }

    #[tokio::test]
    async fn unsupported_per_request_version_has_downgrade_details() {
        let mut state = ProtocolState::default();
        let response = route(
            &request(
                Some(1),
                "tools/list",
                Some(json!({
                    "_meta": {
                        "io.modelcontextprotocol/protocolVersion": "2099-01-01"
                    }
                })),
            ),
            &mut state,
        )
        .await;
        let error = response.error.unwrap();
        assert_eq!(error.code, -32022);
        let data = error.data.unwrap();
        assert_eq!(data["supported"][0], protocol::MODERN_RC_VERSION);
        assert_eq!(data["requested"], "2099-01-01");
    }
}
