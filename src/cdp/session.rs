//! CDP session state - maps CDP concepts to our internal pipeline.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use reqwest::cookie::Jar;
use reqwest::Client;

use crate::cdp::cookies::CookieJar;
use crate::js::pipeline::PipelineConfig;
use crate::js::runtime::{JsRuntime, RuntimeConfig};
use crate::network::fetch;
use crate::network::intercept::{
    InterceptAction, NetworkInterceptor, ResourceType as InterceptResourceType,
};
use crate::som::metadata::StructuredData;
use crate::som::types::{Element, ElementRole, Som};

static NODE_COUNTER: AtomicU64 = AtomicU64::new(1);
static TARGET_COUNTER: AtomicU64 = AtomicU64::new(1);

/// CDP browsing context - maps to a "target" in CDP terms.
pub struct CdpTarget {
    pub target_id: String,
    pub session_id: String,
    pub client: Client,
    pub reqwest_jar: Arc<Jar>,
    pub cookie_jar: CookieJar,
    pub timeout_ms: u64,
    pub user_agent: String,
    pub extra_headers: HashMap<String, String>,

    // Page state
    pub current_url: Option<String>,
    pub current_html: Option<String>,
    /// The effective HTML after JS execution (post-JS DOM serialized back to HTML).
    /// This is the HTML that CDP Runtime.evaluate operates against.
    pub effective_html: Option<String>,
    pub current_som: Option<Som>,
    pub current_structured_data: Option<StructuredData>,

    // CDP DOM node mapping: nodeId -> (backendNodeId, SOM element)
    pub node_map: HashMap<u64, NodeEntry>,
    pub document_node_id: u64,

    // Frame
    pub frame_id: String,
    pub loader_id: String,

    // Pipeline config
    pub pipeline_config: PipelineConfig,

    // Network interception
    pub interceptor: NetworkInterceptor,

    // All session IDs that map to this target (for multi-attach routing)
    pub session_ids: Vec<String>,
    // Pending target needing attachedToTarget (set by createTarget, consumed by setAutoAttach)
    pub pending_attach: Option<(String, String)>, // (target_id, session_id)
    // Whether auto-attach has been configured at browser level (prevents duplicate events)
    pub auto_attach_configured: bool,
}

#[derive(Clone)]
pub struct NodeEntry {
    pub node_id: u64,
    pub backend_node_id: u64,
    pub som_element_id: Option<String>,
    pub node_type: u16, // 1=Element, 3=Text, 9=Document
    pub node_name: String,
    pub node_value: String,
    pub children_ids: Vec<u64>,
}

const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";

impl CdpTarget {
    pub fn new() -> Result<Self, String> {
        Self::new_with_profiles(crate::auth::config::profiles())
    }

    pub fn new_with_profiles(auth_profiles: &[String]) -> Result<Self, String> {
        let target_num = TARGET_COUNTER.fetch_add(1, Ordering::Relaxed);
        let target_id = format!("{:032X}", target_num);
        let session_id = format!("{:032X}", target_num + 1000);
        let frame_id = format!("{:032X}", target_num + 2000);
        let loader_id = format!("{:032X}", target_num + 3000);

        let reqwest_jar = Arc::new(Jar::default());

        // Load auth cookies from profiles
        for domain in auth_profiles {
            if let Err(e) = crate::auth::store::load_into_jar(domain, &reqwest_jar) {
                tracing::warn!(domain = %domain, error = %e, "Failed to load auth profile");
            }
        }

        let client = fetch::build_client_h1_fallback(Some(DEFAULT_USER_AGENT), reqwest_jar.clone())
            .map_err(|e| e.to_string())?;

        // Create CDP cookie jar that syncs with the reqwest jar
        let cookie_jar = CookieJar::new(reqwest_jar.clone());

        Ok(CdpTarget {
            target_id,
            session_id: session_id.clone(),
            client,
            reqwest_jar,
            cookie_jar,
            timeout_ms: 30000,
            user_agent: DEFAULT_USER_AGENT.to_string(),
            extra_headers: HashMap::new(),
            session_ids: vec![session_id],
            pending_attach: None,
            auto_attach_configured: false,
            current_url: None,
            current_html: None,
            effective_html: None,
            current_som: None,
            current_structured_data: None,
            node_map: HashMap::new(),
            document_node_id: 0,
            frame_id,
            loader_id,
            pipeline_config: PipelineConfig {
                execute_js: true,
                fetch_external_scripts: true,
                ..Default::default()
            },
            interceptor: NetworkInterceptor::new(),
        })
    }

    /// Generate a fresh target ID and session ID for a new "page".
    /// The new session routes to this same CdpTarget (single-page for v0.1).
    /// Does NOT immediately attach - the next setAutoAttach fires the events.
    pub fn create_child_target(&mut self) -> String {
        let num = TARGET_COUNTER.fetch_add(1, Ordering::Relaxed);
        let new_target_id = format!("{:032X}", num);
        let new_session_id = format!("{:032X}", num + 1000);
        let new_frame_id = format!("{:032X}", num + 2000);

        // Update to use the new IDs (latest "page" wins)
        self.target_id = new_target_id.clone();
        self.session_id = new_session_id.clone();
        self.frame_id = new_frame_id.clone();
        self.session_ids.push(new_session_id.clone());
        self.pending_attach = Some((new_target_id.clone(), new_session_id));

        // Reset page state for the new "page"
        self.current_url = None;
        self.current_html = None;
        self.effective_html = None;
        self.current_som = None;
        self.current_structured_data = None;
        self.node_map.clear();

        new_target_id
    }

    /// Set page content directly (no network fetch), run through JS pipeline, update SOM.
    /// Used by Page.setContent (Playwright's primary way to set HTML).
    pub async fn set_content(&mut self, html: &str) -> Result<SetContentResult, String> {
        let url = self
            .current_url
            .as_deref()
            .unwrap_or("about:blank")
            .to_string();

        let page_result = crate::js::pipeline::process_page_async(
            html,
            &url,
            &self.pipeline_config,
            &self.client,
        )
        .await
        .map_err(|e| e.to_string())?;

        self.current_html = Some(html.to_string());
        self.effective_html = Some(page_result.effective_html);
        self.current_structured_data = page_result.som.structured_data.clone();
        self.current_som = Some(page_result.som);

        // Rebuild node map from SOM
        self.rebuild_node_map();

        // Update loader ID for this content set
        let nav_num = TARGET_COUNTER.fetch_add(1, Ordering::Relaxed);
        self.loader_id = format!("{:032X}", nav_num);

        Ok(SetContentResult {
            frame_id: self.frame_id.clone(),
            loader_id: self.loader_id.clone(),
        })
    }

    /// Navigate using our full pipeline, return events to emit.
    pub async fn navigate(&mut self, url: &str) -> Result<NavigateResult, String> {
        // Check interception rules before fetch
        let (action, _intercept_info) =
            self.interceptor
                .check_request(url, &InterceptResourceType::Document, true);

        let mut fetch_result = match action {
            InterceptAction::Fulfill(params) => {
                NetworkInterceptor::fulfill_request(&params, url)
            }
            InterceptAction::Fail(reason) => {
                return Err(NetworkInterceptor::fail_request(&reason, url).to_string());
            }
            InterceptAction::Continue(overrides) => {
                let actual_url = overrides
                    .as_ref()
                    .and_then(|o| o.url.as_ref())
                    .map(|u| u.as_str())
                    .unwrap_or(url);

                let extra_headers = overrides
                    .as_ref()
                    .and_then(|o| o.headers.clone())
                    .unwrap_or_default();

                if extra_headers.is_empty() {
                    fetch::fetch_url(&self.client, actual_url, self.timeout_ms)
                        .await
                        .map_err(|e| e.to_string())?
                } else {
                    fetch::fetch_url_with_headers(
                        &self.client,
                        actual_url,
                        self.timeout_ms,
                        &extra_headers,
                    )
                    .await
                    .map_err(|e| e.to_string())?
                }
            }
        };

        // Check response interception rules
        self.interceptor.check_response(
            url,
            &InterceptResourceType::Document,
            &mut fetch_result,
        );

        let final_url = fetch_result.url.clone();

        // Parse Set-Cookie headers and add to our CDP cookie jar
        for set_cookie in &fetch_result.set_cookies {
            self.cookie_jar.parse_set_cookie(set_cookie, &final_url);
        }

        let page_result = crate::js::pipeline::process_page_async(
            &fetch_result.html,
            &final_url,
            &self.pipeline_config,
            &self.client,
        )
        .await
        .map_err(|e| e.to_string())?;

        let status = fetch_result.status;
        let mime_type = fetch_result.content_type.clone();
        let encoded_data_length = fetch_result.html_bytes;
        let html = fetch_result.html;

        self.current_url = Some(final_url.clone());
        self.current_html = Some(html);
        self.effective_html = Some(page_result.effective_html);
        self.current_structured_data = page_result.som.structured_data.clone();
        self.current_som = Some(page_result.som);

        // Rebuild node map from SOM
        self.rebuild_node_map();

        // Update loader ID for this navigation
        let nav_num = TARGET_COUNTER.fetch_add(1, Ordering::Relaxed);
        self.loader_id = format!("{:032X}", nav_num);

        Ok(NavigateResult {
            url: final_url,
            loader_id: self.loader_id.clone(),
            frame_id: self.frame_id.clone(),
            status,
            mime_type,
            encoded_data_length,
        })
    }

    /// Build a CDP-compatible node tree from our SOM.
    fn rebuild_node_map(&mut self) {
        self.node_map.clear();

        let som = match &self.current_som {
            Some(s) => s,
            None => return,
        };

        // Create document node
        let doc_id = next_node_id();
        self.document_node_id = doc_id;

        let mut child_ids = Vec::new();

        // Create nodes for each region and its elements
        for region in &som.regions {
            let region_id = next_node_id();
            let mut region_children = Vec::new();

            for element in &region.elements {
                let el_id = next_node_id();

                // Create text child if element has text
                let mut el_children = Vec::new();
                if let Some(text) = &element.text {
                    if !text.is_empty() {
                        let text_id = next_node_id();
                        self.node_map.insert(
                            text_id,
                            NodeEntry {
                                node_id: text_id,
                                backend_node_id: text_id,
                                som_element_id: None,
                                node_type: 3, // Text
                                node_name: "#text".to_string(),
                                node_value: text.clone(),
                                children_ids: vec![],
                            },
                        );
                        el_children.push(text_id);
                    }
                }

                self.node_map.insert(
                    el_id,
                    NodeEntry {
                        node_id: el_id,
                        backend_node_id: el_id,
                        som_element_id: Some(element.id.clone()),
                        node_type: 1, // Element
                        node_name: role_to_tag(&element.role),
                        node_value: String::new(),
                        children_ids: el_children,
                    },
                );

                region_children.push(el_id);
            }

            self.node_map.insert(
                region_id,
                NodeEntry {
                    node_id: region_id,
                    backend_node_id: region_id,
                    som_element_id: None,
                    node_type: 1,
                    node_name: "section".to_string(),
                    node_value: String::new(),
                    children_ids: region_children,
                },
            );

            child_ids.push(region_id);
        }

        self.node_map.insert(
            doc_id,
            NodeEntry {
                node_id: doc_id,
                backend_node_id: doc_id,
                som_element_id: None,
                node_type: 9, // Document
                node_name: "#document".to_string(),
                node_value: String::new(),
                children_ids: child_ids,
            },
        );
    }

    /// Find an element by SOM element ID.
    pub fn find_element_by_som_id(&self, som_id: &str) -> Option<&Element> {
        let som = self.current_som.as_ref()?;
        for region in &som.regions {
            for element in &region.elements {
                if element.id == som_id {
                    return Some(element);
                }
            }
        }
        None
    }

    /// Find a node by CSS selector (simplified: tag, #id, .class).
    pub fn query_selector(&self, selector: &str) -> Option<u64> {
        // Ensure a page is loaded
        let _ = self.current_som.as_ref()?;

        for (node_id, entry) in &self.node_map {
            if entry.node_type != 1 {
                continue;
            }

            // Match by tag name
            if entry.node_name == selector {
                return Some(*node_id);
            }

            // Match by SOM element role mapping
            if let Some(ref som_id) = entry.som_element_id {
                if let Some(element) = self.find_element_by_som_id(som_id) {
                    // Match by role name (e.g., "link", "button")
                    let role_str = format!("{:?}", element.role).to_lowercase();
                    if selector == role_str {
                        return Some(*node_id);
                    }

                    // Match by text content
                    if let Some(ref text) = element.text {
                        if text.contains(selector) {
                            return Some(*node_id);
                        }
                    }
                }
            }
        }
        None
    }

    /// Query all matching selectors.
    pub fn query_selector_all(&self, selector: &str) -> Vec<u64> {
        let mut results = Vec::new();
        for (node_id, entry) in &self.node_map {
            if entry.node_type != 1 {
                continue;
            }
            if entry.node_name == selector {
                results.push(*node_id);
            }
        }
        results
    }

    /// Evaluate JavaScript expression against the post-navigation DOM.
    ///
    /// This creates a temporary JsRuntime, bootstraps the DOM from effective_html
    /// (without re-executing scripts), and evaluates the expression.
    ///
    /// Returns the result as a serde_json::Value, or an error string.
    pub fn evaluate_js(&self, expression: &str) -> Result<serde_json::Value, String> {
        let html = self
            .effective_html
            .as_ref()
            .ok_or_else(|| "No page loaded".to_string())?;
        let url = self.current_url.as_deref().unwrap_or("about:blank");

        // Create a temporary runtime for this evaluation
        // We use spawn_blocking because V8 is !Send and we're in an async context
        let html_clone = html.clone();
        let url_clone = url.to_string();
        let expression_clone = expression.to_string();

        // Create the runtime and evaluate synchronously
        // This is safe because we're creating a new isolate just for this evaluation
        let mut runtime = JsRuntime::new(RuntimeConfig {
            inject_dom_shim: true,
            execute_inline_scripts: false, // Don't re-execute scripts
            ..Default::default()
        });

        // Bootstrap DOM from effective HTML (scripts won't re-execute because
        // the DOM shim's bootstrap just parses HTML into the DOM tree)
        runtime.bootstrap_dom(&html_clone, &url_clone);

        // Wrap expression to properly serialize objects/arrays
        let wrapped_expr = format!(
            "(function() {{ var __r = ({}); return typeof __r === 'object' && __r !== null ? JSON.stringify(__r) : __r; }})()",
            expression_clone
        );

        // Evaluate the expression
        match runtime.eval(&wrapped_expr) {
            Ok(result) => {
                // Handle undefined
                if result == "undefined" || result.is_empty() {
                    return Ok(serde_json::Value::Null);
                }
                // Try to parse as JSON first (for objects/arrays)
                if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&result) {
                    Ok(json_val)
                } else {
                    // Return as string value
                    Ok(serde_json::Value::String(result))
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }

    /// Execute a function call against the post-navigation DOM.
    ///
    /// This is used by Runtime.callFunctionOn. The function is wrapped and called
    /// with the provided arguments.
    pub fn call_function_on(
        &self,
        function_declaration: &str,
        arguments: &[serde_json::Value],
    ) -> Result<serde_json::Value, String> {
        let html = self
            .effective_html
            .as_ref()
            .ok_or_else(|| "No page loaded".to_string())?;
        let url = self.current_url.as_deref().unwrap_or("about:blank");

        let html_clone = html.clone();
        let url_clone = url.to_string();

        let mut runtime = JsRuntime::new(RuntimeConfig {
            inject_dom_shim: true,
            execute_inline_scripts: false,
            ..Default::default()
        });

        runtime.bootstrap_dom(&html_clone, &url_clone);

        // Build the call expression: (function).call(null, arg1, arg2, ...)
        // We wrap the result in JSON.stringify to properly serialize objects/arrays
        // Serialize arguments to JSON
        let args_json: Vec<String> = arguments.iter().map(|a| a.to_string()).collect();
        let args_str = args_json.join(", ");

        let call_expr = if args_str.is_empty() {
            format!(
                "(function() {{ var __r = ({})(); return typeof __r === 'object' && __r !== null ? JSON.stringify(__r) : __r; }})()",
                function_declaration
            )
        } else {
            format!(
                "(function() {{ var __r = ({}).call(null, {}); return typeof __r === 'object' && __r !== null ? JSON.stringify(__r) : __r; }})()",
                function_declaration, args_str
            )
        };

        match runtime.eval(&call_expr) {
            Ok(result) => {
                // Handle undefined
                if result == "undefined" || result.is_empty() {
                    return Ok(serde_json::Value::Null);
                }
                // Try to parse as JSON first (for objects/arrays)
                if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&result) {
                    Ok(json_val)
                } else {
                    // Return as string value
                    Ok(serde_json::Value::String(result))
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }
}

pub struct NavigateResult {
    pub url: String,
    pub loader_id: String,
    pub frame_id: String,
    pub status: u16,
    pub mime_type: String,
    pub encoded_data_length: usize,
}

pub struct SetContentResult {
    pub frame_id: String,
    pub loader_id: String,
}

fn next_node_id() -> u64 {
    NODE_COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn role_to_tag(role: &ElementRole) -> String {
    match role {
        ElementRole::Link => "a".to_string(),
        ElementRole::Button => "button".to_string(),
        ElementRole::TextInput => "input".to_string(),
        ElementRole::Textarea => "textarea".to_string(),
        ElementRole::Select => "select".to_string(),
        ElementRole::Checkbox => "input".to_string(),
        ElementRole::Radio => "input".to_string(),
        ElementRole::Image => "img".to_string(),
        ElementRole::Heading => "h2".to_string(),
        ElementRole::Paragraph => "p".to_string(),
        ElementRole::List => "ul".to_string(),
        ElementRole::Table => "table".to_string(),
        ElementRole::Section => "section".to_string(),
        ElementRole::Separator => "hr".to_string(),
    }
}
