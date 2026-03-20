use std::sync::Arc;
use std::time::Instant;

use reqwest::cookie::Jar;
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

use crate::js::pipeline::{self, PipelineConfig};
use crate::network::fetch;
use crate::som::compiler;

#[derive(Debug, Clone)]
pub struct CoverageOptions {
    pub timeout_ms: u64,
    pub concurrency: usize,

    pub execute_js: bool,
    pub fetch_external_scripts: bool,

    /// V8 heap limit for the page runtime. 0 means unlimited.
    pub js_max_heap_bytes: usize,

    /// External script fetching limits (only used when fetch_external_scripts is true).
    pub max_external_scripts: usize,
    pub max_external_script_bytes: usize,
    pub max_external_total_bytes: usize,
    pub external_script_timeout_ms: u64,

    pub timer_drain_ms: u64,
    pub max_urls: Option<usize>,
}

impl Default for CoverageOptions {
    fn default() -> Self {
        Self {
            timeout_ms: 15000,
            concurrency: 8,

            execute_js: true,
            fetch_external_scripts: true,

            js_max_heap_bytes: 256 * 1024 * 1024,

            max_external_scripts: 20,
            max_external_script_bytes: 512 * 1024,
            max_external_total_bytes: 4 * 1024 * 1024,
            external_script_timeout_ms: 5000,

            timer_drain_ms: 100,
            max_urls: Some(100),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageStatus {
    Ok,
    Thin,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureKind {
    Timeout,
    HttpError,
    NavigationFailed,
    NonHtml,
    PipelineError,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageResult {
    pub input_url: String,
    pub final_url: Option<String>,
    pub status: CoverageStatus,

    pub http_status: Option<u16>,
    pub content_type: Option<String>,
    pub title: Option<String>,

    pub html_bytes: Option<usize>,
    pub som_bytes: Option<usize>,
    pub element_count: Option<usize>,
    pub interactive_count: Option<usize>,

    pub fetch_ms: Option<u64>,
    pub pipeline_ms: Option<u64>,

    pub js_total_scripts: Option<usize>,
    pub js_succeeded: Option<usize>,
    pub js_failed: Option<usize>,
    pub js_errors: Option<Vec<String>>,

    pub failure_kind: Option<FailureKind>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageBreakdownItem {
    pub key: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageSummary {
    pub urls_total: usize,
    pub ok: usize,
    pub thin: usize,
    pub failed: usize,
    pub ok_percent: f64,
    pub breakdown: Vec<CoverageBreakdownItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub generated_at_utc: String,
    pub plasmate_version: String,
    pub options: CoverageReportOptions,
    pub summary: CoverageSummary,
    pub results: Vec<CoverageResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReportOptions {
    pub timeout_ms: u64,
    pub concurrency: usize,

    pub execute_js: bool,
    pub fetch_external_scripts: bool,

    pub js_max_heap_bytes: usize,

    pub max_external_scripts: usize,
    pub max_external_script_bytes: usize,
    pub max_external_total_bytes: usize,
    pub external_script_timeout_ms: u64,

    pub timer_drain_ms: u64,
    pub max_urls: Option<usize>,
}

fn now_utc_rfc3339ish() -> String {
    // Avoid chrono dependency. Good enough for UI + logs.
    let now = std::time::SystemTime::now();
    let secs = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    // This is not a true RFC3339 conversion, but stable and sortable.
    format!("unix:{}", secs)
}

fn classify_fetch_error(err: &fetch::FetchError) -> (FailureKind, String) {
    match err {
        fetch::FetchError::Timeout(ms) => (FailureKind::Timeout, format!("Timeout after {ms}ms")),
        fetch::FetchError::HttpError { status, url } => (
            FailureKind::HttpError,
            format!("HTTP error {status} for {url}"),
        ),
        fetch::FetchError::NavigationFailed(msg) => (FailureKind::NavigationFailed, msg.clone()),
    }
}

fn is_thin(som_bytes: usize, element_count: usize, interactive_count: usize) -> bool {
    // Conservative thresholds, goal is to flag pages that are technically "ok" but likely useless.
    // We can tune after the first few runs.
    if som_bytes < 300 {
        return true;
    }
    if element_count < 15 {
        return true;
    }
    if interactive_count == 0 && element_count < 40 {
        return true;
    }
    false
}

pub fn parse_urls_file(content: &str) -> Vec<String> {
    content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect()
}

pub async fn run(urls: &[String], opts: &CoverageOptions) -> CoverageReport {
    let jar = Arc::new(Jar::default());
    let client = fetch::build_client(None, jar).expect("Failed to build HTTP client");

    let max = opts.max_urls.unwrap_or(urls.len());
    let urls: Vec<String> = urls.iter().take(max).cloned().collect();

    info!(count = urls.len(), "Running coverage suite");

    let sem = Arc::new(Semaphore::new(opts.concurrency.max(1)));
    let mut handles = Vec::new();

    for input_url in urls {
        let client = client.clone();
        let sem = sem.clone();
        let opts = opts.clone();

        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.expect("semaphore poisoned");

            let timeout = std::time::Duration::from_millis(opts.timeout_ms);
            match tokio::time::timeout(timeout, cover_single(&client, &input_url, &opts)).await {
                Ok(r) => r,
                Err(_) => CoverageResult {
                    input_url,
                    final_url: None,
                    status: CoverageStatus::Failed,
                    http_status: None,
                    content_type: None,
                    title: None,
                    html_bytes: None,
                    som_bytes: None,
                    element_count: None,
                    interactive_count: None,
                    fetch_ms: None,
                    pipeline_ms: None,
                    js_total_scripts: None,
                    js_succeeded: None,
                    js_failed: None,
                    js_errors: None,
                    failure_kind: Some(FailureKind::Timeout),
                    error: Some(format!("Overall timeout after {}ms", opts.timeout_ms)),
                },
            }
        }));
    }

    let mut results = Vec::new();
    for h in handles {
        match h.await {
            Ok(r) => results.push(r),
            Err(e) => {
                warn!(error = %e, "Coverage task join error");
            }
        }
    }

    // Stable-ish ordering for diff readability.
    results.sort_by(|a, b| a.input_url.cmp(&b.input_url));

    let mut ok = 0usize;
    let mut thin = 0usize;
    let mut failed = 0usize;

    let mut breakdown: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();

    for r in &results {
        match r.status {
            CoverageStatus::Ok => ok += 1,
            CoverageStatus::Thin => thin += 1,
            CoverageStatus::Failed => failed += 1,
        }

        let key = match (&r.status, &r.failure_kind) {
            (CoverageStatus::Ok, _) => "ok".to_string(),
            (CoverageStatus::Thin, _) => "thin".to_string(),
            (CoverageStatus::Failed, Some(k)) => format!("failed:{k:?}").to_lowercase(),
            (CoverageStatus::Failed, None) => "failed:unknown".to_string(),
        };
        *breakdown.entry(key).or_insert(0) += 1;
    }

    let total = results.len();
    let ok_percent = if total == 0 {
        0.0
    } else {
        (ok as f64 / total as f64) * 100.0
    };

    let breakdown = breakdown
        .into_iter()
        .map(|(key, count)| CoverageBreakdownItem { key, count })
        .collect();

    CoverageReport {
        generated_at_utc: now_utc_rfc3339ish(),
        plasmate_version: env!("CARGO_PKG_VERSION").to_string(),
        options: CoverageReportOptions {
            timeout_ms: opts.timeout_ms,
            concurrency: opts.concurrency,

            execute_js: opts.execute_js,
            fetch_external_scripts: opts.fetch_external_scripts,

            js_max_heap_bytes: opts.js_max_heap_bytes,

            max_external_scripts: opts.max_external_scripts,
            max_external_script_bytes: opts.max_external_script_bytes,
            max_external_total_bytes: opts.max_external_total_bytes,
            external_script_timeout_ms: opts.external_script_timeout_ms,

            timer_drain_ms: opts.timer_drain_ms,
            max_urls: opts.max_urls,
        },
        summary: CoverageSummary {
            urls_total: total,
            ok,
            thin,
            failed,
            ok_percent,
            breakdown,
        },
        results,
    }
}

async fn cover_single(
    client: &reqwest::Client,
    input_url: &str,
    opts: &CoverageOptions,
) -> CoverageResult {
    let fetch_start = Instant::now();
    let fetch_result = match fetch::fetch_url(client, input_url, opts.timeout_ms).await {
        Ok(r) => r,
        Err(e) => {
            let (kind, msg) = classify_fetch_error(&e);
            return CoverageResult {
                input_url: input_url.to_string(),
                final_url: None,
                status: CoverageStatus::Failed,
                http_status: None,
                content_type: None,
                title: None,
                html_bytes: None,
                som_bytes: None,
                element_count: None,
                interactive_count: None,
                fetch_ms: Some(fetch_start.elapsed().as_millis() as u64),
                pipeline_ms: None,
                js_total_scripts: None,
                js_succeeded: None,
                js_failed: None,
                js_errors: None,
                failure_kind: Some(kind),
                error: Some(msg),
            };
        }
    };

    let fetch_ms = fetch_start.elapsed().as_millis() as u64;

    // Filter non-HTML responses.
    if !fetch_result
        .content_type
        .to_lowercase()
        .contains("text/html")
    {
        return CoverageResult {
            input_url: input_url.to_string(),
            final_url: Some(fetch_result.url),
            status: CoverageStatus::Failed,
            http_status: Some(fetch_result.status),
            content_type: Some(fetch_result.content_type),
            title: None,
            html_bytes: Some(fetch_result.html_bytes),
            som_bytes: None,
            element_count: None,
            interactive_count: None,
            fetch_ms: Some(fetch_ms),
            pipeline_ms: None,
            js_total_scripts: None,
            js_succeeded: None,
            js_failed: None,
            js_errors: None,
            failure_kind: Some(FailureKind::NonHtml),
            error: Some("Non-HTML content-type".into()),
        };
    }

    let pipeline_start = Instant::now();

    // Pre-JS: compile SOM from raw HTML first (to compare with post-JS result).
    // Some sites (nodejs.org, store.steampowered.com) DEGRADE with JS because
    // JS overwrites the DOM with fewer elements. We keep whichever is richer.
    let pre_js_som = if opts.execute_js {
        compiler::compile(&fetch_result.html, &fetch_result.url).ok()
    } else {
        None
    };

    let mut config = PipelineConfig::default();
    config.execute_js = opts.execute_js;
    config.fetch_external_scripts = opts.fetch_external_scripts;
    config.timer_drain_ms = opts.timer_drain_ms;

    // Coverage runs must not crash. V8 OOM is fatal, so we run with a larger heap cap.
    config.js_config.max_heap_bytes = opts.js_max_heap_bytes;

    config.external_script_limits.max_external = opts.max_external_scripts;
    config.external_script_limits.max_script_bytes = opts.max_external_script_bytes;
    config.external_script_limits.max_total_bytes = opts.max_external_total_bytes;
    config.external_script_limits.timeout_ms = opts.external_script_timeout_ms;

    let page =
        match pipeline::process_page_async(&fetch_result.html, &fetch_result.url, &config, client)
            .await
        {
            Ok(r) => r,
            Err(e) => {
                return CoverageResult {
                    input_url: input_url.to_string(),
                    final_url: Some(fetch_result.url),
                    status: CoverageStatus::Failed,
                    http_status: Some(fetch_result.status),
                    content_type: Some(fetch_result.content_type),
                    title: None,
                    html_bytes: Some(fetch_result.html_bytes),
                    som_bytes: None,
                    element_count: None,
                    interactive_count: None,
                    fetch_ms: Some(fetch_ms),
                    pipeline_ms: Some(pipeline_start.elapsed().as_millis() as u64),
                    js_total_scripts: None,
                    js_succeeded: None,
                    js_failed: None,
                    js_errors: None,
                    failure_kind: Some(FailureKind::PipelineError),
                    error: Some(format!("{e:?}")),
                };
            }
        };

    let pipeline_ms = pipeline_start.elapsed().as_millis() as u64;

    // Compare pre-JS and post-JS SOMs, keep whichever has more elements.
    // This handles cases where JS destroys content (e.g., replaces body with loading spinner).
    let (final_som, used_pre_js) = match &pre_js_som {
        Some(pre) if pre.meta.element_count > page.som.meta.element_count => (pre, true),
        _ => (&page.som, false),
    };

    if used_pre_js {
        debug!(
            url = %input_url,
            pre_js_elements = pre_js_som.as_ref().map(|s| s.meta.element_count),
            post_js_elements = page.som.meta.element_count,
            "Using pre-JS SOM (JS degraded content)"
        );
    }

    let som_bytes = final_som.meta.som_bytes;
    let element_count = final_som.meta.element_count;
    let interactive_count = final_som.meta.interactive_count;

    let status = if is_thin(som_bytes, element_count, interactive_count) {
        CoverageStatus::Thin
    } else {
        CoverageStatus::Ok
    };

    let (js_total, js_succeeded, js_failed, js_errors) = page
        .js_report
        .as_ref()
        .map(|r| {
            let errors: Vec<String> = r
                .errors
                .iter()
                .map(|(label, err)| format!("{}: {}", label, err))
                .collect();
            let errors_opt = if errors.is_empty() { None } else { Some(errors) };
            (Some(r.total), Some(r.succeeded), Some(r.failed), errors_opt)
        })
        .unwrap_or((None, None, None, None));

    CoverageResult {
        input_url: input_url.to_string(),
        final_url: Some(fetch_result.url),
        status,
        http_status: Some(fetch_result.status),
        content_type: Some(fetch_result.content_type),
        title: Some(final_som.title.clone()),
        html_bytes: Some(fetch_result.html_bytes),
        som_bytes: Some(som_bytes),
        element_count: Some(element_count),
        interactive_count: Some(interactive_count),
        fetch_ms: Some(fetch_ms),
        pipeline_ms: Some(pipeline_ms),
        js_total_scripts: js_total,
        js_succeeded,
        js_failed,
        js_errors,
        failure_kind: None,
        error: None,
    }
}
