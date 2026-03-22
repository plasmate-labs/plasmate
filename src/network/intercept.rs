//! Network request interception.
//!
//! Provides URL pattern matching, request modification, response mocking,
//! and request blocking. Used by both CDP (Fetch domain) and AWP.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use super::fetch::{FetchError, FetchResult};

/// Counter for generating unique request IDs.
static REQUEST_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

fn next_request_id() -> String {
    format!(
        "intercept.{}",
        REQUEST_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
    )
}

/// Resource types for pattern matching (CDP-compatible).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    Document,
    Stylesheet,
    Image,
    Media,
    Font,
    Script,
    TextTrack,
    XHR,
    Fetch,
    Prefetch,
    EventSource,
    WebSocket,
    Manifest,
    SignedExchange,
    Ping,
    CSPViolationReport,
    Preflight,
    Other,
}

impl ResourceType {
    pub fn from_cdp_str(s: &str) -> Self {
        match s {
            "Document" => Self::Document,
            "Stylesheet" => Self::Stylesheet,
            "Image" => Self::Image,
            "Media" => Self::Media,
            "Font" => Self::Font,
            "Script" => Self::Script,
            "TextTrack" => Self::TextTrack,
            "XHR" => Self::XHR,
            "Fetch" => Self::Fetch,
            "Prefetch" => Self::Prefetch,
            "EventSource" => Self::EventSource,
            "WebSocket" => Self::WebSocket,
            "Manifest" => Self::Manifest,
            "SignedExchange" => Self::SignedExchange,
            "Ping" => Self::Ping,
            "CSPViolationReport" => Self::CSPViolationReport,
            "Preflight" => Self::Preflight,
            _ => Self::Other,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Document => "Document",
            Self::Stylesheet => "Stylesheet",
            Self::Image => "Image",
            Self::Media => "Media",
            Self::Font => "Font",
            Self::Script => "Script",
            Self::TextTrack => "TextTrack",
            Self::XHR => "XHR",
            Self::Fetch => "Fetch",
            Self::Prefetch => "Prefetch",
            Self::EventSource => "EventSource",
            Self::WebSocket => "WebSocket",
            Self::Manifest => "Manifest",
            Self::SignedExchange => "SignedExchange",
            Self::Ping => "Ping",
            Self::CSPViolationReport => "CSPViolationReport",
            Self::Preflight => "Preflight",
            Self::Other => "Other",
        }
    }
}

/// When to intercept: before the request or after the response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestStage {
    Request,
    Response,
}

/// Error reasons for failing a request (CDP Fetch.failRequest compatible).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorReason {
    Failed,
    Aborted,
    TimedOut,
    AccessDenied,
    ConnectionClosed,
    ConnectionReset,
    ConnectionRefused,
    ConnectionAborted,
    ConnectionFailed,
    NameNotResolved,
    InternetDisconnected,
    AddressUnreachable,
    BlockedByClient,
    BlockedByResponse,
}

impl ErrorReason {
    pub fn from_cdp_str(s: &str) -> Self {
        match s {
            "Failed" => Self::Failed,
            "Aborted" => Self::Aborted,
            "TimedOut" => Self::TimedOut,
            "AccessDenied" => Self::AccessDenied,
            "ConnectionClosed" => Self::ConnectionClosed,
            "ConnectionReset" => Self::ConnectionReset,
            "ConnectionRefused" => Self::ConnectionRefused,
            "ConnectionAborted" => Self::ConnectionAborted,
            "ConnectionFailed" => Self::ConnectionFailed,
            "NameNotResolved" => Self::NameNotResolved,
            "InternetDisconnected" => Self::InternetDisconnected,
            "AddressUnreachable" => Self::AddressUnreachable,
            "BlockedByClient" => Self::BlockedByClient,
            "BlockedByResponse" => Self::BlockedByResponse,
            _ => Self::Failed,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Failed => "Failed",
            Self::Aborted => "Aborted",
            Self::TimedOut => "TimedOut",
            Self::AccessDenied => "AccessDenied",
            Self::ConnectionClosed => "ConnectionClosed",
            Self::ConnectionReset => "ConnectionReset",
            Self::ConnectionRefused => "ConnectionRefused",
            Self::ConnectionAborted => "ConnectionAborted",
            Self::ConnectionFailed => "ConnectionFailed",
            Self::NameNotResolved => "NameNotResolved",
            Self::InternetDisconnected => "InternetDisconnected",
            Self::AddressUnreachable => "AddressUnreachable",
            Self::BlockedByClient => "BlockedByClient",
            Self::BlockedByResponse => "BlockedByResponse",
        }
    }
}

/// A URL/resource-type pattern for matching requests.
#[derive(Debug, Clone)]
pub struct RequestPattern {
    /// Glob-style URL pattern (e.g., `"*://api.example.com/*"`, `"*.js"`).
    /// `None` matches all URLs.
    pub url_pattern: Option<String>,
    /// Only match specific resource types. `None` matches all types.
    pub resource_type: Option<ResourceType>,
    /// Whether to intercept at request or response stage.
    pub request_stage: RequestStage,
}

/// Overrides to apply when continuing a request.
#[derive(Debug, Clone, Default)]
pub struct RequestOverrides {
    pub url: Option<String>,
    pub method: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub post_data: Option<String>,
}

/// Parameters for fulfilling a request with a mock response.
#[derive(Debug, Clone)]
pub struct FulfillParams {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

/// What the interceptor decides for a matched request.
#[derive(Debug, Clone)]
pub enum InterceptAction {
    /// Let the request proceed, optionally with modifications.
    Continue(Option<RequestOverrides>),
    /// Fulfill the request with a synthetic response (no network).
    Fulfill(FulfillParams),
    /// Fail the request with an error reason.
    Fail(ErrorReason),
}

/// An interception rule: pattern + action.
#[derive(Debug, Clone)]
pub struct InterceptRule {
    pub pattern: RequestPattern,
    pub action: InterceptAction,
}

/// A request that was intercepted (for event reporting).
#[derive(Debug, Clone, Serialize)]
pub struct InterceptedRequestInfo {
    pub request_id: String,
    pub url: String,
    pub method: String,
    pub resource_type: String,
    pub is_navigation: bool,
}

/// Overrides to apply to a response before it reaches the pipeline.
#[derive(Debug, Clone, Default)]
pub struct ResponseOverrides {
    pub status: Option<u16>,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
}

/// A response interception rule.
#[derive(Debug, Clone)]
pub struct ResponseRule {
    pub pattern: RequestPattern,
    pub overrides: ResponseOverrides,
}

/// Network interceptor that manages patterns and rules.
///
/// Interception flow:
/// 1. `enable()` registers URL patterns that trigger interception.
/// 2. `add_rule()` registers actions (continue/fulfill/fail) for matching patterns.
/// 3. During fetch, `check_request()` finds the first matching rule and returns
///    the action to apply. If no rule matches, the request continues normally.
/// 4. After fetch, `check_response()` can modify the response body/status.
pub struct NetworkInterceptor {
    enabled: bool,
    /// Patterns that trigger interception (from Fetch.enable / network.setInterception).
    patterns: Vec<RequestPattern>,
    /// Rules that define what to do with intercepted requests.
    rules: Vec<InterceptRule>,
    /// Rules for modifying responses.
    response_rules: Vec<ResponseRule>,
    /// Log of intercepted requests (bounded ring buffer).
    intercepted_log: Vec<InterceptedRequestInfo>,
    /// Max entries in the log.
    max_log_entries: usize,
}

impl NetworkInterceptor {
    pub fn new() -> Self {
        NetworkInterceptor {
            enabled: false,
            patterns: Vec::new(),
            rules: Vec::new(),
            response_rules: Vec::new(),
            intercepted_log: Vec::new(),
            max_log_entries: 1000,
        }
    }

    /// Enable interception with the given patterns.
    pub fn enable(&mut self, patterns: Vec<RequestPattern>) {
        self.enabled = true;
        self.patterns = patterns;
    }

    /// Disable interception and clear all patterns/rules.
    pub fn disable(&mut self) {
        self.enabled = false;
        self.patterns.clear();
        self.rules.clear();
        self.response_rules.clear();
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Add an interception rule. First matching rule wins.
    pub fn add_rule(&mut self, rule: InterceptRule) {
        self.rules.push(rule);
    }

    /// Add a response modification rule.
    pub fn add_response_rule(&mut self, rule: ResponseRule) {
        self.response_rules.push(rule);
    }

    /// Remove all rules whose URL pattern matches the given string.
    pub fn remove_rules_by_url(&mut self, url_pattern: &str) {
        self.rules
            .retain(|r| r.pattern.url_pattern.as_deref() != Some(url_pattern));
        self.response_rules
            .retain(|r| r.pattern.url_pattern.as_deref() != Some(url_pattern));
    }

    /// Clear all rules but keep patterns (interception stays enabled).
    pub fn clear_rules(&mut self) {
        self.rules.clear();
        self.response_rules.clear();
    }

    /// Get the interception log.
    pub fn intercepted_log(&self) -> &[InterceptedRequestInfo] {
        &self.intercepted_log
    }

    /// Clear the interception log.
    pub fn clear_log(&mut self) {
        self.intercepted_log.clear();
    }

    /// Check a request against patterns and rules.
    ///
    /// Returns the action to take and an optional `InterceptedRequestInfo` for
    /// logging/events. If interception is disabled or no pattern matches,
    /// returns `Continue(None)` with no info.
    pub fn check_request(
        &mut self,
        url: &str,
        resource_type: &ResourceType,
        is_navigation: bool,
    ) -> (InterceptAction, Option<InterceptedRequestInfo>) {
        if !self.enabled {
            return (InterceptAction::Continue(None), None);
        }

        // Check if URL matches any enabled request-stage pattern
        let matches_pattern = self.patterns.is_empty()
            || self.patterns.iter().any(|p| {
                if p.request_stage != RequestStage::Request {
                    return false;
                }
                pattern_matches(p, url, resource_type)
            });

        if !matches_pattern {
            return (InterceptAction::Continue(None), None);
        }

        let info = InterceptedRequestInfo {
            request_id: next_request_id(),
            url: url.to_string(),
            method: "GET".to_string(),
            resource_type: resource_type.as_str().to_string(),
            is_navigation,
        };

        // Bounded log
        if self.intercepted_log.len() >= self.max_log_entries {
            self.intercepted_log.remove(0);
        }
        self.intercepted_log.push(info.clone());

        // Find the first matching rule
        for rule in &self.rules {
            if pattern_matches(&rule.pattern, url, resource_type) {
                return (rule.action.clone(), Some(info));
            }
        }

        // No specific rule — continue normally
        (InterceptAction::Continue(None), Some(info))
    }

    /// Check a response against response rules and apply modifications in place.
    ///
    /// Returns `true` if the response was modified.
    pub fn check_response(
        &self,
        url: &str,
        resource_type: &ResourceType,
        result: &mut FetchResult,
    ) -> bool {
        if !self.enabled {
            return false;
        }

        for rule in &self.response_rules {
            if pattern_matches(&rule.pattern, url, resource_type) {
                let mut modified = false;
                if let Some(status) = rule.overrides.status {
                    result.status = status;
                    modified = true;
                }
                if let Some(ref body) = rule.overrides.body {
                    result.html = body.clone();
                    result.html_bytes = body.len();
                    modified = true;
                }
                return modified;
            }
        }

        false
    }

    /// Create a mock `FetchResult` from fulfill parameters.
    pub fn fulfill_request(params: &FulfillParams, url: &str) -> FetchResult {
        let body = params.body.clone().unwrap_or_default();
        let body_len = body.len();

        let content_type = params
            .headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| "text/html".to_string());

        FetchResult {
            url: url.to_string(),
            status: params.status,
            content_type,
            html: body,
            html_bytes: body_len,
            load_ms: 0,
            set_cookies: Vec::new(),
        }
    }

    /// Convert a fail reason to a `FetchError`.
    pub fn fail_request(reason: &ErrorReason, url: &str) -> FetchError {
        match reason {
            ErrorReason::TimedOut => FetchError::Timeout(0),
            ErrorReason::AccessDenied => FetchError::HttpError {
                status: 403,
                url: url.to_string(),
            },
            _ => FetchError::NavigationFailed(format!("{}: {}", reason.as_str(), url)),
        }
    }
}

/// Check if a pattern matches a URL and resource type.
fn pattern_matches(pattern: &RequestPattern, url: &str, resource_type: &ResourceType) -> bool {
    if let Some(ref rt) = pattern.resource_type {
        if rt != resource_type {
            return false;
        }
    }

    match &pattern.url_pattern {
        None => true,
        Some(pat) => url_glob_match(pat, url),
    }
}

/// Simple glob-style URL pattern matching.
///
/// Supports `*` as a wildcard for any sequence of characters.
/// Examples: `"*"`, `"*.js"`, `"https://example.com/*"`, `"*tracking*"`.
pub fn url_glob_match(pattern: &str, url: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    let parts: Vec<&str> = pattern.split('*').collect();

    if parts.len() == 1 {
        // No wildcards — exact match
        return url == pattern;
    }

    let mut pos = 0;

    // First part must match the start
    if let Some(first) = parts.first() {
        if !first.is_empty() {
            if !url.starts_with(first) {
                return false;
            }
            pos = first.len();
        }
    }

    // Middle parts must appear in order
    for part in &parts[1..parts.len() - 1] {
        if part.is_empty() {
            continue;
        }
        match url[pos..].find(part) {
            Some(idx) => pos += idx + part.len(),
            None => return false,
        }
    }

    // Last part must match the end
    if let Some(last) = parts.last() {
        if !last.is_empty() {
            if !url[pos..].ends_with(last) {
                return false;
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- URL glob matching ----

    #[test]
    fn test_url_glob_match_wildcard_all() {
        assert!(url_glob_match("*", "https://example.com/page"));
    }

    #[test]
    fn test_url_glob_match_exact() {
        assert!(url_glob_match("https://example.com", "https://example.com"));
        assert!(!url_glob_match("https://example.com", "https://other.com"));
    }

    #[test]
    fn test_url_glob_match_prefix() {
        assert!(url_glob_match(
            "https://example.com/*",
            "https://example.com/page"
        ));
        assert!(url_glob_match(
            "https://example.com/*",
            "https://example.com/"
        ));
        assert!(!url_glob_match(
            "https://example.com/*",
            "https://other.com/page"
        ));
    }

    #[test]
    fn test_url_glob_match_suffix() {
        assert!(url_glob_match("*.js", "https://example.com/script.js"));
        assert!(!url_glob_match("*.js", "https://example.com/page.html"));
    }

    #[test]
    fn test_url_glob_match_middle() {
        assert!(url_glob_match(
            "*://api.*/*",
            "https://api.example.com/v1"
        ));
        assert!(url_glob_match(
            "*tracking*",
            "https://ads.example.com/tracking/pixel"
        ));
    }

    // ---- Interceptor lifecycle ----

    #[test]
    fn test_interceptor_disabled_by_default() {
        let mut interceptor = NetworkInterceptor::new();
        let (action, info) =
            interceptor.check_request("https://example.com", &ResourceType::Document, true);
        assert!(matches!(action, InterceptAction::Continue(None)));
        assert!(info.is_none());
    }

    #[test]
    fn test_interceptor_enable_with_patterns() {
        let mut interceptor = NetworkInterceptor::new();
        interceptor.enable(vec![RequestPattern {
            url_pattern: Some("*.js".to_string()),
            resource_type: None,
            request_stage: RequestStage::Request,
        }]);

        // JS file should be intercepted
        let (_, info) = interceptor.check_request(
            "https://example.com/app.js",
            &ResourceType::Script,
            false,
        );
        assert!(info.is_some());

        // HTML page should not match the *.js pattern
        let (_, info) = interceptor.check_request(
            "https://example.com/page.html",
            &ResourceType::Document,
            true,
        );
        assert!(info.is_none());
    }

    #[test]
    fn test_disable_clears_state() {
        let mut interceptor = NetworkInterceptor::new();
        interceptor.enable(vec![RequestPattern {
            url_pattern: Some("*".to_string()),
            resource_type: None,
            request_stage: RequestStage::Request,
        }]);
        interceptor.add_rule(InterceptRule {
            pattern: RequestPattern {
                url_pattern: Some("*".to_string()),
                resource_type: None,
                request_stage: RequestStage::Request,
            },
            action: InterceptAction::Fail(ErrorReason::BlockedByClient),
        });

        interceptor.disable();

        assert!(!interceptor.is_enabled());
        let (action, info) =
            interceptor.check_request("https://example.com", &ResourceType::Document, true);
        assert!(matches!(action, InterceptAction::Continue(None)));
        assert!(info.is_none());
    }

    // ---- Interception rules ----

    #[test]
    fn test_block_rule() {
        let mut interceptor = NetworkInterceptor::new();
        interceptor.enable(vec![RequestPattern {
            url_pattern: Some("*".to_string()),
            resource_type: None,
            request_stage: RequestStage::Request,
        }]);
        interceptor.add_rule(InterceptRule {
            pattern: RequestPattern {
                url_pattern: Some("*tracking*".to_string()),
                resource_type: None,
                request_stage: RequestStage::Request,
            },
            action: InterceptAction::Fail(ErrorReason::BlockedByClient),
        });

        let (action, _) = interceptor.check_request(
            "https://ads.example.com/tracking/pixel",
            &ResourceType::Image,
            false,
        );
        assert!(matches!(
            action,
            InterceptAction::Fail(ErrorReason::BlockedByClient)
        ));

        // Non-tracking URL should continue
        let (action, _) =
            interceptor.check_request("https://example.com/page", &ResourceType::Document, true);
        assert!(matches!(action, InterceptAction::Continue(None)));
    }

    #[test]
    fn test_fulfill_rule() {
        let mut interceptor = NetworkInterceptor::new();
        interceptor.enable(vec![RequestPattern {
            url_pattern: Some("*".to_string()),
            resource_type: None,
            request_stage: RequestStage::Request,
        }]);
        interceptor.add_rule(InterceptRule {
            pattern: RequestPattern {
                url_pattern: Some("*/api/data*".to_string()),
                resource_type: None,
                request_stage: RequestStage::Request,
            },
            action: InterceptAction::Fulfill(FulfillParams {
                status: 200,
                headers: vec![("Content-Type".to_string(), "application/json".to_string())],
                body: Some(r#"{"mock": true}"#.to_string()),
            }),
        });

        let (action, info) = interceptor.check_request(
            "https://example.com/api/data?q=test",
            &ResourceType::XHR,
            false,
        );
        assert!(matches!(action, InterceptAction::Fulfill(_)));
        assert!(info.is_some());
    }

    #[test]
    fn test_continue_with_overrides() {
        let mut interceptor = NetworkInterceptor::new();
        interceptor.enable(vec![RequestPattern {
            url_pattern: Some("*".to_string()),
            resource_type: None,
            request_stage: RequestStage::Request,
        }]);

        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token123".to_string());

        interceptor.add_rule(InterceptRule {
            pattern: RequestPattern {
                url_pattern: Some("*/api/*".to_string()),
                resource_type: None,
                request_stage: RequestStage::Request,
            },
            action: InterceptAction::Continue(Some(RequestOverrides {
                headers: Some(headers),
                ..Default::default()
            })),
        });

        let (action, _) = interceptor.check_request(
            "https://example.com/api/users",
            &ResourceType::XHR,
            false,
        );
        match action {
            InterceptAction::Continue(Some(overrides)) => {
                assert!(overrides.headers.unwrap().contains_key("Authorization"));
            }
            _ => panic!("Expected Continue with overrides"),
        }
    }

    #[test]
    fn test_resource_type_filter() {
        let mut interceptor = NetworkInterceptor::new();
        interceptor.enable(vec![RequestPattern {
            url_pattern: None,
            resource_type: Some(ResourceType::Script),
            request_stage: RequestStage::Request,
        }]);
        interceptor.add_rule(InterceptRule {
            pattern: RequestPattern {
                url_pattern: None,
                resource_type: Some(ResourceType::Script),
                request_stage: RequestStage::Request,
            },
            action: InterceptAction::Fail(ErrorReason::BlockedByClient),
        });

        // Script should be blocked
        let (action, _) = interceptor.check_request(
            "https://example.com/app.js",
            &ResourceType::Script,
            false,
        );
        assert!(matches!(action, InterceptAction::Fail(_)));

        // Document should pass through (doesn't match pattern)
        let (action, info) =
            interceptor.check_request("https://example.com/page", &ResourceType::Document, true);
        assert!(matches!(action, InterceptAction::Continue(None)));
        assert!(info.is_none());
    }

    #[test]
    fn test_first_matching_rule_wins() {
        let mut interceptor = NetworkInterceptor::new();
        interceptor.enable(vec![RequestPattern {
            url_pattern: Some("*".to_string()),
            resource_type: None,
            request_stage: RequestStage::Request,
        }]);

        // Specific rule first
        interceptor.add_rule(InterceptRule {
            pattern: RequestPattern {
                url_pattern: Some("*/api/*".to_string()),
                resource_type: None,
                request_stage: RequestStage::Request,
            },
            action: InterceptAction::Fulfill(FulfillParams {
                status: 200,
                headers: vec![],
                body: Some("mock".to_string()),
            }),
        });

        // Broader rule second — should NOT apply to /api/ URLs
        interceptor.add_rule(InterceptRule {
            pattern: RequestPattern {
                url_pattern: Some("*".to_string()),
                resource_type: None,
                request_stage: RequestStage::Request,
            },
            action: InterceptAction::Fail(ErrorReason::BlockedByClient),
        });

        let (action, _) = interceptor.check_request(
            "https://example.com/api/users",
            &ResourceType::XHR,
            false,
        );
        assert!(matches!(action, InterceptAction::Fulfill(_)));
    }

    // ---- Response interception ----

    #[test]
    fn test_response_modification() {
        let mut interceptor = NetworkInterceptor::new();
        interceptor.enable(vec![RequestPattern {
            url_pattern: Some("*".to_string()),
            resource_type: None,
            request_stage: RequestStage::Response,
        }]);
        interceptor.add_response_rule(ResponseRule {
            pattern: RequestPattern {
                url_pattern: Some("*example.com*".to_string()),
                resource_type: None,
                request_stage: RequestStage::Response,
            },
            overrides: ResponseOverrides {
                body: Some("<html>Modified</html>".to_string()),
                ..Default::default()
            },
        });

        let mut result = FetchResult {
            url: "https://example.com/page".to_string(),
            status: 200,
            content_type: "text/html".to_string(),
            html: "<html>Original</html>".to_string(),
            html_bytes: 21,
            load_ms: 100,
            set_cookies: vec![],
        };

        let modified =
            interceptor.check_response("https://example.com/page", &ResourceType::Document, &mut result);
        assert!(modified);
        assert_eq!(result.html, "<html>Modified</html>");
    }

    // ---- Logging ----

    #[test]
    fn test_intercepted_log() {
        let mut interceptor = NetworkInterceptor::new();
        interceptor.enable(vec![RequestPattern {
            url_pattern: Some("*".to_string()),
            resource_type: None,
            request_stage: RequestStage::Request,
        }]);

        interceptor.check_request("https://a.com", &ResourceType::Document, true);
        interceptor.check_request("https://b.com", &ResourceType::Script, false);

        assert_eq!(interceptor.intercepted_log().len(), 2);
        assert_eq!(interceptor.intercepted_log()[0].url, "https://a.com");
        assert_eq!(interceptor.intercepted_log()[1].url, "https://b.com");
    }

    // ---- Fulfill / Fail helpers ----

    #[test]
    fn test_fulfill_request_helper() {
        let params = FulfillParams {
            status: 200,
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            body: Some(r#"{"data": "mock"}"#.to_string()),
        };

        let result = NetworkInterceptor::fulfill_request(&params, "https://api.example.com/data");
        assert_eq!(result.status, 200);
        assert_eq!(result.content_type, "application/json");
        assert_eq!(result.html, r#"{"data": "mock"}"#);
        assert_eq!(result.url, "https://api.example.com/data");
        assert_eq!(result.load_ms, 0);
    }

    #[test]
    fn test_fail_request_helper() {
        let err = NetworkInterceptor::fail_request(
            &ErrorReason::BlockedByClient,
            "https://example.com/ad",
        );
        assert!(err.to_string().contains("Blocked"));

        let err =
            NetworkInterceptor::fail_request(&ErrorReason::TimedOut, "https://example.com/slow");
        assert!(matches!(err, FetchError::Timeout(_)));
    }

    #[test]
    fn test_remove_rules_by_url() {
        let mut interceptor = NetworkInterceptor::new();
        interceptor.enable(vec![RequestPattern {
            url_pattern: Some("*".to_string()),
            resource_type: None,
            request_stage: RequestStage::Request,
        }]);

        interceptor.add_rule(InterceptRule {
            pattern: RequestPattern {
                url_pattern: Some("*tracking*".to_string()),
                resource_type: None,
                request_stage: RequestStage::Request,
            },
            action: InterceptAction::Fail(ErrorReason::BlockedByClient),
        });

        interceptor.add_rule(InterceptRule {
            pattern: RequestPattern {
                url_pattern: Some("*/api/*".to_string()),
                resource_type: None,
                request_stage: RequestStage::Request,
            },
            action: InterceptAction::Fulfill(FulfillParams {
                status: 200,
                headers: vec![],
                body: None,
            }),
        });

        // Remove tracking rule
        interceptor.remove_rules_by_url("*tracking*");

        // Tracking URL should now continue (rule removed)
        let (action, _) = interceptor.check_request(
            "https://ads.example.com/tracking/pixel",
            &ResourceType::Image,
            false,
        );
        assert!(matches!(action, InterceptAction::Continue(None)));

        // API rule should still work
        let (action, _) = interceptor.check_request(
            "https://example.com/api/data",
            &ResourceType::XHR,
            false,
        );
        assert!(matches!(action, InterceptAction::Fulfill(_)));
    }
}
