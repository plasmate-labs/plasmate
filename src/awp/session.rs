use std::sync::Arc;
use std::time::Instant;

use reqwest::cookie::Jar;
use reqwest::Client;
use tokio::sync::Mutex;

use crate::js::pipeline::PipelineConfig;
use crate::network::fetch;
use crate::network::intercept::{
    InterceptAction, NetworkInterceptor, ResourceType as InterceptResourceType,
};
use crate::network::tls::TlsConfig;
use crate::plugin::PluginManager;
use crate::som::metadata::StructuredData;
use crate::som::types::Som;

/// An in-memory browsing session with full pipeline support.
pub struct Session {
    pub id: String,
    pub user_agent: String,
    pub locale: String,
    pub timeout_ms: u64,
    pub cookie_jar: Arc<Jar>,
    pub client: Client,
    /// Current page state.
    pub current_som: Option<Som>,
    pub current_html: Option<String>,
    pub current_url: Option<String>,
    pub current_structured_data: Option<StructuredData>,
    /// Current vertical scroll position.
    pub scroll_y: Option<i64>,
    /// Navigation history (URLs visited in this session).
    pub history: Vec<HistoryEntry>,
    /// Pipeline configuration for this session.
    pub pipeline_config: PipelineConfig,
    /// Network interception.
    pub interceptor: NetworkInterceptor,
    /// Session creation time.
    pub created_at: Instant,
    /// Number of pages navigated.
    pub page_count: u64,
}

/// A navigation history entry.
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub url: String,
    pub title: String,
    pub timestamp_ms: u64,
    pub html_bytes: usize,
    pub som_bytes: usize,
}

const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";

impl Session {
    pub fn new(
        id: String,
        user_agent: Option<String>,
        locale: Option<String>,
        timeout_ms: Option<u64>,
        tls_config: Option<TlsConfig>,
    ) -> Result<Self, String> {
        Self::new_with_profiles(
            id,
            user_agent,
            locale,
            timeout_ms,
            crate::auth::config::profiles(),
            tls_config,
        )
    }

    pub fn new_with_profiles(
        id: String,
        user_agent: Option<String>,
        locale: Option<String>,
        timeout_ms: Option<u64>,
        auth_profiles: &[String],
        tls_config: Option<TlsConfig>,
    ) -> Result<Self, String> {
        let ua = user_agent.unwrap_or_else(|| DEFAULT_USER_AGENT.to_string());
        let locale = locale.unwrap_or_else(|| "en-US".to_string());
        let timeout = timeout_ms.unwrap_or(30000);
        let jar = Arc::new(Jar::default());

        // Load auth cookies from profiles
        for domain in auth_profiles {
            if let Err(e) = crate::auth::store::load_into_jar(domain, &jar) {
                tracing::warn!(domain = %domain, error = %e, "Failed to load auth profile");
            }
        }

        // Use per-session TLS config, fall back to global, fall back to none
        let effective_tls = tls_config
            .as_ref()
            .or_else(|| crate::network::tls::global());
        let client = fetch::build_client_h1_fallback(Some(&ua), jar.clone(), effective_tls)
            .map_err(|e| e.to_string())?;

        Ok(Session {
            id,
            user_agent: ua,
            locale,
            timeout_ms: timeout,
            cookie_jar: jar,
            client,
            current_som: None,
            current_html: None,
            current_url: None,
            current_structured_data: None,
            scroll_y: None,
            history: Vec::new(),
            pipeline_config: PipelineConfig {
                execute_js: true,
                fetch_external_scripts: true,
                ..Default::default()
            },
            interceptor: NetworkInterceptor::new(),
            created_at: Instant::now(),
            page_count: 0,
        })
    }

    /// Navigate to a URL using the full async pipeline.
    pub async fn navigate(&mut self, url: &str) -> Result<NavigateResult, String> {
        let start = Instant::now();

        // Check interception rules before fetch
        let (action, _intercept_info) =
            self.interceptor
                .check_request(url, &InterceptResourceType::Document, true);

        let mut fetch_result = match action {
            InterceptAction::Fulfill(params) => NetworkInterceptor::fulfill_request(&params, url),
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
        self.interceptor
            .check_response(url, &InterceptResourceType::Document, &mut fetch_result);

        let fetch_ms = fetch_result.load_ms;
        let html_bytes = fetch_result.html_bytes;
        let status = fetch_result.status;
        let final_url = fetch_result.url.clone();
        let content_type = fetch_result.content_type.clone();

        // Run full pipeline: extract scripts -> fetch external -> V8 execute -> SOM compile
        let page_result = crate::js::pipeline::process_page_async(
            &fetch_result.html,
            &final_url,
            &self.pipeline_config,
            &self.client,
        )
        .await
        .map_err(|e| e.to_string())?;

        let pipeline_ms = page_result.timing.total_us / 1000;
        let som_bytes = page_result.som.meta.som_bytes;
        let title = page_result.som.title.clone();
        let structured_data = page_result.som.structured_data.clone();

        let js_report = page_result.js_report.as_ref().map(|r| JsReportSummary {
            total: r.total,
            succeeded: r.succeeded,
            failed: r.failed,
        });

        // Update session state
        self.current_url = Some(final_url.clone());
        self.current_html = Some(fetch_result.html);
        self.current_structured_data = structured_data;
        self.current_som = Some(page_result.som);
        self.page_count += 1;

        // Add to history
        self.history.push(HistoryEntry {
            url: final_url.clone(),
            title: title.clone(),
            timestamp_ms: start.elapsed().as_millis() as u64,
            html_bytes,
            som_bytes,
        });

        Ok(NavigateResult {
            url: final_url,
            status,
            content_type,
            html_bytes,
            som_bytes,
            fetch_ms,
            pipeline_ms: pipeline_ms as u64,
            title,
            js_report,
        })
    }

    /// Navigate with optional Wasm plugin hooks at each pipeline stage.
    pub async fn navigate_with_plugins(
        &mut self,
        url: &str,
        plugins: &Option<Arc<Mutex<PluginManager>>>,
    ) -> Result<NavigateResult, String> {
        // If no plugins, delegate to the plain navigate.
        let pm = match plugins {
            Some(pm) => pm,
            None => return self.navigate(url).await,
        };

        let start = Instant::now();

        let fetch_result = fetch::fetch_url(&self.client, url, self.timeout_ms)
            .await
            .map_err(|e| e.to_string())?;

        let fetch_ms = fetch_result.load_ms;
        let html_bytes = fetch_result.html_bytes;
        let status = fetch_result.status;
        let final_url = fetch_result.url.clone();
        let content_type = fetch_result.content_type.clone();

        let page_result = {
            let mut guard = pm.lock().await;
            crate::js::pipeline::process_page_async_with_plugins(
                &fetch_result.html,
                &final_url,
                &self.pipeline_config,
                &self.client,
                &mut guard,
            )
            .await
            .map_err(|e| e.to_string())?
        };

        let pipeline_ms = page_result.timing.total_us / 1000;
        let som_bytes = page_result.som.meta.som_bytes;
        let title = page_result.som.title.clone();
        let structured_data = page_result.som.structured_data.clone();

        let js_report = page_result.js_report.as_ref().map(|r| JsReportSummary {
            total: r.total,
            succeeded: r.succeeded,
            failed: r.failed,
        });

        self.current_url = Some(final_url.clone());
        self.current_html = Some(fetch_result.html);
        self.current_structured_data = structured_data;
        self.current_som = Some(page_result.som);
        self.page_count += 1;

        self.history.push(HistoryEntry {
            url: final_url.clone(),
            title: title.clone(),
            timestamp_ms: start.elapsed().as_millis() as u64,
            html_bytes,
            som_bytes,
        });

        Ok(NavigateResult {
            url: final_url,
            status,
            content_type,
            html_bytes,
            som_bytes,
            fetch_ms,
            pipeline_ms: pipeline_ms as u64,
            title,
            js_report,
        })
    }

    /// Go back in history.
    pub fn can_go_back(&self) -> bool {
        self.history.len() > 1
    }

    /// Get session info.
    pub fn info(&self) -> serde_json::Value {
        serde_json::json!({
            "session_id": self.id,
            "current_url": self.current_url,
            "page_count": self.page_count,
            "history_length": self.history.len(),
            "uptime_ms": self.created_at.elapsed().as_millis() as u64,
        })
    }
}

/// Result of a navigation.
pub struct NavigateResult {
    pub url: String,
    pub status: u16,
    pub content_type: String,
    pub html_bytes: usize,
    pub som_bytes: usize,
    pub fetch_ms: u64,
    pub pipeline_ms: u64,
    pub title: String,
    pub js_report: Option<JsReportSummary>,
}

#[derive(Debug, Clone)]
pub struct JsReportSummary {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
}
