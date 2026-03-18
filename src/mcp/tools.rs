//! MCP tool definitions and handlers.
//!
//! Phase 1 implements two stateless tools:
//! - fetch_page: Fetch URL, execute JS, compile SOM, return JSON
//! - extract_text: Same pipeline, but return plain text only
//!
//! Phase 2 implements four stateful tools:
//! - open_page: Open a URL in a persistent browser session
//! - evaluate: Run JavaScript in a session
//! - click: Click an element by SOM element ID
//! - close_page: Close a session

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, info, warn};

use super::sessions::SessionManager;
use crate::js::pipeline::{self, PipelineConfig};
use crate::js::runtime::{JsRuntime, RuntimeConfig};
use crate::network::fetch;

/// Default timeout for fetching pages (30 seconds).
const DEFAULT_TIMEOUT_MS: u64 = 30000;

/// MCP tool definition structure.
#[derive(Debug, Serialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Parameters for fetch_page tool.
#[derive(Debug, Deserialize)]
struct FetchPageParams {
    url: String,
    #[serde(default)]
    budget: Option<usize>,
    #[serde(default = "default_javascript")]
    javascript: bool,
}

fn default_javascript() -> bool {
    true
}

/// Parameters for extract_text tool.
#[derive(Debug, Deserialize)]
struct ExtractTextParams {
    url: String,
    #[serde(default)]
    max_chars: Option<usize>,
}

/// Get the tool definition for fetch_page.
pub fn fetch_page_definition() -> ToolDefinition {
    ToolDefinition {
        name: "fetch_page".to_string(),
        description: "Fetch a web page and return its Semantic Object Model (SOM) - a structured, token-efficient representation of the page content. Use this instead of raw HTML fetching for 10x token savings.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "URL to fetch"
                },
                "budget": {
                    "type": "integer",
                    "description": "Maximum output tokens. SOM will be truncated to fit. Default: no limit."
                },
                "javascript": {
                    "type": "boolean",
                    "description": "Enable JavaScript execution for dynamic/SPA pages. Default: true."
                }
            },
            "required": ["url"]
        }),
    }
}

/// Get the tool definition for extract_text.
pub fn extract_text_definition() -> ToolDefinition {
    ToolDefinition {
        name: "extract_text".to_string(),
        description: "Fetch a web page and return only the clean, readable text content. No HTML, no structure - just the text a human would read.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "URL to fetch"
                },
                "max_chars": {
                    "type": "integer",
                    "description": "Maximum characters to return. Default: no limit."
                }
            },
            "required": ["url"]
        }),
    }
}

/// Handle the fetch_page tool call.
pub async fn handle_fetch_page(arguments: &Value, client: &reqwest::Client) -> Value {
    // Parse arguments
    let params: FetchPageParams = match serde_json::from_value(arguments.clone()) {
        Ok(p) => p,
        Err(e) => {
            return error_response(&format!("Invalid arguments: {}", e));
        }
    };

    info!(url = %params.url, javascript = params.javascript, "fetch_page");

    // Fetch the page
    let fetch_result = match fetch::fetch_url(client, &params.url, DEFAULT_TIMEOUT_MS).await {
        Ok(r) => r,
        Err(e) => {
            return error_response(&format!("Failed to fetch {}: {}", params.url, e));
        }
    };

    debug!(
        url = %fetch_result.url,
        status = fetch_result.status,
        html_bytes = fetch_result.html_bytes,
        load_ms = fetch_result.load_ms,
        "Fetched"
    );

    // Process through the pipeline
    let pipeline_config = PipelineConfig {
        execute_js: params.javascript,
        fetch_external_scripts: params.javascript,
        ..Default::default()
    };

    let page_result = match pipeline::process_page_async(
        &fetch_result.html,
        &fetch_result.url,
        &pipeline_config,
        client,
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            return error_response(&format!("Pipeline error: {}", e));
        }
    };

    debug!(
        som_bytes = page_result.som.meta.som_bytes,
        element_count = page_result.som.meta.element_count,
        interactive_count = page_result.som.meta.interactive_count,
        "SOM compiled"
    );

    // Serialize SOM to JSON
    let som_json = match serde_json::to_value(&page_result.som) {
        Ok(v) => v,
        Err(e) => {
            return error_response(&format!("Failed to serialize SOM: {}", e));
        }
    };

    // Apply budget truncation if specified
    let result = if let Some(budget) = params.budget {
        // Rough approximation: 1 token ≈ 4 characters
        let max_chars = budget * 4;
        let som_str = som_json.to_string();
        if som_str.len() > max_chars {
            // Return truncated JSON with a note
            json!({
                "content": [
                    {
                        "type": "text",
                        "text": format!("{{\"truncated\": true, \"original_bytes\": {}, \"message\": \"SOM exceeded budget of {} tokens\"}}", som_str.len(), budget)
                    }
                ]
            })
        } else {
            json!({
                "content": [
                    {
                        "type": "text",
                        "text": som_str
                    }
                ]
            })
        }
    } else {
        json!({
            "content": [
                {
                    "type": "text",
                    "text": som_json.to_string()
                }
            ]
        })
    };

    result
}

/// Handle the extract_text tool call.
pub async fn handle_extract_text(arguments: &Value, client: &reqwest::Client) -> Value {
    // Parse arguments
    let params: ExtractTextParams = match serde_json::from_value(arguments.clone()) {
        Ok(p) => p,
        Err(e) => {
            return error_response(&format!("Invalid arguments: {}", e));
        }
    };

    info!(url = %params.url, "extract_text");

    // Fetch the page
    let fetch_result = match fetch::fetch_url(client, &params.url, DEFAULT_TIMEOUT_MS).await {
        Ok(r) => r,
        Err(e) => {
            return error_response(&format!("Failed to fetch {}: {}", params.url, e));
        }
    };

    debug!(
        url = %fetch_result.url,
        status = fetch_result.status,
        html_bytes = fetch_result.html_bytes,
        "Fetched"
    );

    // Process through the pipeline with JS enabled
    let pipeline_config = PipelineConfig {
        execute_js: true,
        fetch_external_scripts: true,
        ..Default::default()
    };

    let page_result = match pipeline::process_page_async(
        &fetch_result.html,
        &fetch_result.url,
        &pipeline_config,
        client,
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            return error_response(&format!("Pipeline error: {}", e));
        }
    };

    // Extract text from all regions
    let mut text_parts: Vec<String> = Vec::new();

    // Add title if present
    if !page_result.som.title.is_empty() {
        text_parts.push(page_result.som.title.clone());
        text_parts.push(String::new()); // Empty line after title
    }

    // Extract text from each region
    for region in &page_result.som.regions {
        for element in &region.elements {
            extract_element_text(element, &mut text_parts);
        }
    }

    let mut text = text_parts.join("\n");

    // Apply max_chars limit if specified
    if let Some(max_chars) = params.max_chars {
        if text.len() > max_chars {
            text.truncate(max_chars);
            // Try to truncate at a word boundary
            if let Some(last_space) = text.rfind(' ') {
                text.truncate(last_space);
            }
            text.push_str("...");
        }
    }

    json!({
        "content": [
            {
                "type": "text",
                "text": text
            }
        ]
    })
}

/// Recursively extract text from a SOM element.
fn extract_element_text(element: &crate::som::types::Element, parts: &mut Vec<String>) {
    // Add element text if present
    if let Some(text) = &element.text {
        if !text.is_empty() {
            parts.push(text.clone());
        }
    }

    // Handle list items
    if let Some(attrs) = &element.attrs {
        if let Some(items) = attrs.get("items") {
            if let Some(items_arr) = items.as_array() {
                for item in items_arr {
                    if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                        parts.push(format!("• {}", text));
                    }
                }
            }
        }
    }

    // Recurse into children
    if let Some(children) = &element.children {
        for child in children {
            extract_element_text(child, parts);
        }
    }
}

/// Create an MCP error response.
fn error_response(message: &str) -> Value {
    json!({
        "isError": true,
        "content": [
            {
                "type": "text",
                "text": message
            }
        ]
    })
}

// ============================================================================
// Phase 2: Stateful browser tools
// ============================================================================

/// Parameters for open_page tool.
#[derive(Debug, Deserialize)]
struct OpenPageParams {
    url: String,
}

/// Parameters for evaluate tool.
#[derive(Debug, Deserialize)]
struct EvaluateParams {
    session_id: String,
    expression: String,
}

/// Parameters for click tool.
#[derive(Debug, Deserialize)]
struct ClickParams {
    session_id: String,
    element_id: String,
}

/// Parameters for close_page tool.
#[derive(Debug, Deserialize)]
struct ClosePageParams {
    session_id: String,
}

/// Get the tool definition for open_page.
pub fn open_page_definition() -> ToolDefinition {
    ToolDefinition {
        name: "open_page".to_string(),
        description: "Open a web page in a persistent browser session. Returns a session ID and the initial SOM. Use with click, type, and evaluate for multi-step interactions.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "URL to open"
                }
            },
            "required": ["url"]
        }),
    }
}

/// Get the tool definition for evaluate.
pub fn evaluate_definition() -> ToolDefinition {
    ToolDefinition {
        name: "evaluate".to_string(),
        description: "Execute JavaScript in the page context and return the result. Use for custom data extraction or page manipulation.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "session_id": {
                    "type": "string",
                    "description": "Session ID from open_page"
                },
                "expression": {
                    "type": "string",
                    "description": "JavaScript expression to evaluate. Return value is serialized to JSON."
                }
            },
            "required": ["session_id", "expression"]
        }),
    }
}

/// Get the tool definition for click.
pub fn click_definition() -> ToolDefinition {
    ToolDefinition {
        name: "click".to_string(),
        description: "Click an element on the page by its SOM element ID. Returns the updated page SOM after the click.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "session_id": {
                    "type": "string",
                    "description": "Session ID from open_page"
                },
                "element_id": {
                    "type": "string",
                    "description": "Element ID from SOM (e.g. 'e5')"
                }
            },
            "required": ["session_id", "element_id"]
        }),
    }
}

/// Get the tool definition for close_page.
pub fn close_page_definition() -> ToolDefinition {
    ToolDefinition {
        name: "close_page".to_string(),
        description: "Close a browser session and free resources.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "session_id": {
                    "type": "string",
                    "description": "Session ID to close"
                }
            },
            "required": ["session_id"]
        }),
    }
}

/// Handle the open_page tool call.
pub async fn handle_open_page(
    arguments: &Value,
    client: &reqwest::Client,
    sessions: &Arc<SessionManager>,
) -> Value {
    // Parse arguments
    let params: OpenPageParams = match serde_json::from_value(arguments.clone()) {
        Ok(p) => p,
        Err(e) => {
            return error_response(&format!("Invalid arguments: {}", e));
        }
    };

    info!(url = %params.url, "open_page");

    // Create a new session
    let session_id = match sessions.create_session().await {
        Ok(id) => id,
        Err(e) => {
            return error_response(&e);
        }
    };

    // Fetch and navigate using the shared HTTP client
    let fetch_result = match fetch::fetch_url(client, &params.url, DEFAULT_TIMEOUT_MS).await {
        Ok(r) => r,
        Err(e) => {
            // Clean up the session on failure
            sessions.close_session(&session_id).await;
            return error_response(&format!("Failed to fetch {}: {}", params.url, e));
        }
    };

    let pipeline_config = PipelineConfig {
        execute_js: true,
        fetch_external_scripts: true,
        ..Default::default()
    };

    let page_result = match pipeline::process_page_async(
        &fetch_result.html,
        &fetch_result.url,
        &pipeline_config,
        client,
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            sessions.close_session(&session_id).await;
            return error_response(&format!("Pipeline error: {}", e));
        }
    };

    // Store the result in the session
    let som_json = sessions
        .with_session(&session_id, |session| {
            session.target.current_url = Some(fetch_result.url.clone());
            session.target.current_html = Some(fetch_result.html.clone());
            session.target.effective_html = Some(page_result.effective_html.clone());
            session.target.current_som = Some(page_result.som.clone());

            // Return SOM JSON
            serde_json::to_value(&page_result.som).ok()
        })
        .await;

    let som_json = match som_json.flatten() {
        Some(v) => v,
        None => {
            sessions.close_session(&session_id).await;
            return error_response("Failed to serialize SOM");
        }
    };

    // Return session ID + SOM
    json!({
        "content": [
            {
                "type": "text",
                "text": json!({
                    "session_id": session_id,
                    "title": page_result.som.title,
                    "url": fetch_result.url,
                    "regions": som_json.get("regions")
                }).to_string()
            }
        ]
    })
}

/// Handle the evaluate tool call.
///
/// Runs JavaScript in the session's page context using V8.
/// V8's OwnedIsolate is !Send, so we use spawn_blocking.
pub async fn handle_evaluate(arguments: &Value, sessions: &Arc<SessionManager>) -> Value {
    // Parse arguments
    let params: EvaluateParams = match serde_json::from_value(arguments.clone()) {
        Ok(p) => p,
        Err(e) => {
            return error_response(&format!("Invalid arguments: {}", e));
        }
    };

    info!(session_id = %params.session_id, expression = %params.expression, "evaluate");

    // Get the effective HTML and URL from the session
    let session_data = sessions
        .with_session(&params.session_id, |session| {
            let effective_html = session.target.effective_html.clone();
            let url = session.target.current_url.clone();
            (effective_html, url)
        })
        .await;

    let (effective_html, url) = match session_data {
        Some((Some(html), url)) => (html, url.unwrap_or_else(|| "about:blank".to_string())),
        Some((None, _)) => {
            return error_response("No page loaded in session");
        }
        None => {
            return error_response(&format!("Session not found: {}", params.session_id));
        }
    };

    // Run JS evaluation in a blocking task (V8 is !Send)
    let expression = params.expression.clone();
    let eval_result = tokio::task::spawn_blocking(move || {
        let mut runtime = JsRuntime::new(RuntimeConfig {
            inject_dom_shim: true,
            execute_inline_scripts: false, // Don't re-execute page scripts
            ..Default::default()
        });

        // Bootstrap DOM from effective HTML
        runtime.bootstrap_dom(&effective_html, &url);

        // Wrap expression to properly serialize objects/arrays
        let wrapped_expr = format!(
            "(function() {{ var __r = ({}); return typeof __r === 'object' && __r !== null ? JSON.stringify(__r) : __r; }})()",
            expression
        );

        runtime.eval(&wrapped_expr)
    })
    .await;

    match eval_result {
        Ok(Ok(result)) => {
            // Parse result - try JSON first, then string
            let value = if result == "undefined" || result.is_empty() {
                Value::Null
            } else if let Ok(json_val) = serde_json::from_str::<Value>(&result) {
                json_val
            } else {
                Value::String(result)
            };

            json!({
                "content": [
                    {
                        "type": "text",
                        "text": json!({
                            "result": value
                        }).to_string()
                    }
                ]
            })
        }
        Ok(Err(e)) => error_response(&format!("JavaScript error: {}", e)),
        Err(e) => error_response(&format!("Execution error: {}", e)),
    }
}

/// Handle the click tool call.
///
/// Simulates a click on an element by:
/// 1. Finding the element by SOM ID
/// 2. Dispatching a click event via JS
/// 3. Re-processing the page to get updated SOM
pub async fn handle_click(
    arguments: &Value,
    client: &reqwest::Client,
    sessions: &Arc<SessionManager>,
) -> Value {
    // Parse arguments
    let params: ClickParams = match serde_json::from_value(arguments.clone()) {
        Ok(p) => p,
        Err(e) => {
            return error_response(&format!("Invalid arguments: {}", e));
        }
    };

    info!(session_id = %params.session_id, element_id = %params.element_id, "click");

    // Get session data
    let session_data = sessions
        .with_session(&params.session_id, |session| {
            let effective_html = session.target.effective_html.clone();
            let url = session.target.current_url.clone();
            let som = session.target.current_som.clone();
            (effective_html, url, som)
        })
        .await;

    let (effective_html, url, som) = match session_data {
        Some((Some(html), Some(url), Some(som))) => (html, url, som),
        Some((None, _, _)) | Some((_, None, _)) | Some((_, _, None)) => {
            return error_response("No page loaded in session");
        }
        None => {
            return error_response(&format!("Session not found: {}", params.session_id));
        }
    };

    // Find the element in the SOM
    let element = som
        .regions
        .iter()
        .flat_map(|r| &r.elements)
        .find(|e| e.id == params.element_id);

    if element.is_none() {
        return error_response(&format!("Element not found: {}", params.element_id));
    }

    let element = element.unwrap();

    // Check if element is clickable (has actions or is interactive)
    let is_interactive = element.role.is_interactive();
    if !is_interactive {
        warn!(element_id = %params.element_id, role = ?element.role, "Clicking non-interactive element");
    }

    // Generate JavaScript to simulate click
    // We look for the element by its data-plasmate-id attribute or by finding
    // an element that matches the SOM element's characteristics
    let element_id = params.element_id.clone();
    let click_js = format!(
        r#"
        (function() {{
            // Try to find element by data attribute first
            var el = document.querySelector('[data-plasmate-id="{}"]');

            // If not found, try other strategies based on element characteristics
            if (!el) {{
                // Try by tag and text content for links/buttons
                var allEls = document.querySelectorAll('a, button, input[type="submit"], [role="button"]');
                for (var i = 0; i < allEls.length; i++) {{
                    if (allEls[i].textContent && allEls[i].textContent.trim() === '{}') {{
                        el = allEls[i];
                        break;
                    }}
                }}
            }}

            if (el) {{
                // Dispatch click event
                var evt = new MouseEvent('click', {{
                    bubbles: true,
                    cancelable: true,
                    view: window
                }});
                el.dispatchEvent(evt);

                // For links, check if we need to navigate
                if (el.tagName === 'A' && el.href) {{
                    return JSON.stringify({{ navigated: true, href: el.href }});
                }}
                return JSON.stringify({{ clicked: true }});
            }}
            return JSON.stringify({{ error: 'Element not found in DOM' }});
        }})()
        "#,
        element_id,
        element
            .text
            .as_deref()
            .unwrap_or("")
            .replace('\'', "\\'")
            .replace('\n', " ")
    );

    // Execute the click JS
    let url_clone = url.clone();
    let effective_html_clone = effective_html.clone();
    let click_result = tokio::task::spawn_blocking(move || {
        let mut runtime = JsRuntime::new(RuntimeConfig {
            inject_dom_shim: true,
            execute_inline_scripts: false,
            ..Default::default()
        });

        runtime.bootstrap_dom(&effective_html_clone, &url_clone);

        let result = runtime.eval(&click_js).map_err(|e| e.to_string())?;

        // Get the updated HTML after click handlers ran
        let updated_html = runtime
            .eval("document.documentElement.outerHTML")
            .map_err(|e| e.to_string())?;

        Ok::<(String, String), String>((result, updated_html))
    })
    .await;

    let (click_result_json, updated_html) = match click_result {
        Ok(Ok((result, html))) => (result, html),
        Ok(Err(e)) => {
            return error_response(&format!("Click failed: {}", e));
        }
        Err(e) => {
            return error_response(&format!("Execution error: {}", e));
        }
    };

    // Parse click result to check for navigation
    let click_data: Value = serde_json::from_str(&click_result_json).unwrap_or(json!({}));

    // Check if we need to navigate to a new URL
    let new_url = if click_data
        .get("navigated")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        click_data
            .get("href")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    } else {
        None
    };

    // If we navigated, fetch the new page
    let (final_html, final_url) = if let Some(href) = new_url {
        match fetch::fetch_url(client, &href, DEFAULT_TIMEOUT_MS).await {
            Ok(r) => (r.html, r.url),
            Err(e) => {
                return error_response(&format!("Navigation failed: {}", e));
            }
        }
    } else {
        // Use the updated HTML from click handlers
        (updated_html, url.clone())
    };

    // Re-process the page to get updated SOM
    let pipeline_config = PipelineConfig {
        execute_js: true,
        fetch_external_scripts: true,
        ..Default::default()
    };

    let page_result =
        match pipeline::process_page_async(&final_html, &final_url, &pipeline_config, client).await
        {
            Ok(r) => r,
            Err(e) => {
                return error_response(&format!("Pipeline error: {}", e));
            }
        };

    // Update session with new state
    let som_json = sessions
        .with_session(&params.session_id, |session| {
            session.target.current_url = Some(final_url.clone());
            session.target.current_html = Some(final_html.clone());
            session.target.effective_html = Some(page_result.effective_html.clone());
            session.target.current_som = Some(page_result.som.clone());

            serde_json::to_value(&page_result.som).ok()
        })
        .await;

    let som_json = match som_json.flatten() {
        Some(v) => v,
        None => {
            return error_response("Failed to serialize SOM");
        }
    };

    // Return updated SOM
    json!({
        "content": [
            {
                "type": "text",
                "text": json!({
                    "title": page_result.som.title,
                    "url": final_url,
                    "regions": som_json.get("regions")
                }).to_string()
            }
        ]
    })
}

/// Handle the close_page tool call.
pub async fn handle_close_page(arguments: &Value, sessions: &Arc<SessionManager>) -> Value {
    // Parse arguments
    let params: ClosePageParams = match serde_json::from_value(arguments.clone()) {
        Ok(p) => p,
        Err(e) => {
            return error_response(&format!("Invalid arguments: {}", e));
        }
    };

    info!(session_id = %params.session_id, "close_page");

    let closed = sessions.close_session(&params.session_id).await;

    if closed {
        json!({
            "content": [
                {
                    "type": "text",
                    "text": json!({
                        "closed": true,
                        "session_id": params.session_id
                    }).to_string()
                }
            ]
        })
    } else {
        error_response(&format!("Session not found: {}", params.session_id))
    }
}
