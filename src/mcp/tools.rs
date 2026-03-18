//! MCP tool definitions and handlers.
//!
//! Phase 1 implements two stateless tools:
//! - fetch_page: Fetch URL, execute JS, compile SOM, return JSON
//! - extract_text: Same pipeline, but return plain text only

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, info};

use crate::js::pipeline::{self, PipelineConfig};
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
