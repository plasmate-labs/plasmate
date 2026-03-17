//! Page processing pipeline: fetch HTML -> extract scripts -> execute JS -> compile SOM.
//!
//! This is the core of Plasmate's page understanding. Unlike Lightpanda and Chrome
//! which build a full DOM then convert it afterward, we go:
//!   HTML -> (optional JS execution) -> SOM
//! in a single pass with no intermediate rendering.

use std::time::Instant;
use tracing::debug;

use super::extract;
use super::runtime::{JsExecutionReport, JsRuntime, RuntimeConfig};
use super::script_fetch;
use crate::som::compiler;
use crate::som::types::Som;

/// Result of the full page processing pipeline.
#[derive(Debug)]
pub struct PageResult {
    /// The compiled SOM.
    pub som: Som,
    /// The page URL (after redirects).
    pub url: String,
    /// Pipeline timing breakdown.
    pub timing: PipelineTiming,
    /// JS execution report (if JS was enabled).
    pub js_report: Option<JsExecutionReport>,
}

/// Timing breakdown for the pipeline stages.
#[derive(Debug, Clone)]
pub struct PipelineTiming {
    /// Time to extract scripts from HTML.
    pub extract_scripts_us: u128,
    /// Time spent executing JS.
    pub js_execution_us: u128,
    /// Time to compile HTML to SOM.
    pub som_compile_us: u128,
    /// Total pipeline time (excluding network fetch).
    pub total_us: u128,
}

/// Configuration for the pipeline.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Whether to execute JavaScript before compiling SOM.
    pub execute_js: bool,
    /// Whether to fetch external <script src="..."> files.
    pub fetch_external_scripts: bool,
    /// JS runtime configuration.
    pub js_config: RuntimeConfig,
    /// Max timer drain threshold in ms (execute short setTimeout callbacks).
    pub timer_drain_ms: u64,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            execute_js: true,
            fetch_external_scripts: false, // Off by default for sync API; async API enables it
            js_config: RuntimeConfig::default(),
            timer_drain_ms: 100,
        }
    }
}

/// Process a page through the full pipeline (async version with external script fetching).
pub async fn process_page_async(
    html: &str,
    url: &str,
    config: &PipelineConfig,
    client: &reqwest::Client,
) -> Result<PageResult, PipelineError> {
    let pipeline_start = Instant::now();

    let mut js_report = None;
    let mut extract_us = 0u128;
    let mut js_us = 0u128;
    let mut effective_html = std::borrow::Cow::Borrowed(html);

    if config.execute_js {
        let t0 = Instant::now();
        let scripts = extract::extract_scripts(html);
        extract_us = t0.elapsed().as_micros();

        // Resolve external scripts (fetch from network)
        let t1 = Instant::now();
        let resolved = if config.fetch_external_scripts {
            script_fetch::resolve_scripts(&scripts, url, client).await
        } else {
            scripts
                .iter()
                .filter(|s| s.is_inline)
                .map(|s| script_fetch::ResolvedScript {
                    source: s.source.clone(),
                    label: s.label.clone(),
                    index: s.index,
                })
                .collect()
        };

        let exec_scripts: Vec<(String, String)> = resolved
            .iter()
            .filter(|s| !s.source.is_empty())
            .map(|s| (s.source.clone(), s.label.clone()))
            .collect();

        // Always create runtime to bootstrap DOM, even if no scripts
        let mut runtime = JsRuntime::new(config.js_config.clone());

        // Inject the fetch bridge for real HTTP requests from JS
        runtime.inject_fetch_bridge(client.clone());

        // Bootstrap the DOM tree from source HTML
        runtime.bootstrap_dom(html, url);

        if !exec_scripts.is_empty() {
            // Execute page scripts
            let report = runtime.execute_page_scripts(&exec_scripts);

            // Pump microtasks after script execution (resolves Promise.then chains)
            runtime.pump_microtasks();

            // Fire DOMContentLoaded after scripts execute
            runtime.fire_dom_content_loaded();
            runtime.pump_microtasks();

            // Drain short timers
            if config.timer_drain_ms > 0 {
                runtime.drain_timers(config.timer_drain_ms);
                runtime.pump_microtasks();
            }

            // Fire load event
            runtime.fire_load();
            runtime.pump_microtasks();

            js_us = t1.elapsed().as_micros();

            debug!(
                scripts_total = report.total,
                scripts_ok = report.succeeded,
                scripts_err = report.failed,
                external_fetched = resolved
                    .iter()
                    .filter(|s| !s.label.starts_with("inline"))
                    .count(),
                js_ms = js_us / 1000,
                "JS execution complete (async)"
            );

            js_report = Some(report);

            // Serialize the DOM tree back to HTML (also pumps microtasks internally)
            if let Ok(serialized) = runtime.serialize_dom() {
                if !serialized.is_empty() && serialized != "undefined" {
                    effective_html = std::borrow::Cow::Owned(serialized);
                }
            }
        }
    }

    let t2 = Instant::now();
    let som = compiler::compile(&effective_html, url)
        .map_err(|e| PipelineError::SomCompile(e.to_string()))?;
    let som_us = t2.elapsed().as_micros();

    let total_us = pipeline_start.elapsed().as_micros();

    Ok(PageResult {
        som,
        url: url.to_string(),
        timing: PipelineTiming {
            extract_scripts_us: extract_us,
            js_execution_us: js_us,
            som_compile_us: som_us,
            total_us,
        },
        js_report,
    })
}

/// Process a page through the full pipeline.
///
/// This is the main entry point for converting fetched HTML into a SOM.
/// If JS execution is enabled:
/// 1. The source HTML is parsed into a JS DOM tree
/// 2. Inline scripts are extracted and executed in V8
/// 3. DOMContentLoaded and load events are fired
/// 4. The resulting DOM tree is serialized back to HTML
/// 5. The serialized HTML (with JS modifications) is compiled to SOM
pub fn process_page(
    html: &str,
    url: &str,
    config: &PipelineConfig,
) -> Result<PageResult, PipelineError> {
    process_page_with_client(html, url, config, None)
}

/// Process a page through the full pipeline with an optional HTTP client.
///
/// When a client is provided, JS fetch() and XMLHttpRequest will make real
/// HTTP requests. Without a client, they return stub responses.
pub fn process_page_with_client(
    html: &str,
    url: &str,
    config: &PipelineConfig,
    client: Option<&reqwest::Client>,
) -> Result<PageResult, PipelineError> {
    let pipeline_start = Instant::now();

    let mut js_report = None;
    let mut extract_us = 0u128;
    let mut js_us = 0u128;
    let mut effective_html = std::borrow::Cow::Borrowed(html);

    // Phase 1: Extract scripts (if JS enabled)
    if config.execute_js {
        let t0 = Instant::now();
        let scripts = extract::extract_scripts(html);
        extract_us = t0.elapsed().as_micros();

        let inline_scripts: Vec<(String, String)> = scripts
            .iter()
            .filter(|s| s.is_inline)
            .map(|s| (s.source.clone(), s.label.clone()))
            .collect();

        // Phase 2: Bootstrap DOM and execute JS
        let t1 = Instant::now();
        let mut runtime = JsRuntime::new(config.js_config.clone());

        // Inject the fetch bridge if a client is provided
        if let Some(c) = client {
            runtime.inject_fetch_bridge(c.clone());
        }

        // Bootstrap the DOM tree from source HTML
        runtime.bootstrap_dom(html, url);

        if !inline_scripts.is_empty() {
            // Execute page scripts in the context with the bootstrapped DOM
            let report = runtime.execute_page_scripts(&inline_scripts);

            // Pump microtasks after script execution (resolves Promise.then chains)
            runtime.pump_microtasks();

            // Fire DOMContentLoaded after scripts execute
            runtime.fire_dom_content_loaded();
            runtime.pump_microtasks();

            // Drain short timers (many pages use setTimeout(fn, 0) for initialization)
            if config.timer_drain_ms > 0 {
                runtime.drain_timers(config.timer_drain_ms);
                runtime.pump_microtasks();
            }

            // Fire load event
            runtime.fire_load();
            runtime.pump_microtasks();

            js_us = t1.elapsed().as_micros();

            debug!(
                scripts_total = report.total,
                scripts_ok = report.succeeded,
                scripts_err = report.failed,
                js_ms = js_us / 1000,
                "JS execution complete"
            );

            js_report = Some(report);

            // Serialize the DOM tree back to HTML
            // This captures all JS modifications: createElement, appendChild, innerHTML, etc.
            if let Ok(serialized) = runtime.serialize_dom() {
                if !serialized.is_empty() && serialized != "undefined" {
                    effective_html = std::borrow::Cow::Owned(serialized);
                }
            }
        }
    }

    let t2 = Instant::now();
    let som = compiler::compile(&effective_html, url)
        .map_err(|e| PipelineError::SomCompile(e.to_string()))?;
    let som_us = t2.elapsed().as_micros();

    let total_us = pipeline_start.elapsed().as_micros();

    Ok(PageResult {
        som,
        url: url.to_string(),
        timing: PipelineTiming {
            extract_scripts_us: extract_us,
            js_execution_us: js_us,
            som_compile_us: som_us,
            total_us,
        },
        js_report,
    })
}

/// Errors from the page processing pipeline.
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("SOM compilation failed: {0}")]
    SomCompile(String),
    #[error("JS execution failed: {0}")]
    JsExecution(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_no_js() {
        let config = PipelineConfig {
            execute_js: false,
            ..Default::default()
        };
        let html = r#"<html><body><h1>Hello</h1><p>World</p></body></html>"#;
        let result = process_page(html, "https://example.com", &config).unwrap();
        assert_eq!(result.som.title, "");
        assert!(result.js_report.is_none());
        assert!(result.timing.js_execution_us == 0);
    }

    #[test]
    fn test_pipeline_with_js() {
        let config = PipelineConfig::default();
        let html = r#"<html><head>
            <title>Test</title>
            <script>document.title = 'JS Title';</script>
        </head><body><h1>Hello</h1></body></html>"#;
        let result = process_page(html, "https://example.com", &config).unwrap();
        assert!(result.js_report.is_some());
        let report = result.js_report.unwrap();
        assert_eq!(report.succeeded, 1);
    }

    #[test]
    fn test_pipeline_timing() {
        let config = PipelineConfig::default();
        let html = r#"<html><body><script>var x = 1;</script><p>Hello</p></body></html>"#;
        let result = process_page(html, "https://example.com", &config).unwrap();
        assert!(result.timing.total_us > 0);
        assert!(result.timing.som_compile_us > 0);
    }

    #[test]
    fn test_pipeline_js_error_nonfatal() {
        let config = PipelineConfig::default();
        let html = r#"<html><body>
            <script>undefinedFunction();</script>
            <script>var ok = true;</script>
            <h1>Content</h1>
        </body></html>"#;
        let result = process_page(html, "https://example.com", &config).unwrap();
        let report = result.js_report.unwrap();
        assert_eq!(report.failed, 1);
        assert_eq!(report.succeeded, 1);
        // SOM should still compile
        assert!(result.som.meta.element_count > 0);
    }

    #[test]
    fn test_pipeline_real_world_js_patterns() {
        let config = PipelineConfig::default();
        // Simulate common JS patterns found on real sites
        let html = r#"<html><head>
            <script>
                // Analytics
                window.dataLayer = window.dataLayer || [];
                function gtag(){ dataLayer.push(arguments); }
                gtag('js', new Date());
                gtag('config', 'GA-123');
            </script>
            <script>
                // DOM ready pattern
                document.addEventListener('DOMContentLoaded', function() {
                    document.title = 'Loaded';
                });
            </script>
            <script>
                // Feature detection
                var hasTouch = 'ontouchstart' in window;
                var hasFetch = typeof fetch === 'function';
            </script>
        </head><body>
            <nav><a href="/">Home</a></nav>
            <main><h1>Page</h1><p>Content</p></main>
        </body></html>"#;
        let result = process_page(html, "https://example.com", &config).unwrap();
        let report = result.js_report.unwrap();
        assert_eq!(
            report.succeeded, 3,
            "All 3 common JS patterns should execute: {:?}",
            report.errors
        );
        assert_eq!(report.failed, 0);
    }

    // =========================================================================
    // JS Rendering Pipeline Tests
    // =========================================================================

    #[test]
    fn test_pipeline_js_creates_elements() {
        // Test that JS-created elements appear in the SOM
        let config = PipelineConfig::default();
        let html = r#"<html><body>
            <div id="root"></div>
            <script>
                var root = document.getElementById('root');
                var p = document.createElement('p');
                p.id = 'dynamic-para';
                p.textContent = 'Created by JavaScript';
                root.appendChild(p);
            </script>
        </body></html>"#;
        let result = process_page(html, "https://example.com", &config).unwrap();

        // Serialize SOM to JSON and check for dynamic content
        let som_json = serde_json::to_string(&result.som).unwrap();
        assert!(
            som_json.contains("Created by JavaScript"),
            "SOM should contain JS-created content"
        );
    }

    #[test]
    fn test_pipeline_innerhtml_rendering() {
        // Test that innerHTML modifications appear in SOM
        let config = PipelineConfig::default();
        let html = r#"<html><body>
            <div id="app"></div>
            <script>
                document.getElementById('app').innerHTML = '<h1>Rendered Title</h1><p>Rendered paragraph</p>';
            </script>
        </body></html>"#;
        let result = process_page(html, "https://example.com", &config).unwrap();

        let som_json = serde_json::to_string(&result.som).unwrap();
        assert!(
            som_json.contains("Rendered Title"),
            "SOM should contain innerHTML content"
        );
        assert!(
            som_json.contains("Rendered paragraph"),
            "SOM should contain innerHTML paragraph"
        );
    }

    #[test]
    fn test_pipeline_getelementbyid_modify() {
        // Test that modifying existing elements via getElementById works
        let config = PipelineConfig::default();
        let html = r#"<html><body>
            <p id="target">Original text</p>
            <script>
                document.getElementById('target').textContent = 'Modified by JS';
            </script>
        </body></html>"#;
        let result = process_page(html, "https://example.com", &config).unwrap();

        let som_json = serde_json::to_string(&result.som).unwrap();
        assert!(
            som_json.contains("Modified by JS"),
            "SOM should reflect JS text modification"
        );
        assert!(
            !som_json.contains("Original text"),
            "Original text should be replaced"
        );
    }

    #[test]
    fn test_pipeline_timer_based_rendering() {
        // Test that setTimeout(fn, 0) callbacks are executed
        let config = PipelineConfig::default();
        let html = r#"<html><body>
            <div id="container"></div>
            <script>
                setTimeout(function() {
                    var el = document.createElement('span');
                    el.textContent = 'Timer rendered';
                    document.getElementById('container').appendChild(el);
                }, 0);
            </script>
        </body></html>"#;
        let result = process_page(html, "https://example.com", &config).unwrap();

        let som_json = serde_json::to_string(&result.som).unwrap();
        assert!(
            som_json.contains("Timer rendered"),
            "Timer callback should have executed"
        );
    }

    #[test]
    fn test_pipeline_domcontentloaded_handler() {
        // Test that DOMContentLoaded handlers execute
        let config = PipelineConfig::default();
        let html = r#"<html><body>
            <div id="target"></div>
            <script>
                document.addEventListener('DOMContentLoaded', function() {
                    document.getElementById('target').innerHTML = '<strong>Loaded!</strong>';
                });
            </script>
        </body></html>"#;
        let result = process_page(html, "https://example.com", &config).unwrap();

        let som_json = serde_json::to_string(&result.som).unwrap();
        assert!(
            som_json.contains("Loaded!"),
            "DOMContentLoaded handler should have executed"
        );
    }

    #[test]
    fn test_pipeline_react_style_app() {
        // Simulate a React-style single page app with JS rendering
        let config = PipelineConfig::default();
        let html = r#"<html><body>
            <div id="root"></div>
            <script>
                function App() {
                    var div = document.createElement('div');
                    div.className = 'app-container';

                    var header = document.createElement('header');
                    var h1 = document.createElement('h1');
                    h1.textContent = 'My React App';
                    header.appendChild(h1);
                    div.appendChild(header);

                    var main = document.createElement('main');
                    var p = document.createElement('p');
                    p.textContent = 'Welcome to the application';
                    main.appendChild(p);

                    var btn = document.createElement('button');
                    btn.textContent = 'Get Started';
                    btn.id = 'cta-button';
                    main.appendChild(btn);
                    div.appendChild(main);

                    return div;
                }

                document.getElementById('root').appendChild(App());
            </script>
        </body></html>"#;
        let result = process_page(html, "https://example.com", &config).unwrap();

        // Check that all React-rendered content is in the SOM
        let som_json = serde_json::to_string(&result.som).unwrap();
        assert!(
            som_json.contains("My React App"),
            "Heading should be present"
        );
        assert!(
            som_json.contains("Welcome to the application"),
            "Content should be present"
        );
        assert!(som_json.contains("Get Started"), "Button should be present");

        // Check interactive element count (button should be counted)
        assert!(
            result.som.meta.interactive_count > 0,
            "Should have interactive elements"
        );
    }

    #[test]
    fn test_pipeline_document_write() {
        // Test that document.write works
        let config = PipelineConfig::default();
        let html = r#"<html><body>
            <script>
                document.write('<p id="written">Written content</p>');
            </script>
        </body></html>"#;
        let result = process_page(html, "https://example.com", &config).unwrap();

        let som_json = serde_json::to_string(&result.som).unwrap();
        assert!(
            som_json.contains("Written content"),
            "document.write content should appear"
        );
    }

    #[test]
    fn test_pipeline_no_regression_static_html() {
        // Ensure static HTML without JS still works correctly
        let config = PipelineConfig::default();
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>Static Page</title></head>
<body>
<nav>
    <a href="/">Home</a>
    <a href="/about">About</a>
</nav>
<main>
    <h1>Welcome</h1>
    <p>This is static content with no JavaScript.</p>
    <form action="/search" method="GET">
        <input type="text" placeholder="Search...">
        <button type="submit">Search</button>
    </form>
</main>
<footer>
    <p>Copyright 2026</p>
</footer>
</body>
</html>"#;
        let result = process_page(html, "https://example.com", &config).unwrap();

        // SOM should have proper structure
        assert_eq!(result.som.title, "Static Page");
        assert!(!result.som.regions.is_empty());

        // Should have navigation, main, form regions
        let region_roles: Vec<_> = result.som.regions.iter().map(|r| &r.role).collect();
        assert!(
            result.som.regions.iter().any(|r| !r.elements.is_empty()),
            "Regions should have elements"
        );
    }

    #[test]
    fn test_pipeline_queryselector_in_script() {
        // Test that querySelector works in scripts
        let config = PipelineConfig::default();
        let html = r#"<html><body>
            <div class="container">
                <p class="message">Initial</p>
            </div>
            <script>
                var msg = document.querySelector('.container .message');
                if (msg) {
                    msg.textContent = 'Found and modified';
                }
            </script>
        </body></html>"#;
        let result = process_page(html, "https://example.com", &config).unwrap();

        let som_json = serde_json::to_string(&result.som).unwrap();
        assert!(
            som_json.contains("Found and modified"),
            "querySelector should find element"
        );
    }

    #[test]
    fn test_pipeline_multiple_scripts_state_persistence() {
        // Test that state persists across multiple script blocks
        let config = PipelineConfig::default();
        let html = r#"<html><body>
            <div id="output"></div>
            <script>
                var items = [];
                items.push('first');
            </script>
            <script>
                items.push('second');
            </script>
            <script>
                items.push('third');
                document.getElementById('output').textContent = items.join(', ');
            </script>
        </body></html>"#;
        let result = process_page(html, "https://example.com", &config).unwrap();

        let som_json = serde_json::to_string(&result.som).unwrap();
        assert!(
            som_json.contains("first, second, third"),
            "State should persist across script blocks"
        );
    }

    #[test]
    fn test_pipeline_link_creation() {
        // Test that dynamically created links appear in SOM with correct attributes
        let config = PipelineConfig::default();
        let html = r#"<html><body>
            <nav id="nav"></nav>
            <script>
                var nav = document.getElementById('nav');
                var links = [
                    {href: '/home', text: 'Home'},
                    {href: '/products', text: 'Products'},
                    {href: '/contact', text: 'Contact'}
                ];
                links.forEach(function(link) {
                    var a = document.createElement('a');
                    a.href = link.href;
                    a.textContent = link.text;
                    nav.appendChild(a);
                });
            </script>
        </body></html>"#;
        let result = process_page(html, "https://example.com", &config).unwrap();

        let som_json = serde_json::to_string(&result.som).unwrap();
        assert!(som_json.contains("/home"), "Home link should be present");
        assert!(
            som_json.contains("/products"),
            "Products link should be present"
        );
        assert!(
            som_json.contains("Contact"),
            "Contact text should be present"
        );
        assert!(
            result.som.meta.interactive_count >= 3,
            "Should have at least 3 interactive links"
        );
    }

    #[test]
    fn test_process_page_with_client_no_client() {
        // Test process_page_with_client with None client (fallback to stub)
        let config = PipelineConfig::default();
        let html = r#"<html><body>
            <div id="result"></div>
            <script>
                // fetch without bridge should return stub response
                fetch('/api/data').then(function(r) {
                    document.getElementById('result').textContent = r.ok ? 'OK' : 'FAIL';
                });
            </script>
        </body></html>"#;
        let result = process_page_with_client(html, "https://example.com", &config, None).unwrap();

        // The stub fetch returns ok:true, so result should be "OK"
        let som_json = serde_json::to_string(&result.som).unwrap();
        assert!(som_json.contains("OK"), "Stub fetch should return ok");
    }
}
