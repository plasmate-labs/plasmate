use std::sync::Arc;
use std::time::Instant;

use reqwest::cookie::Jar;
use reqwest::Client;

use crate::js::pipeline::PipelineConfig;
use crate::network::fetch;
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
    /// Navigation history (URLs visited in this session).
    pub history: Vec<HistoryEntry>,
    /// Pipeline configuration for this session.
    pub pipeline_config: PipelineConfig,
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
    ) -> Result<Self, String> {
        Self::new_with_profiles(id, user_agent, locale, timeout_ms, crate::auth::config::profiles())
    }

    pub fn new_with_profiles(
        id: String,
        user_agent: Option<String>,
        locale: Option<String>,
        timeout_ms: Option<u64>,
        auth_profiles: &[String],
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

        let client =
            fetch::build_client_h1_fallback(Some(&ua), jar.clone()).map_err(|e| e.to_string())?;

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
            history: Vec::new(),
            pipeline_config: PipelineConfig {
                execute_js: true,
                fetch_external_scripts: true,
                ..Default::default()
            },
            created_at: Instant::now(),
            page_count: 0,
        })
    }

    /// Navigate to a URL using the full async pipeline.
    pub async fn navigate(&mut self, url: &str) -> Result<NavigateResult, String> {
        let start = Instant::now();

        // Fetch HTML
        let fetch_result = fetch::fetch_url(&self.client, url, self.timeout_ms)
            .await
            .map_err(|e| e.to_string())?;

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
