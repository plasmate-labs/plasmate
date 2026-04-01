//! MCP stdio server implementation.
//!
//! Reads JSON-RPC 2.0 messages from stdin, processes them, and writes responses to stdout.
//! All log output goes to stderr to keep stdout clean for the protocol.

use std::io::{self, BufRead, Write};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, error, info};

use super::sessions::SessionManager;
use super::tools::{self, ToolDefinition};
use crate::network::fetch;

/// MCP protocol version we support.
const PROTOCOL_VERSION: &str = "2024-11-05";

/// Server name.
const SERVER_NAME: &str = "plasmate";

/// Server version (matches crate version).
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// JSON-RPC 2.0 request structure.
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

/// JSON-RPC 2.0 response structure.
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 error structure.
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

// Standard JSON-RPC error codes
const PARSE_ERROR: i32 = -32700;
const INVALID_REQUEST: i32 = -32600;
const METHOD_NOT_FOUND: i32 = -32601;
const INVALID_PARAMS: i32 = -32602;
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
        let response = handle_request(&request, &client, &sessions).await;

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
async fn handle_request(
    request: &JsonRpcRequest,
    client: &reqwest::Client,
    sessions: &Arc<SessionManager>,
) -> JsonRpcResponse {
    match request.method.as_str() {
        // MCP lifecycle methods
        "initialize" => handle_initialize(request),
        "notifications/initialized" => handle_initialized_notification(request),

        // MCP tool methods
        "tools/list" => handle_tools_list(request),
        "tools/call" => handle_tools_call(request, client, sessions).await,

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
fn handle_initialize(request: &JsonRpcRequest) -> JsonRpcResponse {
    // Validate protocol version from params
    if let Some(params) = &request.params {
        if let Some(version) = params.get("protocolVersion").and_then(|v| v.as_str()) {
            debug!("Client protocol version: {}", version);
            // We support 2024-11-05
            if version != PROTOCOL_VERSION {
                debug!(
                    "Client using different protocol version: {} (we support {})",
                    version, PROTOCOL_VERSION
                );
            }
        }
    }

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: Some(json!({
            "protocolVersion": PROTOCOL_VERSION,
            "serverInfo": {
                "name": SERVER_NAME,
                "version": SERVER_VERSION
            },
            "capabilities": {
                "tools": {}
            }
        })),
        error: None,
    }
}

/// Handle the 'notifications/initialized' notification.
fn handle_initialized_notification(request: &JsonRpcRequest) -> JsonRpcResponse {
    // This is a notification (no id expected), but we'll respond anyway if there's an id
    info!("MCP client initialized");

    // Notifications don't require a response, but if there's an id we should respond
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

/// Handle the 'tools/list' method.
fn handle_tools_list(request: &JsonRpcRequest) -> JsonRpcResponse {
    let tools: Vec<ToolDefinition> = vec![
        // Phase 1: Stateless tools
        tools::fetch_page_definition(),
        tools::extract_text_definition(),
        tools::extract_links_definition(),
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
    ];

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: Some(json!({
            "tools": tools
        })),
        error: None,
    }
}

/// Handle the 'tools/call' method.
async fn handle_tools_call(
    request: &JsonRpcRequest,
    client: &reqwest::Client,
    sessions: &Arc<SessionManager>,
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

    let result = match tool_name {
        // Phase 1: Stateless tools
        "fetch_page" => tools::handle_fetch_page(&arguments, client).await,
        "extract_text" => tools::handle_extract_text(&arguments, client).await,
        "extract_links" => tools::handle_extract_links(&arguments, client).await,
        // Screenshot
        "screenshot_page" => tools::handle_screenshot_page(&arguments, client).await,
        // Phase 2: Stateful tools
        "open_page" => tools::handle_open_page(&arguments, client, sessions).await,
        "evaluate" => tools::handle_evaluate(&arguments, sessions).await,
        "click" => tools::handle_click(&arguments, client, sessions).await,
        "close_page" => tools::handle_close_page(&arguments, sessions).await,
        // Phase 3: Interaction tools
        "navigate_to" => tools::handle_navigate_to(&arguments, client, sessions).await,
        "type_text" => tools::handle_type_text(&arguments, client, sessions).await,
        "select_option" => tools::handle_select_option(&arguments, client, sessions).await,
        "scroll" => tools::handle_scroll(&arguments, client, sessions).await,
        "toggle" => tools::handle_toggle(&arguments, client, sessions).await,
        "clear" => tools::handle_clear(&arguments, client, sessions).await,
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

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: Some(result),
        error: None,
    }
}
