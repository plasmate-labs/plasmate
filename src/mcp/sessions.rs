//! MCP session management for stateful browser tools.
//!
//! Manages browser sessions with V8 runtime state. Each session holds:
//! - A CdpTarget for CDP operations
//! - The effective HTML after JS execution
//! - Created timestamp for idle timeout

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use serde::Serialize;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::cdp::session::CdpTarget;
use crate::js::worker::JsWorkerOptions;
use crate::mcp::trace::{
    self, PendingTraceEvent, ReplayRequest, TraceAttempt, TraceExport, TraceLog, TraceStatus,
};

/// Maximum number of concurrent sessions.
pub const MAX_SESSIONS: usize = 10;

#[derive(Debug, Clone, Serialize)]
pub struct SessionSnapshot {
    pub active_sessions: usize,
    pub max_sessions: usize,
    pub available_sessions: usize,
    pub oldest_session_age_ms: u128,
    pub longest_idle_ms: u128,
    pub sessions: Vec<SessionSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub url: Option<String>,
    pub title: Option<String>,
    pub age_ms: u128,
    pub idle_ms: u128,
    pub has_page: bool,
    pub has_structured_data: bool,
    pub has_effective_html: bool,
    pub node_count: usize,
    pub html_bytes: Option<usize>,
    pub effective_html_bytes: Option<usize>,
    pub som_bytes: Option<usize>,
    pub element_count: Option<usize>,
    pub interactive_count: Option<usize>,
    pub disabled_count: Option<usize>,
    pub readonly_count: Option<usize>,
}

/// State for a single MCP browser session.
pub struct SessionState {
    /// The CDP target (holds page state, HTML, SOM, etc.)
    pub target: CdpTarget,
    /// When this session was created.
    pub created_at: Instant,
    /// When this session was last accessed.
    pub last_accessed: Instant,
    /// Optional privacy-safe action tracing. Disabled by default.
    pub trace: TraceLog,
}

impl SessionState {
    /// Create a new session state with a fresh CDP target.
    pub fn new(target: CdpTarget) -> Self {
        let now = Instant::now();
        SessionState {
            target,
            created_at: now,
            last_accessed: now,
            trace: TraceLog::new(),
        }
    }

    /// Update the last accessed timestamp.
    pub fn touch(&mut self) {
        self.last_accessed = Instant::now();
    }
}

/// Session manager for MCP browser sessions.
///
/// Thread-safe with interior mutability via RwLock.
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, SessionState>>>,
    js_worker_options: Arc<JsWorkerOptions>,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager {
    /// Create a new session manager.
    pub fn new() -> Self {
        Self::with_worker_options(JsWorkerOptions::default())
    }

    /// Construct a manager with an explicit stateful-evaluation worker policy.
    ///
    /// Production callers use [`Self::new`]. The explicit constructor keeps
    /// crash, timeout, and output-limit behavior directly testable without
    /// process-global environment overrides.
    pub fn with_worker_options(js_worker_options: JsWorkerOptions) -> Self {
        SessionManager {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            js_worker_options: Arc::new(js_worker_options),
        }
    }

    pub(crate) fn js_worker_options(&self) -> JsWorkerOptions {
        self.js_worker_options.as_ref().clone()
    }

    /// Generate a new unique session ID.
    fn generate_session_id() -> String {
        Uuid::new_v4().to_string()
    }

    /// Create a new session. Returns the session ID or an error if max sessions reached.
    pub async fn create_session(&self) -> Result<String, String> {
        let mut sessions = self.sessions.write().await;

        if sessions.len() >= MAX_SESSIONS {
            return Err(format!(
                "Maximum sessions ({}) reached. Close a session first.",
                MAX_SESSIONS
            ));
        }

        let session_id = Self::generate_session_id();
        let target = CdpTarget::new()?;

        sessions.insert(session_id.clone(), SessionState::new(target));

        Ok(session_id)
    }

    /// Get a mutable reference to a session's target.
    /// Returns None if session doesn't exist.
    pub async fn with_session<F, R>(&self, session_id: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut SessionState) -> R,
    {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.touch();
            Some(f(session))
        } else {
            None
        }
    }

    /// Close a session and free its resources.
    pub async fn close_session(&self, session_id: &str) -> bool {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id).is_some()
    }

    /// Enable tracing for a newly opened session. Tracing is opt-in and
    /// remains scoped to this in-memory session.
    pub async fn enable_trace(&self, session_id: &str) -> bool {
        self.with_session(session_id, |session| session.trace.set_enabled(true))
            .await
            .is_some()
    }

    /// Capture the pre-action state needed for a trace event. Nothing is
    /// captured when tracing is disabled or the action is not traceable.
    pub async fn prepare_trace(
        &self,
        action: &str,
        arguments: &serde_json::Value,
    ) -> Option<TraceAttempt> {
        if !trace::is_traceable_action(action) {
            return None;
        }
        let session_id = trace::session_id_from_arguments(arguments)?;
        self.with_session(session_id, |session| {
            if !session.trace.enabled() {
                return None;
            }
            Some(TraceAttempt {
                session_id: session_id.to_string(),
                action: action.to_string(),
                parameters: session.trace.sanitized_parameters(action, arguments),
                target: session.trace.replay_target(
                    session.target.current_som.as_ref(),
                    arguments
                        .get("element_id")
                        .and_then(serde_json::Value::as_str),
                ),
                before: session.trace.page_state(
                    session.target.current_url.as_deref(),
                    session.target.current_som.as_ref(),
                ),
                detached_log: session.trace.clone(),
            })
        })
        .await
        .flatten()
    }

    /// Complete a trace attempt. A close event is returned as a final export
    /// because the owning session is removed by the action.
    pub async fn finish_trace(
        &self,
        attempt: TraceAttempt,
        result: &serde_json::Value,
        duration: std::time::Duration,
    ) -> Option<TraceExport> {
        let completed = self
            .with_session(&attempt.session_id, |session| {
                let after = session.trace.page_state(
                    session.target.current_url.as_deref(),
                    session.target.current_som.as_ref(),
                );
                session.trace.append(
                    PendingTraceEvent {
                        action: attempt.action.clone(),
                        parameters: attempt.parameters.clone(),
                        target: attempt.target.clone(),
                        before: attempt.before.clone(),
                        after,
                    },
                    result,
                    duration,
                );
            })
            .await;
        if completed.is_some() {
            return None;
        }

        if attempt.action == "close_page" {
            let mut log = attempt.detached_log;
            let after = log.closed_page_state();
            log.append(
                PendingTraceEvent {
                    action: attempt.action,
                    parameters: attempt.parameters,
                    target: attempt.target,
                    before: attempt.before,
                    after,
                },
                result,
                duration,
            );
            return Some(log.export(&attempt.session_id));
        }
        None
    }

    pub async fn record_open_trace(
        &self,
        session_id: &str,
        arguments: &serde_json::Value,
        result: &serde_json::Value,
        duration: std::time::Duration,
    ) {
        let _ = self
            .with_session(session_id, |session| {
                if !session.trace.enabled() {
                    return;
                }
                let after = session.trace.page_state(
                    session.target.current_url.as_deref(),
                    session.target.current_som.as_ref(),
                );
                let parameters = session.trace.sanitized_parameters("open_page", arguments);
                let before = session.trace.closed_page_state();
                session.trace.append(
                    PendingTraceEvent {
                        action: "open_page".to_string(),
                        parameters,
                        target: None,
                        before,
                        after,
                    },
                    result,
                    duration,
                );
            })
            .await;
    }

    pub async fn trace_status(&self, session_id: &str) -> Option<TraceStatus> {
        self.with_session(session_id, |session| session.trace.status())
            .await
    }

    pub async fn trace_export(&self, session_id: &str) -> Option<TraceExport> {
        self.with_session(session_id, |session| session.trace.export(session_id))
            .await
    }

    pub async fn clear_trace(&self, session_id: &str) -> Option<(u64, TraceStatus)> {
        self.with_session(session_id, |session| {
            let cleared = session.trace.clear();
            (cleared, session.trace.status())
        })
        .await
    }

    pub async fn validate_trace_replay(
        &self,
        request: &ReplayRequest,
    ) -> Option<serde_json::Value> {
        self.with_session(&request.session_id, |session| {
            let current = session.trace.page_state(
                session.target.current_url.as_deref(),
                session.target.current_som.as_ref(),
            );
            trace::validate_replay(
                &session.trace,
                &request.session_id,
                request,
                current,
                session.target.current_som.as_ref(),
            )
        })
        .await
    }

    /// Check if a session exists.
    #[cfg(test)]
    pub async fn session_exists(&self, session_id: &str) -> bool {
        let sessions = self.sessions.read().await;
        sessions.contains_key(session_id)
    }

    /// Get the number of active sessions.
    #[allow(dead_code)]
    pub async fn session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }

    /// Return a lightweight inventory for MCP status output.
    pub async fn snapshot(&self) -> SessionSnapshot {
        let sessions = self.sessions.read().await;
        let now = Instant::now();
        let oldest_session_age_ms = sessions
            .values()
            .map(|session| now.duration_since(session.created_at).as_millis())
            .max()
            .unwrap_or(0);
        let longest_idle_ms = sessions
            .values()
            .map(|session| now.duration_since(session.last_accessed).as_millis())
            .max()
            .unwrap_or(0);
        let mut session_summaries: Vec<SessionSummary> = sessions
            .iter()
            .map(|(session_id, session)| {
                let som = session.target.current_som.as_ref();
                let som_meta = som.map(|som| &som.meta);
                let blocked = som.map(blocked_interactive_counts);
                SessionSummary {
                    session_id: session_id.clone(),
                    url: session.target.current_url.clone(),
                    title: som.map(|som| som.title.clone()),
                    age_ms: now.duration_since(session.created_at).as_millis(),
                    idle_ms: now.duration_since(session.last_accessed).as_millis(),
                    has_page: som.is_some(),
                    has_structured_data: session.target.current_structured_data.is_some(),
                    has_effective_html: session.target.effective_html.is_some(),
                    node_count: session.target.node_map.len(),
                    html_bytes: session.target.current_html.as_ref().map(String::len),
                    effective_html_bytes: session.target.effective_html.as_ref().map(String::len),
                    som_bytes: som_meta.map(|meta| meta.som_bytes),
                    element_count: som_meta.map(|meta| meta.element_count),
                    interactive_count: som_meta.map(|meta| meta.interactive_count),
                    disabled_count: blocked.map(|(disabled, _)| disabled),
                    readonly_count: blocked.map(|(_, readonly)| readonly),
                }
            })
            .collect();

        session_summaries.sort_by(|a, b| a.session_id.cmp(&b.session_id));

        SessionSnapshot {
            active_sessions: sessions.len(),
            max_sessions: MAX_SESSIONS,
            available_sessions: MAX_SESSIONS.saturating_sub(sessions.len()),
            oldest_session_age_ms,
            longest_idle_ms,
            sessions: session_summaries,
        }
    }
}

fn blocked_interactive_counts(som: &crate::som::types::Som) -> (usize, usize) {
    let mut disabled = 0;
    let mut readonly = 0;
    for region in &som.regions {
        count_blocked_interactives(&region.elements, &mut disabled, &mut readonly);
    }
    (disabled, readonly)
}

fn count_blocked_interactives(
    elements: &[crate::som::types::Element],
    disabled: &mut usize,
    readonly: &mut usize,
) {
    for element in elements {
        if element.role.is_interactive() {
            if let Some(attrs) = &element.attrs {
                if attr_flag_true(attrs, "disabled") {
                    *disabled += 1;
                }
                if attr_flag_true(attrs, "readonly") {
                    *readonly += 1;
                }
            }
        }
        if let Some(children) = &element.children {
            count_blocked_interactives(children, disabled, readonly);
        }
        if let Some(shadow) = &element.shadow {
            count_blocked_interactives(&shadow.elements, disabled, readonly);
        }
    }
}

fn attr_flag_true(attrs: &serde_json::Value, key: &str) -> bool {
    match attrs.get(key) {
        Some(serde_json::Value::Bool(true)) => true,
        Some(serde_json::Value::String(value)) => {
            value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case(key)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_close_session() {
        let manager = SessionManager::new();

        let session_id = manager.create_session().await.unwrap();
        assert!(manager.session_exists(&session_id).await);

        let closed = manager.close_session(&session_id).await;
        assert!(closed);
        assert!(!manager.session_exists(&session_id).await);
    }

    #[tokio::test]
    async fn test_max_sessions() {
        let manager = SessionManager::new();

        // Create MAX_SESSIONS sessions
        let mut session_ids = Vec::new();
        for _ in 0..MAX_SESSIONS {
            let id = manager.create_session().await.unwrap();
            session_ids.push(id);
        }

        // Next one should fail
        let result = manager.create_session().await;
        assert!(result.is_err());

        // Close one and try again
        manager.close_session(&session_ids[0]).await;
        let result = manager.create_session().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_snapshot_reports_session_inventory() {
        let manager = SessionManager::new();
        let id = manager.create_session().await.unwrap();
        manager
            .with_session(&id, |session| {
                session.target.current_html = Some("<html></html>".to_string());
                session.target.effective_html = Some("<html><body>ready</body></html>".to_string());
            })
            .await
            .unwrap();

        let snapshot = manager.snapshot().await;

        assert_eq!(snapshot.active_sessions, 1);
        assert_eq!(snapshot.max_sessions, MAX_SESSIONS);
        assert_eq!(snapshot.available_sessions, MAX_SESSIONS - 1);
        assert_eq!(snapshot.sessions.len(), 1);
        assert_eq!(snapshot.sessions[0].session_id, id);
        assert!(!snapshot.sessions[0].has_page);
        assert!(snapshot.sessions[0].has_effective_html);
        assert_eq!(snapshot.sessions[0].html_bytes, Some(13));
        assert_eq!(snapshot.sessions[0].effective_html_bytes, Some(31));
        assert_eq!(snapshot.sessions[0].disabled_count, None);
        assert_eq!(snapshot.sessions[0].readonly_count, None);
        assert!(manager.close_session(&id).await);
    }

    #[tokio::test]
    async fn snapshot_counts_disabled_and_readonly_interactives() {
        let manager = SessionManager::new();
        let id = manager.create_session().await.unwrap();
        let html = "<html><head><title>Locked fields</title></head><body><main><input id='coupon' disabled value='SAVE'><textarea id='notes' readonly>Draft</textarea><button>Ok</button></main></body></html>";
        manager
            .with_session(&id, |session| {
                session.target.current_url = Some("https://example.test/locked".to_string());
                session.target.current_html = Some(html.to_string());
                session.target.effective_html = Some(html.to_string());
                session.target.current_som = Some(
                    crate::som::compiler::compile(html, "https://example.test/locked").unwrap(),
                );
            })
            .await
            .unwrap();

        let snapshot = manager.snapshot().await;
        assert_eq!(snapshot.sessions[0].interactive_count, Some(3));
        assert_eq!(snapshot.sessions[0].disabled_count, Some(1));
        assert_eq!(snapshot.sessions[0].readonly_count, Some(1));
        assert!(manager.close_session(&id).await);
    }

    #[tokio::test]
    async fn tracing_is_disabled_by_default_and_failed_action_is_appended_once() {
        let manager = SessionManager::new();
        let id = manager.create_session().await.unwrap();
        let arguments = serde_json::json!({"session_id": id, "element_id": "missing"});
        assert!(manager.prepare_trace("click", &arguments).await.is_none());

        assert!(manager.enable_trace(&id).await);
        let attempt = manager.prepare_trace("click", &arguments).await.unwrap();
        let error = serde_json::json!({
            "isError": true,
            "content": [{"type":"text", "text":"Element not found: missing"}]
        });
        assert!(manager
            .finish_trace(attempt, &error, std::time::Duration::from_millis(1))
            .await
            .is_none());
        let export = manager.trace_export(&id).await.unwrap();
        assert_eq!(export.retained_events, 1);
        assert_eq!(export.events[0].outcome, "error");
        assert_eq!(export.events[0].error_class, Some("not_found"));
    }

    #[tokio::test]
    async fn failed_close_returns_bounded_final_trace_after_session_removal() {
        let manager = SessionManager::new();
        let id = manager.create_session().await.unwrap();
        assert!(manager.enable_trace(&id).await);
        let arguments = serde_json::json!({"session_id": id});
        let attempt = manager
            .prepare_trace("close_page", &arguments)
            .await
            .unwrap();
        assert!(manager.close_session(&id).await);
        let error = serde_json::json!({
            "isError": true,
            "content": [{"type":"text", "text":"Synthetic close failure with secret detail"}]
        });
        let export = manager
            .finish_trace(attempt, &error, std::time::Duration::ZERO)
            .await
            .unwrap();
        assert_eq!(export.retained_events, 1);
        assert_eq!(export.events[0].action, "close_page");
        assert_eq!(export.events[0].outcome, "error");
        let encoded = serde_json::to_vec(&export).unwrap();
        assert!(encoded.len() <= crate::mcp::trace::MAX_TRACE_EXPORT_BYTES);
        assert!(!String::from_utf8_lossy(&encoded).contains("secret detail"));
    }
}
