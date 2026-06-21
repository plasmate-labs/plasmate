#![allow(dead_code)]
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
}

/// State for a single MCP browser session.
pub struct SessionState {
    /// The CDP target (holds page state, HTML, SOM, etc.)
    pub target: CdpTarget,
    /// When this session was created.
    pub created_at: Instant,
    /// When this session was last accessed.
    pub last_accessed: Instant,
}

impl SessionState {
    /// Create a new session state with a fresh CDP target.
    pub fn new(target: CdpTarget) -> Self {
        let now = Instant::now();
        SessionState {
            target,
            created_at: now,
            last_accessed: now,
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
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager {
    /// Create a new session manager.
    pub fn new() -> Self {
        SessionManager {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
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

    /// Get a read-only reference to a session's target.
    pub async fn with_session_ref<F, R>(&self, session_id: &str, f: F) -> Option<R>
    where
        F: FnOnce(&SessionState) -> R,
    {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).map(f)
    }

    /// Close a session and free its resources.
    pub async fn close_session(&self, session_id: &str) -> bool {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id).is_some()
    }

    /// Check if a session exists.
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
                let som_meta = session.target.current_som.as_ref().map(|som| &som.meta);
                SessionSummary {
                    session_id: session_id.clone(),
                    url: session.target.current_url.clone(),
                    title: session
                        .target
                        .current_som
                        .as_ref()
                        .map(|som| som.title.clone()),
                    age_ms: now.duration_since(session.created_at).as_millis(),
                    idle_ms: now.duration_since(session.last_accessed).as_millis(),
                    has_page: session.target.current_som.is_some(),
                    has_structured_data: session.target.current_structured_data.is_some(),
                    has_effective_html: session.target.effective_html.is_some(),
                    node_count: session.target.node_map.len(),
                    html_bytes: session.target.current_html.as_ref().map(String::len),
                    effective_html_bytes: session.target.effective_html.as_ref().map(String::len),
                    som_bytes: som_meta.map(|meta| meta.som_bytes),
                    element_count: som_meta.map(|meta| meta.element_count),
                    interactive_count: som_meta.map(|meta| meta.interactive_count),
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
        assert!(manager.close_session(&id).await);
    }
}
