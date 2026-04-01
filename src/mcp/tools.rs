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

// ============================================================================
// Screenshot tool
// ============================================================================

/// Parameters for screenshot_page tool.
#[derive(Debug, Deserialize)]
struct ScreenshotPageParams {
    url: String,
    #[serde(default = "default_width")]
    width: u32,
    #[serde(default = "default_height")]
    height: u32,
    #[serde(default = "default_format")]
    format: String,
}

fn default_width() -> u32 {
    1280
}
fn default_height() -> u32 {
    720
}
fn default_format() -> String {
    "png".to_string()
}

/// Get the tool definition for screenshot_page.
pub fn screenshot_page_definition() -> ToolDefinition {
    ToolDefinition {
        name: "screenshot_page".to_string(),
        description: "Capture a pixel-perfect screenshot of a web page using headless Chrome. Requires Chrome/Chromium to be installed. If Chrome is not available, returns the page's Semantic Object Model (SOM) as structured data instead.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "URL to screenshot"
                },
                "width": {
                    "type": "integer",
                    "description": "Viewport width in pixels. Default: 1280. (Reserved for future use.)"
                },
                "height": {
                    "type": "integer",
                    "description": "Viewport height in pixels. Default: 720. (Reserved for future use.)"
                },
                "format": {
                    "type": "string",
                    "description": "Image format: png, jpeg, webp. Default: png. (Reserved for future use.)"
                }
            },
            "required": ["url"]
        }),
    }
}

/// Handle the screenshot_page tool call.
///
/// Since Plasmate doesn't have a built-in renderer yet, this fetches the page,
/// builds the SOM, and returns it as structured data with a clear message.
pub async fn handle_screenshot_page(arguments: &Value, client: &reqwest::Client) -> Value {
    use plasmate::screenshot;

    let params: ScreenshotPageParams = match serde_json::from_value(arguments.clone()) {
        Ok(p) => p,
        Err(e) => {
            return error_response(&format!("Invalid arguments: {}", e));
        }
    };

    info!(url = %params.url, "screenshot_page");

    // Fetch the page and build SOM
    let fetch_result = match fetch::fetch_url(client, &params.url, DEFAULT_TIMEOUT_MS).await {
        Ok(r) => r,
        Err(e) => {
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
            return error_response(&format!("Pipeline error: {}", e));
        }
    };

    // Try Chrome-based screenshot, fall back to SOM if Chrome not found
    let opts = screenshot::ScreenshotOptions {
        width: params.width,
        height: params.height,
        format: screenshot::Format::from_str(&params.format),
        ..Default::default()
    };

    match screenshot::capture_url(&params.url, &opts) {
        Ok(data) => {
            let base64 = base64_encode_simple(&data);
            json!({
                "content": [
                    {
                        "type": "image",
                        "data": base64,
                        "mimeType": screenshot::Format::from_str(&params.format).content_type()
                    }
                ]
            })
        }
        Err(screenshot::ScreenshotError::ChromeNotFound) => {
            // Fall back to SOM as structured data
            let fallback = screenshot::som_fallback(&page_result.som);
            json!({
                "content": [
                    {
                        "type": "text",
                        "text": serde_json::to_string(&fallback).unwrap_or_default()
                    }
                ]
            })
        }
        Err(e) => error_response(&format!("Screenshot failed: {}", e)),
    }
}

/// Simple base64 encoding for image data.
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
        // Resolve relative URLs against the current page URL
        let resolved = if href.starts_with("http://") || href.starts_with("https://") {
            href
        } else if let Ok(base) = url::Url::parse(&url) {
            base.join(&href).map(|u| u.to_string()).unwrap_or(href)
        } else {
            href
        };
        match fetch::fetch_url(client, &resolved, DEFAULT_TIMEOUT_MS).await {
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

// ============================================================================
// Phase 3: Interaction tools
// ============================================================================

/// Parameters for navigate_to tool.
#[derive(Debug, Deserialize)]
struct NavigateToParams {
    session_id: String,
    url: String,
}

/// Parameters for type_text tool.
#[derive(Debug, Deserialize)]
struct TypeTextParams {
    session_id: String,
    element_id: String,
    text: String,
    #[serde(default)]
    append: bool,
}

/// Parameters for select_option tool.
#[derive(Debug, Deserialize)]
struct SelectOptionParams {
    session_id: String,
    element_id: String,
    value: String,
}

/// Parameters for scroll tool.
#[derive(Debug, Deserialize)]
struct ScrollParams {
    session_id: String,
    #[serde(default = "default_direction")]
    direction: String,
    #[serde(default = "default_pixels")]
    pixels: i32,
    #[serde(default)]
    element_id: Option<String>,
}

fn default_direction() -> String {
    "down".to_string()
}

fn default_pixels() -> i32 {
    300
}

/// Get the tool definition for navigate_to.
pub fn navigate_to_definition() -> ToolDefinition {
    ToolDefinition {
        name: "navigate_to".to_string(),
        description: "Navigate to a new URL within an existing browser session. Returns the updated page SOM.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "session_id": {
                    "type": "string",
                    "description": "Session ID from open_page"
                },
                "url": {
                    "type": "string",
                    "description": "URL to navigate to"
                }
            },
            "required": ["session_id", "url"]
        }),
    }
}

/// Get the tool definition for type_text.
pub fn type_text_definition() -> ToolDefinition {
    ToolDefinition {
        name: "type_text".to_string(),
        description: "Type text into a form input or textarea by its SOM element ID. Returns the updated page SOM.".to_string(),
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
                },
                "text": {
                    "type": "string",
                    "description": "Text to type into the element"
                },
                "append": {
                    "type": "boolean",
                    "description": "If true, append to existing value instead of replacing. Default: false."
                }
            },
            "required": ["session_id", "element_id", "text"]
        }),
    }
}

/// Get the tool definition for select_option.
pub fn select_option_definition() -> ToolDefinition {
    ToolDefinition {
        name: "select_option".to_string(),
        description: "Select an option in a <select> dropdown by element ID and option value or label. Returns the updated page SOM.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "session_id": {
                    "type": "string",
                    "description": "Session ID from open_page"
                },
                "element_id": {
                    "type": "string",
                    "description": "Element ID of the <select> from SOM (e.g. 'e5')"
                },
                "value": {
                    "type": "string",
                    "description": "Option value or visible text to select"
                }
            },
            "required": ["session_id", "element_id", "value"]
        }),
    }
}

/// Get the tool definition for scroll.
pub fn scroll_definition() -> ToolDefinition {
    ToolDefinition {
        name: "scroll".to_string(),
        description: "Scroll the page or a specific element into view. Returns the updated page SOM with scroll position.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "session_id": {
                    "type": "string",
                    "description": "Session ID from open_page"
                },
                "direction": {
                    "type": "string",
                    "enum": ["down", "up", "top", "bottom"],
                    "description": "Scroll direction. Default: 'down'."
                },
                "pixels": {
                    "type": "integer",
                    "description": "Number of pixels to scroll for up/down. Default: 300."
                },
                "element_id": {
                    "type": "string",
                    "description": "If provided, scroll this element into view instead of scrolling the page."
                }
            },
            "required": ["session_id"]
        }),
    }
}

/// Handle the navigate_to tool call.
pub async fn handle_navigate_to(
    arguments: &Value,
    client: &reqwest::Client,
    sessions: &Arc<SessionManager>,
) -> Value {
    let params: NavigateToParams = match serde_json::from_value(arguments.clone()) {
        Ok(p) => p,
        Err(e) => {
            return error_response(&format!("Invalid arguments: {}", e));
        }
    };

    info!(session_id = %params.session_id, url = %params.url, "navigate_to");

    // Verify session exists
    let exists = sessions
        .with_session(&params.session_id, |_session| {})
        .await;
    if exists.is_none() {
        return error_response(&format!("Session not found: {}", params.session_id));
    }

    // Fetch the new URL
    let fetch_result = match fetch::fetch_url(client, &params.url, DEFAULT_TIMEOUT_MS).await {
        Ok(r) => r,
        Err(e) => {
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
            return error_response(&format!("Pipeline error: {}", e));
        }
    };

    // Update session state
    let som_json = sessions
        .with_session(&params.session_id, |session| {
            session.target.current_url = Some(fetch_result.url.clone());
            session.target.current_html = Some(fetch_result.html.clone());
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

    json!({
        "content": [
            {
                "type": "text",
                "text": json!({
                    "session_id": params.session_id,
                    "title": page_result.som.title,
                    "url": fetch_result.url,
                    "regions": som_json.get("regions")
                }).to_string()
            }
        ]
    })
}

/// Handle the type_text tool call.
pub async fn handle_type_text(
    arguments: &Value,
    client: &reqwest::Client,
    sessions: &Arc<SessionManager>,
) -> Value {
    let params: TypeTextParams = match serde_json::from_value(arguments.clone()) {
        Ok(p) => p,
        Err(e) => {
            return error_response(&format!("Invalid arguments: {}", e));
        }
    };

    info!(session_id = %params.session_id, element_id = %params.element_id, "type_text");

    // Get session data
    let session_data = sessions
        .with_session(&params.session_id, |session| {
            let effective_html = session.target.effective_html.clone();
            let url = session.target.current_url.clone();
            (effective_html, url)
        })
        .await;

    let (effective_html, url) = match session_data {
        Some((Some(html), Some(url))) => (html, url),
        Some((None, _)) | Some((_, None)) => {
            return error_response("No page loaded in session");
        }
        None => {
            return error_response(&format!("Session not found: {}", params.session_id));
        }
    };

    // Run JS to type text into the element
    let element_id = params.element_id.clone();
    let text = params.text.clone();
    let append = params.append;
    let url_clone = url.clone();
    let type_result = tokio::task::spawn_blocking(move || {
        let mut runtime = JsRuntime::new(RuntimeConfig {
            inject_dom_shim: true,
            execute_inline_scripts: false,
            ..Default::default()
        });

        runtime.bootstrap_dom(&effective_html, &url_clone);

        let escaped_text = text
            .replace('\\', "\\\\")
            .replace('\'', "\\'")
            .replace('\n', "\\n");

        let type_js = format!(
            r#"
            (function() {{
                var el = document.querySelector('[data-plasmate-id="{}"]');
                if (!el) {{
                    return JSON.stringify({{ error: 'Element not found in DOM' }});
                }}
                if ({}) {{
                    el.value = (el.value || '') + '{}';
                }} else {{
                    el.value = '{}';
                }}
                var inputEvt = new Event('input', {{ bubbles: true }});
                el.dispatchEvent(inputEvt);
                var changeEvt = new Event('change', {{ bubbles: true }});
                el.dispatchEvent(changeEvt);
                return JSON.stringify({{ typed: true }});
            }})()
            "#,
            element_id,
            if append { "true" } else { "false" },
            escaped_text,
            escaped_text
        );

        let result = runtime.eval(&type_js).map_err(|e| e.to_string())?;
        let updated_html = runtime
            .eval("document.documentElement.outerHTML")
            .map_err(|e| e.to_string())?;

        Ok::<(String, String), String>((result, updated_html))
    })
    .await;

    let (result_json, updated_html) = match type_result {
        Ok(Ok((result, html))) => (result, html),
        Ok(Err(e)) => {
            return error_response(&format!("Type failed: {}", e));
        }
        Err(e) => {
            return error_response(&format!("Execution error: {}", e));
        }
    };

    // Check for errors from JS
    let result_data: Value = serde_json::from_str(&result_json).unwrap_or(json!({}));
    if let Some(err) = result_data.get("error").and_then(|v| v.as_str()) {
        return error_response(err);
    }

    // Re-process the page to get updated SOM
    let pipeline_config = PipelineConfig {
        execute_js: true,
        fetch_external_scripts: true,
        ..Default::default()
    };

    let page_result =
        match pipeline::process_page_async(&updated_html, &url, &pipeline_config, client).await {
            Ok(r) => r,
            Err(e) => {
                return error_response(&format!("Pipeline error: {}", e));
            }
        };

    // Update session
    let som_json = sessions
        .with_session(&params.session_id, |session| {
            session.target.current_url = Some(url.clone());
            session.target.current_html = Some(updated_html.clone());
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

    json!({
        "content": [
            {
                "type": "text",
                "text": json!({
                    "title": page_result.som.title,
                    "url": url,
                    "regions": som_json.get("regions")
                }).to_string()
            }
        ]
    })
}

/// Handle the select_option tool call.
pub async fn handle_select_option(
    arguments: &Value,
    client: &reqwest::Client,
    sessions: &Arc<SessionManager>,
) -> Value {
    let params: SelectOptionParams = match serde_json::from_value(arguments.clone()) {
        Ok(p) => p,
        Err(e) => {
            return error_response(&format!("Invalid arguments: {}", e));
        }
    };

    info!(session_id = %params.session_id, element_id = %params.element_id, value = %params.value, "select_option");

    // Get session data
    let session_data = sessions
        .with_session(&params.session_id, |session| {
            let effective_html = session.target.effective_html.clone();
            let url = session.target.current_url.clone();
            (effective_html, url)
        })
        .await;

    let (effective_html, url) = match session_data {
        Some((Some(html), Some(url))) => (html, url),
        Some((None, _)) | Some((_, None)) => {
            return error_response("No page loaded in session");
        }
        None => {
            return error_response(&format!("Session not found: {}", params.session_id));
        }
    };

    // Run JS to select option
    let element_id = params.element_id.clone();
    let value = params.value.clone();
    let url_clone = url.clone();
    let select_result = tokio::task::spawn_blocking(move || {
        let mut runtime = JsRuntime::new(RuntimeConfig {
            inject_dom_shim: true,
            execute_inline_scripts: false,
            ..Default::default()
        });

        runtime.bootstrap_dom(&effective_html, &url_clone);

        let escaped_value = value
            .replace('\\', "\\\\")
            .replace('\'', "\\'");

        let select_js = format!(
            r#"
            (function() {{
                var el = document.querySelector('[data-plasmate-id="{}"]');
                if (!el) {{
                    return JSON.stringify({{ error: 'Element not found in DOM' }});
                }}
                if (el.tagName !== 'SELECT') {{
                    return JSON.stringify({{ error: 'Element is not a <select>' }});
                }}
                var found = false;
                for (var i = 0; i < el.options.length; i++) {{
                    if (el.options[i].value === '{}' || el.options[i].text === '{}') {{
                        el.selectedIndex = i;
                        found = true;
                        break;
                    }}
                }}
                if (!found) {{
                    return JSON.stringify({{ error: 'Option not found: {}' }});
                }}
                var changeEvt = new Event('change', {{ bubbles: true }});
                el.dispatchEvent(changeEvt);
                return JSON.stringify({{ selected: true, value: el.value }});
            }})()
            "#,
            element_id,
            escaped_value,
            escaped_value,
            escaped_value
        );

        let result = runtime.eval(&select_js).map_err(|e| e.to_string())?;
        let updated_html = runtime
            .eval("document.documentElement.outerHTML")
            .map_err(|e| e.to_string())?;

        Ok::<(String, String), String>((result, updated_html))
    })
    .await;

    let (result_json, updated_html) = match select_result {
        Ok(Ok((result, html))) => (result, html),
        Ok(Err(e)) => {
            return error_response(&format!("Select failed: {}", e));
        }
        Err(e) => {
            return error_response(&format!("Execution error: {}", e));
        }
    };

    // Check for errors from JS
    let result_data: Value = serde_json::from_str(&result_json).unwrap_or(json!({}));
    if let Some(err) = result_data.get("error").and_then(|v| v.as_str()) {
        return error_response(err);
    }

    // Re-process the page
    let pipeline_config = PipelineConfig {
        execute_js: true,
        fetch_external_scripts: true,
        ..Default::default()
    };

    let page_result =
        match pipeline::process_page_async(&updated_html, &url, &pipeline_config, client).await {
            Ok(r) => r,
            Err(e) => {
                return error_response(&format!("Pipeline error: {}", e));
            }
        };

    // Update session
    let som_json = sessions
        .with_session(&params.session_id, |session| {
            session.target.current_url = Some(url.clone());
            session.target.current_html = Some(updated_html.clone());
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

    json!({
        "content": [
            {
                "type": "text",
                "text": json!({
                    "title": page_result.som.title,
                    "url": url,
                    "regions": som_json.get("regions")
                }).to_string()
            }
        ]
    })
}

/// Handle the scroll tool call.
pub async fn handle_scroll(
    arguments: &Value,
    client: &reqwest::Client,
    sessions: &Arc<SessionManager>,
) -> Value {
    let params: ScrollParams = match serde_json::from_value(arguments.clone()) {
        Ok(p) => p,
        Err(e) => {
            return error_response(&format!("Invalid arguments: {}", e));
        }
    };

    info!(session_id = %params.session_id, direction = %params.direction, "scroll");

    // Get session data
    let session_data = sessions
        .with_session(&params.session_id, |session| {
            let effective_html = session.target.effective_html.clone();
            let url = session.target.current_url.clone();
            (effective_html, url)
        })
        .await;

    let (effective_html, url) = match session_data {
        Some((Some(html), Some(url))) => (html, url),
        Some((None, _)) | Some((_, None)) => {
            return error_response("No page loaded in session");
        }
        None => {
            return error_response(&format!("Session not found: {}", params.session_id));
        }
    };

    // Run JS to scroll
    let direction = params.direction.clone();
    let pixels = params.pixels;
    let element_id = params.element_id.clone();
    let url_clone = url.clone();
    let scroll_result = tokio::task::spawn_blocking(move || {
        let mut runtime = JsRuntime::new(RuntimeConfig {
            inject_dom_shim: true,
            execute_inline_scripts: false,
            ..Default::default()
        });

        runtime.bootstrap_dom(&effective_html, &url_clone);

        let scroll_js = if let Some(ref eid) = element_id {
            format!(
                r#"
                (function() {{
                    var el = document.querySelector('[data-plasmate-id="{}"]');
                    if (!el) {{
                        return JSON.stringify({{ error: 'Element not found in DOM' }});
                    }}
                    el.scrollIntoView({{ behavior: 'instant', block: 'center' }});
                    return JSON.stringify({{ scrolled: true, scrollTop: document.documentElement.scrollTop || 0 }});
                }})()
                "#,
                eid
            )
        } else {
            let scroll_action = match direction.as_str() {
                "up" => format!("window.scrollBy(0, -{})", pixels),
                "top" => "window.scrollTo(0, 0)".to_string(),
                "bottom" => "window.scrollTo(0, document.body.scrollHeight)".to_string(),
                _ => format!("window.scrollBy(0, {})", pixels), // "down" is default
            };
            format!(
                r#"
                (function() {{
                    {};
                    return JSON.stringify({{ scrolled: true, scrollTop: document.documentElement.scrollTop || 0 }});
                }})()
                "#,
                scroll_action
            )
        };

        let result = runtime.eval(&scroll_js).map_err(|e| e.to_string())?;
        let updated_html = runtime
            .eval("document.documentElement.outerHTML")
            .map_err(|e| e.to_string())?;

        Ok::<(String, String), String>((result, updated_html))
    })
    .await;

    let (result_json, updated_html) = match scroll_result {
        Ok(Ok((result, html))) => (result, html),
        Ok(Err(e)) => {
            return error_response(&format!("Scroll failed: {}", e));
        }
        Err(e) => {
            return error_response(&format!("Execution error: {}", e));
        }
    };

    // Check for errors from JS
    let result_data: Value = serde_json::from_str(&result_json).unwrap_or(json!({}));
    if let Some(err) = result_data.get("error").and_then(|v| v.as_str()) {
        return error_response(err);
    }

    let scroll_top = result_data
        .get("scrollTop")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    // Re-process the page
    let pipeline_config = PipelineConfig {
        execute_js: true,
        fetch_external_scripts: true,
        ..Default::default()
    };

    let page_result =
        match pipeline::process_page_async(&updated_html, &url, &pipeline_config, client).await {
            Ok(r) => r,
            Err(e) => {
                return error_response(&format!("Pipeline error: {}", e));
            }
        };

    // Update session
    let som_json = sessions
        .with_session(&params.session_id, |session| {
            session.target.current_url = Some(url.clone());
            session.target.current_html = Some(updated_html.clone());
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

    json!({
        "content": [
            {
                "type": "text",
                "text": json!({
                    "title": page_result.som.title,
                    "url": url,
                    "scroll_position": scroll_top,
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
