//! Session management for concurrent page sessions.
//!
//! Pure infrastructure primitive — like SQLite's connection pool or tokio's
//! task spawning. Embeddable, zero-config, no external dependencies.
//!
//! Usage:
//! - CLI: `plasmate fetch url1 url2 --parallel 5`
//! - Library: `SessionManager::new()` → `create_session()` → `navigate()`
//! - CDP multi-target / AWP multi-session

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::cookie::Jar;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

use crate::js::pipeline::PipelineConfig;
use crate::network::fetch;
use crate::network::intercept::{
    InterceptAction, NetworkInterceptor, ResourceType as InterceptResourceType,
};
use crate::som::metadata::StructuredData;
use crate::som::types::Som;

pub const DEFAULT_MAX_SESSIONS: usize = 50;
pub const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 300;
pub const MAX_CONCURRENCY: usize = 20;
pub const DEFAULT_CONCURRENCY: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus { Idle, Navigating, Ready, Closed }

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionStatus::Idle => write!(f, "idle"),
            SessionStatus::Navigating => write!(f, "navigating"),
            SessionStatus::Ready => write!(f, "ready"),
            SessionStatus::Closed => write!(f, "closed"),
        }
    }
}

pub struct Session {
    pub id: String,
    pub url: Option<String>,
    pub html: Option<String>,
    pub effective_html: Option<String>,
    pub som: Option<Som>,
    pub structured_data: Option<StructuredData>,
    pub created_at: Instant,
    pub last_active: Instant,
    pub status: SessionStatus,
    pub client: Client,
    pub cookie_jar: Arc<Jar>,
    pub pipeline_config: PipelineConfig,
    pub interceptor: NetworkInterceptor,
    pub memory_bytes: usize,
    pub page_count: u64,
    pub history: Vec<NavigationEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NavigationEntry {
    pub url: String, pub title: String, pub status: u16,
    pub html_bytes: usize, pub som_bytes: usize,
    pub fetch_ms: u64, pub pipeline_ms: u64, pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionInfo {
    pub id: String, pub url: Option<String>, pub status: SessionStatus,
    pub page_count: u64, pub memory_bytes: usize,
    pub created_at_ms: u64, pub last_active_ms: u64, pub idle_secs: u64,
}

pub struct NavigateResult {
    pub url: String, pub status: u16, pub content_type: String,
    pub html_bytes: usize, pub som_bytes: usize,
    pub fetch_ms: u64, pub pipeline_ms: u64, pub title: String,
    pub js_report: Option<JsReportSummary>,
}

#[derive(Debug, Clone)]
pub struct JsReportSummary { pub total: usize, pub succeeded: usize, pub failed: usize }

#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_sessions: usize,
    pub idle_timeout: Duration,
    pub total_memory_budget: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        ResourceLimits {
            max_sessions: DEFAULT_MAX_SESSIONS,
            idle_timeout: Duration::from_secs(DEFAULT_IDLE_TIMEOUT_SECS),
            total_memory_budget: 0,
        }
    }
}

const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";

impl Session {
    pub fn new(id: String) -> Result<Self, String> {
        Self::new_with_options(id, None, &[])
    }

    pub fn new_with_options(id: String, user_agent: Option<&str>, auth_profiles: &[String]) -> Result<Self, String> {
        let jar = Arc::new(Jar::default());
        for domain in auth_profiles {
            if let Err(e) = crate::auth::store::load_into_jar(domain, &jar) {
                warn!(domain = %domain, error = %e, "Failed to load auth profile");
            }
        }
        let tls_config = crate::network::tls::global();
        let client = fetch::build_client_h1_fallback(
            Some(user_agent.unwrap_or(DEFAULT_USER_AGENT)), jar.clone(), tls_config,
        ).map_err(|e| e.to_string())?;

        let now = Instant::now();
        Ok(Session {
            id, url: None, html: None, effective_html: None, som: None,
            structured_data: None, created_at: now, last_active: now,
            status: SessionStatus::Idle, client, cookie_jar: jar,
            pipeline_config: PipelineConfig {
                execute_js: true, fetch_external_scripts: true, ..Default::default()
            },
            interceptor: NetworkInterceptor::new(),
            memory_bytes: 0, page_count: 0, history: Vec::new(),
        })
    }

    pub async fn navigate(&mut self, url: &str) -> Result<NavigateResult, String> {
        self.status = SessionStatus::Navigating;
        self.last_active = Instant::now();
        let start = Instant::now();

        let (action, _) = self.interceptor.check_request(url, &InterceptResourceType::Document, true);

        let mut fetch_result = match action {
            InterceptAction::Fulfill(params) => NetworkInterceptor::fulfill_request(&params, url),
            InterceptAction::Fail(reason) => {
                self.status = SessionStatus::Idle;
                return Err(NetworkInterceptor::fail_request(&reason, url).to_string());
            }
            InterceptAction::Continue(overrides) => {
                let actual_url = overrides.as_ref().and_then(|o| o.url.as_ref()).map(|u| u.as_str()).unwrap_or(url);
                let extra_headers = overrides.as_ref().and_then(|o| o.headers.clone()).unwrap_or_default();
                if extra_headers.is_empty() {
                    fetch::fetch_url(&self.client, actual_url, 30000).await
                        .map_err(|e| { self.status = SessionStatus::Idle; e.to_string() })?
                } else {
                    fetch::fetch_url_with_headers(&self.client, actual_url, 30000, &extra_headers).await
                        .map_err(|e| { self.status = SessionStatus::Idle; e.to_string() })?
                }
            }
        };

        self.interceptor.check_response(url, &InterceptResourceType::Document, &mut fetch_result);

        let fetch_ms = fetch_result.load_ms;
        let html_bytes = fetch_result.html_bytes;
        let status_code = fetch_result.status;
        let final_url = fetch_result.url.clone();
        let content_type = fetch_result.content_type.clone();

        let page_result = crate::js::pipeline::process_page_async(
            &fetch_result.html, &final_url, &self.pipeline_config, &self.client,
        ).await.map_err(|e| { self.status = SessionStatus::Idle; e.to_string() })?;

        let pipeline_ms = (page_result.timing.total_us / 1000) as u64;
        let som_bytes = page_result.som.meta.som_bytes;
        let title = page_result.som.title.clone();
        let js_report = page_result.js_report.as_ref().map(|r| JsReportSummary {
            total: r.total, succeeded: r.succeeded, failed: r.failed,
        });

        self.url = Some(final_url.clone());
        self.html = Some(fetch_result.html);
        self.effective_html = Some(page_result.effective_html);
        self.structured_data = page_result.som.structured_data.clone();
        self.som = Some(page_result.som);
        self.status = SessionStatus::Ready;
        self.last_active = Instant::now();
        self.page_count += 1;
        self.memory_bytes = html_bytes + som_bytes;

        self.history.push(NavigationEntry {
            url: final_url.clone(), title: title.clone(), status: status_code,
            html_bytes, som_bytes, fetch_ms, pipeline_ms,
            timestamp_ms: start.elapsed().as_millis() as u64,
        });

        Ok(NavigateResult {
            url: final_url, status: status_code, content_type,
            html_bytes, som_bytes, fetch_ms, pipeline_ms, title, js_report,
        })
    }

    pub fn info(&self) -> SessionInfo {
        SessionInfo {
            id: self.id.clone(), url: self.url.clone(), status: self.status.clone(),
            page_count: self.page_count, memory_bytes: self.memory_bytes,
            created_at_ms: self.created_at.elapsed().as_millis() as u64,
            last_active_ms: self.last_active.elapsed().as_millis() as u64,
            idle_secs: self.last_active.elapsed().as_secs(),
        }
    }
}

/// Thread-safe session manager — embeddable data structure, not a service.
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    limits: ResourceLimits,
}

impl Default for SessionManager {
    fn default() -> Self { Self::new(ResourceLimits::default()) }
}

impl SessionManager {
    pub fn new(limits: ResourceLimits) -> Self {
        SessionManager { sessions: Arc::new(RwLock::new(HashMap::new())), limits }
    }

    pub async fn create_session(&self) -> Result<String, String> {
        self.create_session_with_options(None, &[]).await
    }

    pub async fn create_session_with_options(&self, user_agent: Option<&str>, auth_profiles: &[String]) -> Result<String, String> {
        let mut sessions = self.sessions.write().await;
        if sessions.len() >= self.limits.max_sessions {
            return Err(format!("Maximum sessions ({}) reached", self.limits.max_sessions));
        }
        if self.limits.total_memory_budget > 0 {
            let total: usize = sessions.values().map(|s| s.memory_bytes).sum();
            if total >= self.limits.total_memory_budget {
                return Err(format!("Total memory budget ({} bytes) exceeded", self.limits.total_memory_budget));
            }
        }
        let id = Uuid::new_v4().to_string();
        sessions.insert(id.clone(), Session::new_with_options(id.clone(), user_agent, auth_profiles)?);
        info!(session_id = %id, "Session created");
        Ok(id)
    }

    /// Navigate a session. Takes it out of the map during navigation so other
    /// sessions can proceed concurrently without holding the write lock.
    pub async fn navigate(&self, session_id: &str, url: &str) -> Result<NavigateResult, String> {
        let mut session = {
            let mut sessions = self.sessions.write().await;
            sessions.remove(session_id).ok_or_else(|| format!("Session not found: {}", session_id))?
        };
        let result = session.navigate(url).await;
        { self.sessions.write().await.insert(session_id.to_string(), session); }
        result
    }

    pub async fn get_session_info(&self, session_id: &str) -> Option<SessionInfo> {
        self.sessions.read().await.get(session_id).map(|s| s.info())
    }

    pub async fn with_session<F, R>(&self, session_id: &str, f: F) -> Option<R>
    where F: FnOnce(&mut Session) -> R {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.last_active = Instant::now();
            Some(f(session))
        } else { None }
    }

    pub async fn with_session_ref<F, R>(&self, session_id: &str, f: F) -> Option<R>
    where F: FnOnce(&Session) -> R {
        self.sessions.read().await.get(session_id).map(f)
    }

    pub async fn close_session(&self, session_id: &str) -> bool {
        let removed = self.sessions.write().await.remove(session_id).is_some();
        if removed { info!(session_id = %session_id, "Session closed"); }
        removed
    }

    pub async fn list_sessions(&self) -> Vec<SessionInfo> {
        self.sessions.read().await.values().map(|s| s.info()).collect()
    }

    pub async fn session_count(&self) -> usize { self.sessions.read().await.len() }

    pub async fn session_exists(&self, session_id: &str) -> bool {
        self.sessions.read().await.contains_key(session_id)
    }

    pub async fn cleanup_idle(&self) -> usize {
        let mut sessions = self.sessions.write().await;
        let timeout = self.limits.idle_timeout;
        let before = sessions.len();
        sessions.retain(|id, s| {
            let idle = s.last_active.elapsed() >= timeout;
            if idle { info!(session_id = %id, idle_secs = s.last_active.elapsed().as_secs(), "Cleaning up idle session"); }
            !idle
        });
        before - sessions.len()
    }

    pub async fn total_memory_bytes(&self) -> usize {
        self.sessions.read().await.values().map(|s| s.memory_bytes).sum()
    }

    pub fn limits(&self) -> &ResourceLimits { &self.limits }
}

/// Fetch multiple URLs in parallel, each with its own HTTP client.
/// Results returned in same order as input URLs. Progress printed to stderr.
pub async fn fetch_parallel(
    urls: &[String], concurrency: usize, auth_profiles: &[String], pipeline_config: &PipelineConfig,
) -> Vec<Result<ParallelFetchResult, String>> {
    use futures_util::stream::{self, StreamExt};
    let concurrency = concurrency.min(MAX_CONCURRENCY).max(1);
    let total = urls.len();

    let indexed: Vec<(usize, Result<ParallelFetchResult, String>)> =
        stream::iter(urls.iter().enumerate())
            .map(|(idx, url)| {
                let url = url.clone();
                let profiles = auth_profiles.to_vec();
                let config = pipeline_config.clone();
                async move {
                    let result = fetch_single(&url, &profiles, &config).await;
                    match &result {
                        Ok(r) => eprintln!("  [{}/{}] {} -> {} ({} bytes, {}ms)", idx+1, total, url, r.status, r.html_bytes, r.total_ms),
                        Err(e) => eprintln!("  [{}/{}] {} -> ERROR: {}", idx+1, total, url, e),
                    }
                    (idx, result)
                }
            })
            .buffer_unordered(concurrency)
            .collect()
            .await;

    let mut results = Vec::with_capacity(urls.len());
    results.resize_with(urls.len(), || Err("not started".to_string()));
    for (idx, result) in indexed { results[idx] = result; }
    results
}

#[derive(Debug, Clone, Serialize)]
pub struct ParallelFetchResult {
    pub url: String, pub final_url: String, pub status: u16, pub title: String,
    pub html_bytes: usize, pub som_bytes: usize, pub element_count: usize,
    pub fetch_ms: u64, pub pipeline_ms: u64, pub total_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub js_report: Option<JsReportInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsReportInfo { pub total: usize, pub succeeded: usize, pub failed: usize }

async fn fetch_single(url: &str, auth_profiles: &[String], pipeline_config: &PipelineConfig) -> Result<ParallelFetchResult, String> {
    let start = Instant::now();
    let jar = Arc::new(Jar::default());
    for domain in auth_profiles {
        let _ = crate::auth::store::load_into_jar(domain, &jar);
    }
    let client = fetch::build_client_h1_fallback(None, jar, crate::network::tls::global()).map_err(|e| e.to_string())?;
    let fetch_result = fetch::fetch_url(&client, url, 30000).await.map_err(|e| e.to_string())?;
    let fetch_ms = fetch_result.load_ms;
    let html_bytes = fetch_result.html_bytes;
    let status = fetch_result.status;
    let final_url = fetch_result.url.clone();

    let page_result = crate::js::pipeline::process_page_async(&fetch_result.html, &final_url, pipeline_config, &client)
        .await.map_err(|e| e.to_string())?;

    Ok(ParallelFetchResult {
        url: url.to_string(), final_url, status,
        title: page_result.som.title.clone(),
        html_bytes, som_bytes: page_result.som.meta.som_bytes,
        element_count: page_result.som.meta.element_count,
        fetch_ms, pipeline_ms: (page_result.timing.total_us / 1000) as u64,
        total_ms: start.elapsed().as_millis() as u64,
        js_report: page_result.js_report.as_ref().map(|r| JsReportInfo {
            total: r.total, succeeded: r.succeeded, failed: r.failed,
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_close() {
        let m = SessionManager::default();
        let id = m.create_session().await.unwrap();
        assert!(m.session_exists(&id).await);
        assert_eq!(m.session_count().await, 1);
        assert!(m.close_session(&id).await);
        assert_eq!(m.session_count().await, 0);
    }

    #[tokio::test]
    async fn test_max_sessions() {
        let m = SessionManager::new(ResourceLimits { max_sessions: 2, ..Default::default() });
        let id1 = m.create_session().await.unwrap();
        let _id2 = m.create_session().await.unwrap();
        assert!(m.create_session().await.is_err());
        m.close_session(&id1).await;
        assert!(m.create_session().await.is_ok());
    }

    #[tokio::test]
    async fn test_list() {
        let m = SessionManager::default();
        let id1 = m.create_session().await.unwrap();
        let id2 = m.create_session().await.unwrap();
        let list = m.list_sessions().await;
        assert_eq!(list.len(), 2);
        let ids: Vec<&String> = list.iter().map(|s| &s.id).collect();
        assert!(ids.contains(&&id1) && ids.contains(&&id2));
    }

    #[tokio::test]
    async fn test_cleanup_idle() {
        let m = SessionManager::new(ResourceLimits { max_sessions: 10, idle_timeout: Duration::from_millis(50), ..Default::default() });
        m.create_session().await.unwrap();
        m.create_session().await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(m.cleanup_idle().await, 2);
        assert_eq!(m.session_count().await, 0);
    }

    #[tokio::test]
    async fn test_memory_budget() {
        let m = SessionManager::new(ResourceLimits { max_sessions: 10, total_memory_budget: 100, ..Default::default() });
        let id = m.create_session().await.unwrap();
        m.with_session(&id, |s| { s.memory_bytes = 200; }).await;
        assert!(m.create_session().await.is_err());
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", SessionStatus::Idle), "idle");
        assert_eq!(format!("{}", SessionStatus::Ready), "ready");
    }
}
