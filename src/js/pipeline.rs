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

        if !exec_scripts.is_empty() {
            let mut runtime = JsRuntime::new(config.js_config.clone());
            runtime.set_page_url(url);

            let report = runtime.execute_page_scripts(&exec_scripts);

            if config.timer_drain_ms > 0 {
                runtime.drain_timers(config.timer_drain_ms);
            }

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
        }
    }

    let t2 = Instant::now();
    let som = compiler::compile(html, url).map_err(|e| PipelineError::SomCompile(e.to_string()))?;
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
/// If JS execution is enabled, inline scripts are extracted and run in V8
/// before the SOM is compiled. The DOM shim captures mutations that could
/// affect the semantic structure.
pub fn process_page(
    html: &str,
    url: &str,
    config: &PipelineConfig,
) -> Result<PageResult, PipelineError> {
    let pipeline_start = Instant::now();

    let mut js_report = None;
    let mut extract_us = 0u128;
    let mut js_us = 0u128;

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

        // Phase 2: Execute JS
        if !inline_scripts.is_empty() {
            let t1 = Instant::now();
            let mut runtime = JsRuntime::new(config.js_config.clone());
            runtime.set_page_url(url);

            let report = runtime.execute_page_scripts(&inline_scripts);

            // Drain short timers (many pages use setTimeout(fn, 0) for initialization)
            if config.timer_drain_ms > 0 {
                runtime.drain_timers(config.timer_drain_ms);
            }

            js_us = t1.elapsed().as_micros();

            debug!(
                scripts_total = report.total,
                scripts_ok = report.succeeded,
                scripts_err = report.failed,
                js_ms = js_us / 1000,
                "JS execution complete"
            );

            js_report = Some(report);
        }
    }

    // Phase 3: Apply JS mutations to HTML, then compile SOM.
    // If JS created document.write() calls or DOM insertions, we inject them
    // back into the HTML before SOM compilation for maximum accuracy.
    let effective_html = if config.execute_js && js_report.is_some() {
        let t_mut = Instant::now();
        let mut runtime = JsRuntime::new(config.js_config.clone());
        runtime.set_page_url(url);

        // Re-execute scripts in a fresh context to get mutations
        let scripts = extract::extract_scripts(html);
        let inline_scripts: Vec<(String, String)> = scripts
            .iter()
            .filter(|s| s.is_inline)
            .map(|s| (s.source.clone(), s.label.clone()))
            .collect();

        if !inline_scripts.is_empty() {
            runtime.execute_page_scripts(&inline_scripts);
            if config.timer_drain_ms > 0 {
                runtime.drain_timers(config.timer_drain_ms);
            }
        }

        // Collect document.write and appendChild mutations
        let mutations_json = runtime.execute_in_context(
            "JSON.stringify(__plasmate_mutations.filter(function(m){ return m.type === 'document.write' || m.type === 'appendChild'; }))",
            "<collect-mutations>",
        ).unwrap_or_else(|_| "[]".to_string());

        if let Ok(mutations) = serde_json::from_str::<Vec<serde_json::Value>>(&mutations_json) {
            if !mutations.is_empty() {
                // Inject mutation content before </body>
                let mut patched = html.to_string();
                let mut injection = String::new();
                for m in &mutations {
                    if let Some(html_content) = m.get("html").and_then(|v| v.as_str()) {
                        injection.push_str(html_content);
                    }
                    if let Some(text) = m.get("text").and_then(|v| v.as_str()) {
                        if let Some(tag) = m.get("tag").and_then(|v| v.as_str()) {
                            injection.push_str(&format!(
                                "<{0}>{1}</{0}>",
                                tag.to_lowercase(),
                                text
                            ));
                        }
                    }
                }
                if !injection.is_empty() {
                    if let Some(pos) = patched.to_lowercase().rfind("</body>") {
                        patched.insert_str(pos, &injection);
                    } else {
                        patched.push_str(&injection);
                    }
                    debug!(
                        mutations = mutations.len(),
                        injected_bytes = injection.len(),
                        elapsed_us = t_mut.elapsed().as_micros(),
                        "JS mutations applied to HTML"
                    );
                    std::borrow::Cow::Owned(patched)
                } else {
                    std::borrow::Cow::Borrowed(html)
                }
            } else {
                std::borrow::Cow::Borrowed(html)
            }
        } else {
            std::borrow::Cow::Borrowed(html)
        }
    } else {
        std::borrow::Cow::Borrowed(html)
    };

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
}
