use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::cookie::Jar;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

use crate::js::pipeline::{self, PipelineConfig};
use crate::network::fetch;
use crate::process_supervisor::{self, ProcessOutcome, ProcessOutput, ProcessSpec};
use crate::som::compiler;

pub const COVERAGE_SCHEMA_VERSION: &str = "plasmate.coverage.v2";
const CORPUS_DIGEST_DOMAIN: &[u8] = b"plasmate.coverage.corpus.v1\0";
const CORPUS_DIGEST_SCOPE: &str = "selected_ordered_input_urls";
const CORPUS_CANONICALIZATION: &str =
    "plasmate.coverage.corpus.v1: domain separator, then u64be byte length + UTF-8 bytes per URL";

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// Execute JS-enabled pages in supervised child processes. V8 fatal errors
    /// abort the worker, not the coverage coordinator.
    pub isolate_js: bool,
    /// Linux address-space ceiling for each JS worker. Zero disables it.
    pub worker_memory_bytes: u64,
    /// Maximum stdout/stderr captured from one worker before truncation.
    pub worker_output_bytes: usize,
    /// Test/embedding override. Production uses the current executable.
    #[serde(skip)]
    pub worker_executable: Option<PathBuf>,
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
            isolate_js: true,
            worker_memory_bytes: 0,
            worker_output_bytes: 256 * 1024,
            worker_executable: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageStatus {
    Ok,
    Blocked,
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
    WorkerCrash,
    WorkerResourceExhaustion,
    WorkerExit,
    WorkerProtocolError,
    WorkerSpawnError,
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
    pub compression_ratio: Option<f64>,
    pub element_count: Option<usize>,
    pub interactive_count: Option<usize>,

    pub fetch_ms: Option<u64>,
    pub pipeline_ms: Option<u64>,

    pub js_total_scripts: Option<usize>,
    pub js_succeeded: Option<usize>,
    pub js_failed: Option<usize>,

    /// Parent-observed evidence for an isolated JS coverage worker. This is
    /// absent for non-JS/in-process coverage and populated for every attempted
    /// supervised worker, including launch and protocol failures.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worker: Option<CoverageWorkerEvidence>,

    pub failure_kind: Option<FailureKind>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CoverageWorkerOutcome {
    Success,
    Blocked,
    PageError,
    Timeout,
    Signaled,
    Exited,
    ResourceExhaustion,
    OutputLimit,
    MalformedOutput,
    LaunchFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageWorkerEvidence {
    pub outcome: CoverageWorkerOutcome,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic_excerpt: Option<String>,
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
    pub blocked: usize,
    pub failed: usize,
    /// Success rate across every input URL, including blocked sites.
    #[serde(default)]
    pub success_percent: f64,
    #[serde(default)]
    pub timed_out: usize,
    #[serde(default)]
    pub worker_crashes: usize,
    #[serde(default)]
    pub worker_resource_exhaustions: usize,
    #[serde(default)]
    pub worker_exits: usize,
    /// Coordinator setup or worker-protocol errors, excluding page crashes.
    #[serde(default)]
    pub worker_errors: usize,
    /// Fatal/invalid worker outcomes that should fail automation after the
    /// complete report has been written.
    #[serde(default)]
    pub infrastructure_failures: usize,
    pub parsed_percent: f64,
    pub median_ratio: f64,
    pub mean_ratio: f64,
    pub p95_ratio: f64,
    /// Number of successful results contributing to compression statistics.
    pub compression_samples: usize,
    /// Non-overlapping outcome buckets for the complete input denominator.
    /// The legacy `failed` field above includes crash and timeout results.
    pub outcomes: CoverageOutcomes,
    pub breakdown: Vec<CoverageBreakdownItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CoverageOutcomes {
    pub inputs_total: usize,
    pub success: usize,
    pub blocked: usize,
    pub failed: usize,
    pub crash: usize,
    #[serde(default)]
    pub resource_exhaustion: usize,
    pub timeout: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub schema_version: String,
    pub generated_at_utc: String,
    pub plasmate_version: String,
    pub corpus: CoverageCorpus,
    pub environment: CoverageEnvironment,
    pub measurement: CoverageMeasurement,
    pub options: CoverageReportOptions,
    pub summary: CoverageSummary,
    pub results: Vec<CoverageResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageCorpus {
    /// SHA-256 of the exact ordered URL sequence selected after parsing and `max_urls`.
    pub sha256: String,
    pub digest_scope: String,
    pub canonicalization: String,
    pub inputs_total: usize,
    pub ordered_input_urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageEnvironment {
    pub git_commit: Option<String>,
    pub git_dirty: Option<bool>,
    pub rustc_version: Option<String>,
    pub build_profile: String,
    pub operating_system: String,
    pub architecture: String,
    pub runner: CoverageRunner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageRunner {
    pub provider: String,
    pub name: Option<String>,
    pub operating_system: Option<String>,
    pub architecture: Option<String>,
    pub environment: Option<String>,
    pub repository: Option<String>,
    pub workflow: Option<String>,
    pub run_id: Option<String>,
    pub run_attempt: Option<String>,
    pub event_name: Option<String>,
    pub head_sha: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageMeasurement {
    pub cache: CoverageCacheEvidence,
    pub latency: CoverageLatencyEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageCacheEvidence {
    pub collected: bool,
    pub repetitions_per_input: usize,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageLatencyEvidence {
    pub collected: bool,
    pub method: String,
    pub cache_state: String,
    pub per_input_fields: Vec<String>,
    pub fetch_samples: usize,
    pub pipeline_samples: usize,
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
    #[serde(default)]
    pub isolate_js: bool,
    #[serde(default)]
    pub worker_memory_bytes: u64,
    #[serde(default)]
    pub worker_output_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageWorkerRequest {
    pub input_url: String,
    pub options: CoverageOptions,
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
        fetch::FetchError::UrlBlocked(msg) => (
            FailureKind::NavigationFailed,
            format!("Outbound URL blocked: {msg}"),
        ),
        fetch::FetchError::TooManyRedirects(limit) => (
            FailureKind::NavigationFailed,
            format!("Too many redirects (maximum {limit})"),
        ),
        fetch::FetchError::BodyTooLarge { limit } => (
            FailureKind::NavigationFailed,
            format!("Response body exceeds {limit} bytes"),
        ),
    }
}

fn compute_ratio_stats(ratios: &mut [f64]) -> (f64, f64, f64) {
    if ratios.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    ratios.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mean = ratios.iter().sum::<f64>() / ratios.len() as f64;
    let median = if ratios.len() & 1 == 0 {
        (ratios[ratios.len() / 2 - 1] + ratios[ratios.len() / 2]) / 2.0
    } else {
        ratios[ratios.len() / 2]
    };
    let p95_idx = ((ratios.len() as f64) * 0.95).ceil() as usize;
    let p95 = ratios[p95_idx.min(ratios.len() - 1)];
    (median, mean, p95)
}

pub fn parse_urls_file(content: &str) -> Vec<String> {
    content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect()
}

/// Hash the exact ordered input sequence independently of line endings, comments,
/// or ambiguous URL separators. Length framing means two different URL sequences
/// cannot produce the same canonical byte stream through concatenation alone.
pub fn corpus_sha256(urls: &[String]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(CORPUS_DIGEST_DOMAIN);
    for url in urls {
        hasher.update((url.len() as u64).to_be_bytes());
        hasher.update(url.as_bytes());
    }
    hex::encode(hasher.finalize())
}

fn command_output(program: &str, arguments: &[&str]) -> Option<String> {
    Command::new(program)
        .args(arguments)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|output| output.trim().to_string())
        .filter(|output| !output.is_empty())
}

fn optional_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn coverage_environment() -> CoverageEnvironment {
    let github_actions = optional_env("GITHUB_ACTIONS").as_deref() == Some("true");
    let generic_ci = optional_env("CI").as_deref() == Some("true");
    let compile_time_git_commit = option_env!("PLASMATE_BUILD_GIT_SHA")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(String::from);

    CoverageEnvironment {
        // CI sets PLASMATE_BUILD_GIT_SHA while compiling. Outside CI, runtime
        // repository metadata is explicitly best-effort and therefore optional.
        git_commit: compile_time_git_commit
            .or_else(|| command_output("git", &["rev-parse", "HEAD"])),
        git_dirty: Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|output| !output.stdout.is_empty()),
        rustc_version: command_output("rustc", &["--version"]),
        build_profile: if cfg!(debug_assertions) {
            "debug".to_string()
        } else {
            "release".to_string()
        },
        operating_system: std::env::consts::OS.to_string(),
        architecture: std::env::consts::ARCH.to_string(),
        runner: CoverageRunner {
            provider: if github_actions {
                "github_actions"
            } else if generic_ci {
                "generic_ci"
            } else {
                "local"
            }
            .to_string(),
            name: optional_env("RUNNER_NAME"),
            operating_system: optional_env("RUNNER_OS"),
            architecture: optional_env("RUNNER_ARCH"),
            environment: optional_env("RUNNER_ENVIRONMENT"),
            repository: optional_env("GITHUB_REPOSITORY"),
            workflow: optional_env("GITHUB_WORKFLOW"),
            run_id: optional_env("GITHUB_RUN_ID"),
            run_attempt: optional_env("GITHUB_RUN_ATTEMPT"),
            event_name: optional_env("GITHUB_EVENT_NAME"),
            head_sha: optional_env("GITHUB_SHA"),
        },
    }
}

fn classify_outcomes(results: &[CoverageResult]) -> CoverageOutcomes {
    let mut outcomes = CoverageOutcomes {
        inputs_total: results.len(),
        ..CoverageOutcomes::default()
    };
    for result in results {
        match (&result.status, &result.failure_kind) {
            (CoverageStatus::Ok, _) => outcomes.success += 1,
            (CoverageStatus::Blocked, _) => outcomes.blocked += 1,
            (CoverageStatus::Failed, Some(FailureKind::WorkerCrash)) => outcomes.crash += 1,
            (CoverageStatus::Failed, Some(FailureKind::WorkerResourceExhaustion)) => {
                outcomes.resource_exhaustion += 1
            }
            (CoverageStatus::Failed, Some(FailureKind::Timeout)) => outcomes.timeout += 1,
            (CoverageStatus::Failed, _) => outcomes.failed += 1,
        }
    }
    outcomes
}

fn is_lowercase_hex(value: &str, allowed_lengths: &[usize]) -> bool {
    allowed_lengths.contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

/// Validate structural evidence without treating observed public-site failures as
/// report corruption. A valid report may contain any mix of outcome buckets.
pub fn validate_evidence(report: &CoverageReport) -> Result<(), String> {
    if report.schema_version != COVERAGE_SCHEMA_VERSION {
        return Err(format!(
            "unsupported coverage schema {}; expected {COVERAGE_SCHEMA_VERSION}",
            report.schema_version
        ));
    }
    if !is_lowercase_hex(&report.corpus.sha256, &[64]) {
        return Err("corpus.sha256 must be 64 lowercase hexadecimal characters".to_string());
    }
    if report.corpus.digest_scope != CORPUS_DIGEST_SCOPE
        || report.corpus.canonicalization != CORPUS_CANONICALIZATION
    {
        return Err("coverage corpus digest contract is not recognized".to_string());
    }
    if corpus_sha256(&report.corpus.ordered_input_urls) != report.corpus.sha256 {
        return Err("corpus.sha256 does not match ordered_input_urls".to_string());
    }
    if let Some(commit) = report.environment.git_commit.as_deref() {
        if !is_lowercase_hex(commit, &[40, 64]) {
            return Err(
                "environment.git_commit must be a complete lowercase Git object id".to_string(),
            );
        }
    }
    if report.environment.runner.provider == "github_actions" {
        let commit = report
            .environment
            .git_commit
            .as_deref()
            .ok_or_else(|| "GitHub Actions evidence is missing git_commit".to_string())?;
        let head_sha = report
            .environment
            .runner
            .head_sha
            .as_deref()
            .ok_or_else(|| "GitHub Actions evidence is missing head_sha".to_string())?;
        if commit != head_sha {
            return Err(
                "compiled git_commit does not match the GitHub Actions head_sha".to_string(),
            );
        }
        if report.environment.git_dirty.is_none() || report.environment.rustc_version.is_none() {
            return Err(
                "GitHub Actions evidence is missing dirty-state or compiler metadata".to_string(),
            );
        }
    }
    if report.corpus.inputs_total != report.corpus.ordered_input_urls.len()
        || report.corpus.inputs_total != report.results.len()
        || report.summary.urls_total != report.results.len()
        || report.summary.outcomes.inputs_total != report.results.len()
    {
        return Err("coverage input denominators do not match results length".to_string());
    }
    let mut corpus_urls = report.corpus.ordered_input_urls.clone();
    let mut result_urls: Vec<String> = report
        .results
        .iter()
        .map(|result| result.input_url.clone())
        .collect();
    corpus_urls.sort();
    result_urls.sort();
    if corpus_urls != result_urls {
        return Err("coverage corpus URLs do not match per-input results".to_string());
    }
    let classified = report.summary.outcomes.success
        + report.summary.outcomes.blocked
        + report.summary.outcomes.failed
        + report.summary.outcomes.crash
        + report.summary.outcomes.resource_exhaustion
        + report.summary.outcomes.timeout;
    if classified != report.summary.outcomes.inputs_total {
        return Err("coverage outcome buckets do not partition inputs_total".to_string());
    }
    let observed = classify_outcomes(&report.results);
    if observed != report.summary.outcomes {
        return Err("coverage outcome buckets do not match per-input results".to_string());
    }
    if report.summary.ok != report.summary.outcomes.success
        || report.summary.blocked != report.summary.outcomes.blocked
        || report.summary.failed
            != report.summary.outcomes.failed
                + report.summary.outcomes.crash
                + report.summary.outcomes.resource_exhaustion
                + report.summary.outcomes.timeout
    {
        return Err("legacy coverage aggregates disagree with outcome buckets".to_string());
    }
    if report.summary.timed_out != report.summary.outcomes.timeout
        || report.summary.worker_crashes != report.summary.outcomes.crash
        || report.summary.worker_resource_exhaustions != report.summary.outcomes.resource_exhaustion
    {
        return Err("worker outcome aggregates disagree with outcome buckets".to_string());
    }
    if report.measurement.cache.collected
        || report.measurement.cache.repetitions_per_input != 1
        || report.measurement.latency.cache_state != "not_measured"
    {
        return Err(
            "public coverage must not claim cache evidence without controlled repeats".to_string(),
        );
    }
    let fetch_samples = report
        .results
        .iter()
        .filter(|result| result.fetch_ms.is_some())
        .count();
    let pipeline_samples = report
        .results
        .iter()
        .filter(|result| result.pipeline_ms.is_some())
        .count();
    if report.measurement.latency.fetch_samples != fetch_samples
        || report.measurement.latency.pipeline_samples != pipeline_samples
        || report.measurement.latency.collected != (fetch_samples + pipeline_samples > 0)
    {
        return Err("latency evidence denominators do not match per-input results".to_string());
    }
    let compression_samples = report
        .results
        .iter()
        .filter(|result| result.compression_ratio.is_some())
        .count();
    if report.summary.compression_samples != compression_samples {
        return Err(
            "compression evidence denominator does not match per-input results".to_string(),
        );
    }
    Ok(())
}

pub async fn run(urls: &[String], opts: &CoverageOptions) -> CoverageReport {
    let jar = Arc::new(Jar::default());
    let client = fetch::build_client(None, jar, None).expect("Failed to build HTTP client");

    let max = opts.max_urls.unwrap_or(urls.len());
    let urls: Vec<String> = urls.iter().take(max).cloned().collect();
    let corpus_sha256 = corpus_sha256(&urls);
    let ordered_input_urls = urls.clone();

    info!(count = urls.len(), "Running coverage suite");

    let sem = Arc::new(Semaphore::new(opts.concurrency.max(1)));
    let mut handles = Vec::new();

    for input_url in urls {
        let client = client.clone();
        let sem = sem.clone();
        let opts = opts.clone();
        let task_url = input_url.clone();

        handles.push((
            task_url,
            tokio::spawn(async move {
                let _permit = sem.acquire().await.expect("semaphore poisoned");

                let timeout = std::time::Duration::from_millis(opts.timeout_ms);
                let page = async {
                    if opts.execute_js && opts.isolate_js {
                        cover_single_supervised(&input_url, &opts).await
                    } else {
                        cover_single(&client, &input_url, &opts).await
                    }
                };
                match tokio::time::timeout(timeout + Duration::from_secs(2), page).await {
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
                        compression_ratio: None,
                        element_count: None,
                        interactive_count: None,
                        fetch_ms: None,
                        pipeline_ms: None,
                        js_total_scripts: None,
                        js_succeeded: None,
                        js_failed: None,
                        worker: None,
                        failure_kind: Some(FailureKind::Timeout),
                        error: Some(format!("Overall timeout after {}ms", opts.timeout_ms)),
                    },
                }
            }),
        ));
    }

    let mut results = Vec::new();
    for (input_url, h) in handles {
        match h.await {
            Ok(r) => results.push(r),
            Err(e) => {
                warn!(error = %e, "Coverage task join error");
                results.push(failed_result(
                    input_url,
                    FailureKind::WorkerProtocolError,
                    format!("Coverage coordinator task failed: {e}"),
                ));
            }
        }
    }

    // Stable-ish ordering for diff readability.
    results.sort_by(|a, b| a.input_url.cmp(&b.input_url));

    let mut ok = 0usize;
    let mut blocked = 0usize;
    let mut failed = 0usize;
    let mut timed_out = 0usize;
    let mut worker_crashes = 0usize;
    let mut worker_resource_exhaustions = 0usize;
    let mut worker_exits = 0usize;
    let mut worker_errors = 0usize;
    let mut ratios: Vec<f64> = Vec::new();

    let mut breakdown: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();

    for r in &results {
        match r.status {
            CoverageStatus::Ok => {
                ok += 1;
                if let Some(ratio) = r.compression_ratio {
                    ratios.push(ratio);
                }
            }
            CoverageStatus::Blocked => blocked += 1,
            CoverageStatus::Failed => failed += 1,
        }

        match &r.failure_kind {
            Some(FailureKind::Timeout) => timed_out += 1,
            Some(FailureKind::WorkerCrash) => worker_crashes += 1,
            Some(FailureKind::WorkerResourceExhaustion) => worker_resource_exhaustions += 1,
            Some(FailureKind::WorkerExit) => worker_exits += 1,
            Some(FailureKind::WorkerProtocolError | FailureKind::WorkerSpawnError) => {
                worker_errors += 1
            }
            _ => {}
        }

        let key = match (&r.status, &r.failure_kind) {
            (CoverageStatus::Ok, _) => "ok".to_string(),
            (CoverageStatus::Blocked, _) => "blocked".to_string(),
            (CoverageStatus::Failed, Some(k)) => format!("failed:{k:?}").to_lowercase(),
            (CoverageStatus::Failed, None) => "failed:unknown".to_string(),
        };
        *breakdown.entry(key).or_insert(0) += 1;
    }

    let total = results.len();
    let parseable = total - blocked;
    let success_percent = if total == 0 {
        0.0
    } else {
        (ok as f64 / total as f64) * 100.0
    };
    let parsed_percent = if parseable == 0 {
        0.0
    } else {
        (ok as f64 / parseable as f64) * 100.0
    };

    let (median_ratio, mean_ratio, p95_ratio) = compute_ratio_stats(&mut ratios);
    let compression_samples = ratios.len();
    let fetch_samples = results
        .iter()
        .filter(|result| result.fetch_ms.is_some())
        .count();
    let pipeline_samples = results
        .iter()
        .filter(|result| result.pipeline_ms.is_some())
        .count();

    let breakdown = breakdown
        .into_iter()
        .map(|(key, count)| CoverageBreakdownItem { key, count })
        .collect();
    let outcomes = classify_outcomes(&results);

    CoverageReport {
        schema_version: COVERAGE_SCHEMA_VERSION.to_string(),
        generated_at_utc: now_utc_rfc3339ish(),
        plasmate_version: env!("CARGO_PKG_VERSION").to_string(),
        corpus: CoverageCorpus {
            sha256: corpus_sha256,
            digest_scope: CORPUS_DIGEST_SCOPE.to_string(),
            canonicalization: CORPUS_CANONICALIZATION.to_string(),
            inputs_total: total,
            ordered_input_urls,
        },
        environment: coverage_environment(),
        measurement: CoverageMeasurement {
            cache: CoverageCacheEvidence {
                collected: false,
                repetitions_per_input: 1,
                reason: "each public URL is fetched once; cold/warm cache state is not measured"
                    .to_string(),
            },
            latency: CoverageLatencyEvidence {
                collected: fetch_samples + pipeline_samples > 0,
                method: "single_pass_monotonic_wall_clock".to_string(),
                cache_state: "not_measured".to_string(),
                per_input_fields: vec!["fetch_ms".to_string(), "pipeline_ms".to_string()],
                fetch_samples,
                pipeline_samples,
            },
        },
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
            isolate_js: opts.isolate_js,
            worker_memory_bytes: opts.worker_memory_bytes,
            worker_output_bytes: opts.worker_output_bytes,
        },
        summary: CoverageSummary {
            urls_total: total,
            ok,
            blocked,
            failed,
            success_percent,
            timed_out,
            worker_crashes,
            worker_resource_exhaustions,
            worker_exits,
            worker_errors,
            infrastructure_failures: worker_errors,
            parsed_percent,
            median_ratio,
            mean_ratio,
            p95_ratio,
            compression_samples,
            outcomes,
            breakdown,
        },
        results,
    }
}

fn failed_result(input_url: String, kind: FailureKind, error: String) -> CoverageResult {
    CoverageResult {
        input_url,
        final_url: None,
        status: CoverageStatus::Failed,
        http_status: None,
        content_type: None,
        title: None,
        html_bytes: None,
        som_bytes: None,
        compression_ratio: None,
        element_count: None,
        interactive_count: None,
        fetch_ms: None,
        pipeline_ms: None,
        js_total_scripts: None,
        js_succeeded: None,
        js_failed: None,
        worker: None,
        failure_kind: Some(kind),
        error: Some(error),
    }
}

fn bounded_diagnostic(stderr: &[u8], truncated: bool) -> String {
    let mut diagnostic = String::from_utf8_lossy(stderr).trim().to_string();
    if diagnostic.len() > 2048 {
        diagnostic.truncate(2048);
        diagnostic.push('…');
    }
    if truncated {
        diagnostic.push_str(" [worker stderr truncated]");
    }
    diagnostic
}

fn diagnostic_is_resource_exhaustion(diagnostic: &str) -> bool {
    let normalized = diagnostic.to_ascii_lowercase();
    [
        "heap out of memory",
        "out of memory",
        "memory allocation of",
        "allocation failed",
        "failed to reserve address space",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
}

fn worker_evidence(
    outcome: CoverageWorkerOutcome,
    duration_ms: u64,
    exit_code: Option<i32>,
    signal: Option<i32>,
    diagnostic: &str,
) -> CoverageWorkerEvidence {
    CoverageWorkerEvidence {
        outcome,
        duration_ms,
        exit_code,
        signal,
        diagnostic_excerpt: (!diagnostic.is_empty()).then(|| diagnostic.to_string()),
    }
}

fn with_worker_evidence(
    mut result: CoverageResult,
    evidence: CoverageWorkerEvidence,
) -> CoverageResult {
    result.worker = Some(evidence);
    result
}

async fn cover_single_supervised(input_url: &str, opts: &CoverageOptions) -> CoverageResult {
    let worker_start = Instant::now();
    let executable = match &opts.worker_executable {
        Some(path) => path.clone(),
        None => match std::env::current_exe() {
            Ok(path) => path,
            Err(error) => {
                let message = format!("Cannot resolve coverage worker executable: {error}");
                return with_worker_evidence(
                    failed_result(
                        input_url.to_string(),
                        FailureKind::WorkerSpawnError,
                        message.clone(),
                    ),
                    worker_evidence(
                        CoverageWorkerOutcome::LaunchFailure,
                        worker_start.elapsed().as_millis() as u64,
                        None,
                        None,
                        &message,
                    ),
                );
            }
        },
    };

    let mut worker_options = opts.clone();
    worker_options.isolate_js = false;
    worker_options.worker_executable = None;
    let request = CoverageWorkerRequest {
        input_url: input_url.to_string(),
        options: worker_options,
    };
    let stdin = match serde_json::to_vec(&request) {
        Ok(stdin) => stdin,
        Err(error) => {
            return failed_result(
                input_url.to_string(),
                FailureKind::WorkerProtocolError,
                format!("Cannot encode coverage worker request: {error}"),
            );
        }
    };

    let output = process_supervisor::supervise(ProcessSpec {
        program: executable,
        args: vec![OsString::from("__coverage-worker")],
        env: Vec::new(),
        stdin,
        timeout: Duration::from_millis(opts.timeout_ms),
        max_stdout_bytes: opts.worker_output_bytes,
        max_stderr_bytes: opts.worker_output_bytes,
        memory_limit_bytes: opts.worker_memory_bytes,
    })
    .await;

    let output = match output {
        Ok(output) => output,
        Err(error) => {
            let message = error.to_string();
            return with_worker_evidence(
                failed_result(
                    input_url.to_string(),
                    FailureKind::WorkerSpawnError,
                    message.clone(),
                ),
                worker_evidence(
                    CoverageWorkerOutcome::LaunchFailure,
                    worker_start.elapsed().as_millis() as u64,
                    None,
                    None,
                    &message,
                ),
            );
        }
    };
    classify_worker_output(
        input_url,
        opts.timeout_ms,
        worker_start.elapsed().as_millis() as u64,
        output,
    )
}

fn classify_worker_output(
    input_url: &str,
    timeout_ms: u64,
    duration_ms: u64,
    output: ProcessOutput,
) -> CoverageResult {
    let diagnostic = bounded_diagnostic(&output.stderr, output.stderr_truncated);
    if diagnostic_is_resource_exhaustion(&diagnostic)
        && !matches!(output.outcome, ProcessOutcome::Exited { code: 0 })
    {
        let (exit_code, signal) = match output.outcome {
            ProcessOutcome::Exited { code } => (Some(code), None),
            ProcessOutcome::Signaled { signal } => (None, Some(signal)),
            ProcessOutcome::TimedOut => (None, None),
        };
        return with_worker_evidence(
            failed_result(
                input_url.to_string(),
                FailureKind::WorkerResourceExhaustion,
                format!("Supervised JS worker exhausted resources: {diagnostic}"),
            ),
            worker_evidence(
                CoverageWorkerOutcome::ResourceExhaustion,
                duration_ms,
                exit_code,
                signal,
                &diagnostic,
            ),
        );
    }

    match output.outcome {
        ProcessOutcome::TimedOut => with_worker_evidence(
            failed_result(
                input_url.to_string(),
                FailureKind::Timeout,
                format!(
                    "Supervised JS worker exceeded {}ms{}",
                    timeout_ms,
                    if diagnostic.is_empty() {
                        String::new()
                    } else {
                        format!(": {diagnostic}")
                    }
                ),
            ),
            worker_evidence(
                CoverageWorkerOutcome::Timeout,
                duration_ms,
                None,
                None,
                &diagnostic,
            ),
        ),
        ProcessOutcome::Signaled { signal } => with_worker_evidence(
            failed_result(
                input_url.to_string(),
                FailureKind::WorkerCrash,
                format!(
                    "Supervised JS worker terminated by signal {signal}{}",
                    if diagnostic.is_empty() {
                        String::new()
                    } else {
                        format!(": {diagnostic}")
                    }
                ),
            ),
            worker_evidence(
                CoverageWorkerOutcome::Signaled,
                duration_ms,
                None,
                Some(signal),
                &diagnostic,
            ),
        ),
        ProcessOutcome::Exited { code } if code != 0 => with_worker_evidence(
            failed_result(
                input_url.to_string(),
                FailureKind::WorkerExit,
                format!(
                    "Supervised JS worker exited with code {code}{}",
                    if diagnostic.is_empty() {
                        String::new()
                    } else {
                        format!(": {diagnostic}")
                    }
                ),
            ),
            worker_evidence(
                CoverageWorkerOutcome::Exited,
                duration_ms,
                Some(code),
                None,
                &diagnostic,
            ),
        ),
        ProcessOutcome::Exited { code } if output.stdout_truncated => with_worker_evidence(
            failed_result(
                input_url.to_string(),
                FailureKind::WorkerProtocolError,
                "Supervised JS worker response exceeded the output limit".to_string(),
            ),
            worker_evidence(
                CoverageWorkerOutcome::OutputLimit,
                duration_ms,
                Some(code),
                None,
                &diagnostic,
            ),
        ),
        ProcessOutcome::Exited { code } => {
            match serde_json::from_slice::<CoverageResult>(&output.stdout) {
                Ok(result) => {
                    let outcome = match &result.status {
                        CoverageStatus::Ok => CoverageWorkerOutcome::Success,
                        CoverageStatus::Blocked => CoverageWorkerOutcome::Blocked,
                        CoverageStatus::Failed => CoverageWorkerOutcome::PageError,
                    };
                    with_worker_evidence(
                        result,
                        worker_evidence(outcome, duration_ms, Some(code), None, &diagnostic),
                    )
                }
                Err(error) => {
                    let message = format!("Invalid coverage worker response: {error}");
                    with_worker_evidence(
                        failed_result(
                            input_url.to_string(),
                            FailureKind::WorkerProtocolError,
                            message.clone(),
                        ),
                        worker_evidence(
                            CoverageWorkerOutcome::MalformedOutput,
                            duration_ms,
                            Some(code),
                            None,
                            if diagnostic.is_empty() {
                                &message
                            } else {
                                &diagnostic
                            },
                        ),
                    )
                }
            }
        }
    }
}

/// Run exactly one page inside a coverage worker process.
pub async fn run_worker(mut request: CoverageWorkerRequest) -> CoverageResult {
    request.options.isolate_js = false;
    request.options.worker_executable = None;
    let jar = Arc::new(Jar::default());
    let client = match fetch::build_client(None, jar, None) {
        Ok(client) => client,
        Err(error) => {
            return failed_result(
                request.input_url,
                FailureKind::WorkerSpawnError,
                format!("Coverage worker could not build HTTP client: {error}"),
            );
        }
    };
    cover_single(&client, &request.input_url, &request.options).await
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
            // 401/403 = site blocked us, not a Plasmate failure.
            if let fetch::FetchError::HttpError { status, .. } = &e {
                if *status == 401 || *status == 403 {
                    return CoverageResult {
                        input_url: input_url.to_string(),
                        final_url: None,
                        status: CoverageStatus::Blocked,
                        http_status: Some(*status),
                        content_type: None,
                        title: None,
                        html_bytes: None,
                        som_bytes: None,
                        compression_ratio: None,
                        element_count: None,
                        interactive_count: None,
                        fetch_ms: Some(fetch_start.elapsed().as_millis() as u64),
                        pipeline_ms: None,
                        js_total_scripts: None,
                        js_succeeded: None,
                        js_failed: None,
                        worker: None,
                        failure_kind: None,
                        error: Some(format!("HTTP {status} — site blocked request")),
                    };
                }
            }
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
                compression_ratio: None,
                element_count: None,
                interactive_count: None,
                fetch_ms: Some(fetch_start.elapsed().as_millis() as u64),
                pipeline_ms: None,
                js_total_scripts: None,
                js_succeeded: None,
                js_failed: None,
                worker: None,
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
            compression_ratio: None,
            element_count: None,
            interactive_count: None,
            fetch_ms: Some(fetch_ms),
            pipeline_ms: None,
            js_total_scripts: None,
            js_succeeded: None,
            js_failed: None,
            worker: None,
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

    let mut config = PipelineConfig {
        execute_js: opts.execute_js,
        fetch_external_scripts: opts.fetch_external_scripts,
        timer_drain_ms: opts.timer_drain_ms,
        // `run_worker` is already the one-URL process boundary supervised by
        // the coverage coordinator; do not create a redundant nested worker.
        isolate_js: false,
        ..Default::default()
    };

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
                    compression_ratio: None,
                    element_count: None,
                    interactive_count: None,
                    fetch_ms: Some(fetch_ms),
                    pipeline_ms: Some(pipeline_start.elapsed().as_millis() as u64),
                    js_total_scripts: None,
                    js_succeeded: None,
                    js_failed: None,
                    worker: None,
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

    let compression_ratio = if som_bytes > 0 {
        Some(fetch_result.html_bytes as f64 / som_bytes as f64)
    } else {
        None
    };

    let (js_total, js_succeeded, js_failed) = page
        .js_report
        .as_ref()
        .map(|r| (Some(r.total), Some(r.succeeded), Some(r.failed)))
        .unwrap_or((None, None, None));

    CoverageResult {
        input_url: input_url.to_string(),
        final_url: Some(fetch_result.url),
        status: CoverageStatus::Ok,
        http_status: Some(fetch_result.status),
        content_type: Some(fetch_result.content_type),
        title: Some(final_som.title.clone()),
        html_bytes: Some(fetch_result.html_bytes),
        som_bytes: Some(som_bytes),
        compression_ratio,
        element_count: Some(element_count),
        interactive_count: Some(interactive_count),
        fetch_ms: Some(fetch_ms),
        pipeline_ms: Some(pipeline_ms),
        js_total_scripts: js_total,
        js_succeeded,
        js_failed,
        worker: None,
        failure_kind: None,
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn worker_output(outcome: ProcessOutcome) -> ProcessOutput {
        ProcessOutput {
            outcome,
            stdout: Vec::new(),
            stderr: Vec::new(),
            stdout_truncated: false,
            stderr_truncated: false,
        }
    }

    #[test]
    fn worker_timeout_is_a_per_url_timeout() {
        let result = classify_worker_output(
            "https://example.test",
            123,
            17,
            worker_output(ProcessOutcome::TimedOut),
        );
        assert!(matches!(result.failure_kind, Some(FailureKind::Timeout)));
        assert!(result.error.unwrap().contains("123ms"));
        assert_eq!(result.worker.unwrap().duration_ms, 17);
    }

    #[test]
    fn worker_signal_is_a_per_url_crash() {
        let result = classify_worker_output(
            "https://example.test",
            123,
            18,
            worker_output(ProcessOutcome::Signaled { signal: 9 }),
        );
        assert!(matches!(
            result.failure_kind,
            Some(FailureKind::WorkerCrash)
        ));
        assert_eq!(result.worker.unwrap().signal, Some(9));
    }

    #[test]
    fn worker_nonzero_exit_is_a_per_url_exit() {
        let result = classify_worker_output(
            "https://example.test",
            123,
            19,
            worker_output(ProcessOutcome::Exited { code: 17 }),
        );
        assert!(matches!(result.failure_kind, Some(FailureKind::WorkerExit)));
        assert_eq!(result.worker.unwrap().exit_code, Some(17));
    }

    #[test]
    fn worker_oom_diagnostic_is_distinct_resource_exhaustion() {
        let mut output = worker_output(ProcessOutcome::Signaled { signal: 6 });
        output.stderr =
            b"FATAL ERROR: Reached heap limit Allocation failed - JavaScript heap out of memory"
                .to_vec();
        let result = classify_worker_output("https://example.test", 123, 20, output);
        assert!(matches!(
            result.failure_kind,
            Some(FailureKind::WorkerResourceExhaustion)
        ));
        let evidence = result.worker.unwrap();
        assert_eq!(evidence.outcome, CoverageWorkerOutcome::ResourceExhaustion);
        assert_eq!(evidence.signal, Some(6));
        assert_eq!(evidence.duration_ms, 20);
    }

    #[test]
    fn malformed_worker_response_is_an_infrastructure_error() {
        let mut output = worker_output(ProcessOutcome::Exited { code: 0 });
        output.stdout = b"not-json".to_vec();
        let result = classify_worker_output("https://example.test", 123, 21, output);
        assert!(matches!(
            result.failure_kind,
            Some(FailureKind::WorkerProtocolError)
        ));
    }

    #[test]
    fn corpus_digest_is_stable_and_order_sensitive() {
        let urls = vec![
            "https://example.test/a".to_string(),
            "https://example.test/b".to_string(),
        ];
        assert_eq!(
            corpus_sha256(&urls),
            "7dcf9d5573c32b6e35956f4ef782e9918357a4cc2461470d75ccdcb29c0cfb2f"
        );

        let mut reversed = urls;
        reversed.reverse();
        assert_ne!(corpus_sha256(&reversed), corpus_sha256(&[]));
        assert_ne!(
            corpus_sha256(&reversed),
            "7dcf9d5573c32b6e35956f4ef782e9918357a4cc2461470d75ccdcb29c0cfb2f"
        );
    }

    #[test]
    fn outcome_buckets_are_mutually_exclusive() {
        let results = vec![
            CoverageResult {
                status: CoverageStatus::Ok,
                failure_kind: None,
                error: None,
                ..failed_result("ok".to_string(), FailureKind::Unknown, String::new())
            },
            CoverageResult {
                status: CoverageStatus::Blocked,
                failure_kind: None,
                error: Some("blocked".to_string()),
                ..failed_result("blocked".to_string(), FailureKind::Unknown, String::new())
            },
            failed_result("failed".to_string(), FailureKind::HttpError, String::new()),
            failed_result("crash".to_string(), FailureKind::WorkerCrash, String::new()),
            failed_result(
                "resource".to_string(),
                FailureKind::WorkerResourceExhaustion,
                String::new(),
            ),
            failed_result("timeout".to_string(), FailureKind::Timeout, String::new()),
        ];
        assert_eq!(
            classify_outcomes(&results),
            CoverageOutcomes {
                inputs_total: 6,
                success: 1,
                blocked: 1,
                failed: 1,
                crash: 1,
                resource_exhaustion: 1,
                timeout: 1,
            }
        );
    }

    #[tokio::test]
    async fn evidence_validation_accepts_observed_site_failures() {
        let mut report = run(&[], &CoverageOptions::default()).await;
        report.results = vec![failed_result(
            "https://unavailable.example.test".to_string(),
            FailureKind::NavigationFailed,
            "observed site failure".to_string(),
        )];
        report.corpus.inputs_total = 1;
        report.corpus.ordered_input_urls = vec!["https://unavailable.example.test".to_string()];
        report.corpus.sha256 = corpus_sha256(&report.corpus.ordered_input_urls);
        report.summary.urls_total = 1;
        report.summary.failed = 1;
        report.summary.outcomes = classify_outcomes(&report.results);

        validate_evidence(&report).expect("site failures are valid observational evidence");

        report.summary.outcomes.timeout = 1;
        assert!(validate_evidence(&report)
            .expect_err("overlapping outcome buckets must fail validation")
            .contains("partition"));
    }

    #[tokio::test]
    async fn coordinator_records_spawn_failures_for_every_url_and_continues() {
        let urls = vec![
            "https://one.example.test".to_string(),
            "https://two.example.test".to_string(),
        ];
        let options = CoverageOptions {
            concurrency: 2,
            max_urls: None,
            worker_executable: Some(PathBuf::from("/definitely/not/a/plasmate-worker")),
            ..CoverageOptions::default()
        };

        let report = run(&urls, &options).await;
        assert_eq!(report.results.len(), 2);
        assert_eq!(report.summary.failed, 2);
        assert_eq!(report.summary.infrastructure_failures, 2);
        assert!(report
            .results
            .iter()
            .all(|result| matches!(result.failure_kind, Some(FailureKind::WorkerSpawnError))));
    }
}
