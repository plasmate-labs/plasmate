//! Proxy configuration for HTTP client.
//!
//! Supports HTTP, HTTPS, and SOCKS5 proxies with optional authentication.
//! Includes proxy pool with rotation strategies for high-volume scraping.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;
use std::time::{Duration, Instant};

use rand::Rng;

use reqwest::Proxy;
use serde::{Deserialize, Serialize};

/// Proxy configuration for a session.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProxyConfig {
    /// Proxy URL (e.g., "http://proxy:8080", "socks5://proxy:1080")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Username for proxy authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// Password for proxy authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    /// Bypass proxy for these hosts (comma-separated or array).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bypass: Option<Vec<String>>,
}

impl ProxyConfig {
    /// Create an empty (no proxy) config.
    pub fn none() -> Self {
        ProxyConfig::default()
    }

    /// Create a simple HTTP proxy config.
    pub fn http(url: &str) -> Self {
        ProxyConfig {
            url: Some(url.to_string()),
            ..Default::default()
        }
    }

    /// Create a SOCKS5 proxy config.
    pub fn socks5(url: &str) -> Self {
        let url = if url.starts_with("socks5://") {
            url.to_string()
        } else {
            format!("socks5://{}", url)
        };
        ProxyConfig {
            url: Some(url),
            ..Default::default()
        }
    }

    /// Add authentication credentials.
    pub fn with_auth(mut self, username: &str, password: &str) -> Self {
        self.username = Some(username.to_string());
        self.password = Some(password.to_string());
        self
    }

    /// Add bypass hosts.
    pub fn with_bypass(mut self, hosts: Vec<String>) -> Self {
        self.bypass = Some(hosts);
        self
    }

    /// Check if proxy is configured.
    pub fn is_enabled(&self) -> bool {
        self.url.is_some()
    }

    /// Apply proxy configuration to a reqwest ClientBuilder.
    pub fn apply_to_builder(
        &self,
        builder: reqwest::ClientBuilder,
    ) -> Result<reqwest::ClientBuilder, String> {
        let url = match &self.url {
            Some(u) => u,
            None => return Ok(builder), // No proxy configured
        };

        // Set NO_PROXY env var for bypass hosts before creating proxy
        // reqwest reads this automatically
        if let Some(bypass) = &self.bypass {
            let bypass_str = bypass.join(",");
            std::env::set_var("NO_PROXY", &bypass_str);
        }

        // Parse and create the proxy
        let mut proxy = Proxy::all(url).map_err(|e| format!("Invalid proxy URL: {}", e))?;

        // Add authentication if provided
        if let (Some(user), Some(pass)) = (&self.username, &self.password) {
            proxy = proxy.basic_auth(user, pass);
        }

        Ok(builder.proxy(proxy))
    }
}

/// Parse proxy config from AWP/JSON params.
pub fn proxy_from_params(params: &serde_json::Value) -> Option<ProxyConfig> {
    let proxy = params.get("proxy")?;

    // Handle simple string URL
    if let Some(url) = proxy.as_str() {
        return Some(ProxyConfig::http(url));
    }

    // Handle object config
    if let Some(obj) = proxy.as_object() {
        let url = obj.get("url").and_then(|v| v.as_str()).map(String::from);

        if url.is_none() {
            return None;
        }

        let username = obj.get("username").and_then(|v| v.as_str()).map(String::from);
        let password = obj.get("password").and_then(|v| v.as_str()).map(String::from);
        let bypass = obj.get("bypass").and_then(|v| {
            v.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
        });

        return Some(ProxyConfig {
            url,
            username,
            password,
            bypass,
        });
    }

    None
}

// ============================================================================
// Proxy Pool with Rotation
// ============================================================================

/// Rotation strategy for proxy pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RotationStrategy {
    /// Cycle through proxies in order.
    #[default]
    RoundRobin,
    /// Pick a random proxy each time.
    Random,
    /// Use the same proxy for all requests to a given domain.
    StickyPerDomain,
}

/// Health status of a proxy.
#[derive(Debug, Clone)]
struct ProxyHealth {
    failures: u32,
    last_failure: Option<Instant>,
    last_success: Option<Instant>,
}

impl Default for ProxyHealth {
    fn default() -> Self {
        ProxyHealth {
            failures: 0,
            last_failure: None,
            last_success: None,
        }
    }
}

/// A pool of proxies with rotation and health tracking.
#[derive(Debug)]
pub struct ProxyPool {
    proxies: Vec<ProxyConfig>,
    /// The rotation strategy for selecting proxies.
    pub strategy: RotationStrategy,
    /// Current index for round-robin rotation.
    index: AtomicUsize,
    /// Domain -> proxy index mapping for sticky strategy.
    sticky_map: RwLock<HashMap<String, usize>>,
    /// Health status per proxy.
    health: RwLock<Vec<ProxyHealth>>,
    /// Number of failures before marking proxy unhealthy.
    failure_threshold: u32,
    /// Time to wait before retrying an unhealthy proxy.
    recovery_time: Duration,
}

impl ProxyPool {
    /// Create a new proxy pool with the given proxies and strategy.
    pub fn new(proxies: Vec<ProxyConfig>, strategy: RotationStrategy) -> Self {
        let count = proxies.len();
        ProxyPool {
            proxies,
            strategy,
            index: AtomicUsize::new(0),
            sticky_map: RwLock::new(HashMap::new()),
            health: RwLock::new(vec![ProxyHealth::default(); count]),
            failure_threshold: 3,
            recovery_time: Duration::from_secs(60),
        }
    }

    /// Create a pool from a list of proxy URLs (simple format).
    pub fn from_urls(urls: &[&str]) -> Self {
        let proxies: Vec<ProxyConfig> = urls
            .iter()
            .map(|url| ProxyConfig::http(url))
            .collect();
        Self::new(proxies, RotationStrategy::RoundRobin)
    }

    /// Set the rotation strategy.
    pub fn with_strategy(mut self, strategy: RotationStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set the failure threshold before marking unhealthy.
    pub fn with_failure_threshold(mut self, threshold: u32) -> Self {
        self.failure_threshold = threshold;
        self
    }

    /// Set the recovery time for unhealthy proxies.
    pub fn with_recovery_time(mut self, duration: Duration) -> Self {
        self.recovery_time = duration;
        self
    }

    /// Get the number of proxies in the pool.
    pub fn len(&self) -> usize {
        self.proxies.len()
    }

    /// Check if the pool is empty.
    pub fn is_empty(&self) -> bool {
        self.proxies.is_empty()
    }

    /// Get the next proxy based on the rotation strategy.
    /// Returns None if all proxies are unhealthy.
    pub fn next(&self) -> Option<&ProxyConfig> {
        self.next_for_domain(None)
    }

    /// Get the next proxy, optionally sticky to a domain.
    pub fn next_for_domain(&self, domain: Option<&str>) -> Option<&ProxyConfig> {
        if self.proxies.is_empty() {
            return None;
        }

        let idx = match self.strategy {
            RotationStrategy::RoundRobin => self.next_round_robin(),
            RotationStrategy::Random => self.next_random(),
            RotationStrategy::StickyPerDomain => {
                if let Some(d) = domain {
                    self.next_sticky(d)
                } else {
                    self.next_round_robin()
                }
            }
        };

        idx.map(|i| &self.proxies[i])
    }

    /// Round-robin selection, skipping unhealthy proxies.
    fn next_round_robin(&self) -> Option<usize> {
        let len = self.proxies.len();
        let start = self.index.fetch_add(1, Ordering::Relaxed) % len;

        // Try each proxy once, starting from current index
        for offset in 0..len {
            let idx = (start + offset) % len;
            if self.is_healthy(idx) {
                return Some(idx);
            }
        }

        // All unhealthy - return the next one anyway (let it fail and recover)
        Some(start % len)
    }

    /// Random selection, preferring healthy proxies.
    fn next_random(&self) -> Option<usize> {
        let len = self.proxies.len();
        let healthy: Vec<usize> = (0..len).filter(|&i| self.is_healthy(i)).collect();

        let mut rng = rand::thread_rng();
        if healthy.is_empty() {
            // All unhealthy - pick random anyway
            Some(rng.gen_range(0..len))
        } else {
            Some(healthy[rng.gen_range(0..healthy.len())])
        }
    }

    /// Sticky selection - same proxy per domain.
    fn next_sticky(&self, domain: &str) -> Option<usize> {
        // Check if we already have a mapping
        {
            let map = self.sticky_map.read().unwrap();
            if let Some(&idx) = map.get(domain) {
                if self.is_healthy(idx) {
                    return Some(idx);
                }
            }
        }

        // Assign a new proxy for this domain
        let idx = self.next_round_robin()?;
        {
            let mut map = self.sticky_map.write().unwrap();
            map.insert(domain.to_string(), idx);
        }
        Some(idx)
    }

    /// Check if a proxy is healthy.
    fn is_healthy(&self, idx: usize) -> bool {
        let health = self.health.read().unwrap();
        if let Some(h) = health.get(idx) {
            if h.failures >= self.failure_threshold {
                // Check if recovery time has passed
                if let Some(last_fail) = h.last_failure {
                    return last_fail.elapsed() > self.recovery_time;
                }
            }
            true
        } else {
            true
        }
    }

    /// Report a successful request through a proxy.
    pub fn report_success(&self, proxy: &ProxyConfig) {
        if let Some(idx) = self.find_proxy_index(proxy) {
            let mut health = self.health.write().unwrap();
            if let Some(h) = health.get_mut(idx) {
                h.failures = 0;
                h.last_success = Some(Instant::now());
            }
        }
    }

    /// Report a failed request through a proxy.
    pub fn report_failure(&self, proxy: &ProxyConfig) {
        if let Some(idx) = self.find_proxy_index(proxy) {
            let mut health = self.health.write().unwrap();
            if let Some(h) = health.get_mut(idx) {
                h.failures += 1;
                h.last_failure = Some(Instant::now());
            }
        }
    }

    /// Find the index of a proxy in the pool.
    fn find_proxy_index(&self, proxy: &ProxyConfig) -> Option<usize> {
        self.proxies.iter().position(|p| p.url == proxy.url)
    }

    /// Get pool statistics.
    pub fn stats(&self) -> ProxyPoolStats {
        let health = self.health.read().unwrap();
        let healthy = health
            .iter()
            .filter(|h| h.failures < self.failure_threshold)
            .count();

        ProxyPoolStats {
            total: self.proxies.len(),
            healthy,
            unhealthy: self.proxies.len() - healthy,
        }
    }
}

/// Statistics about the proxy pool.
#[derive(Debug, Clone, Serialize)]
pub struct ProxyPoolStats {
    pub total: usize,
    pub healthy: usize,
    pub unhealthy: usize,
}

/// Parse a proxy pool from AWP/JSON params.
pub fn pool_from_params(params: &serde_json::Value) -> Option<ProxyPool> {
    let pool = params.get("proxy_pool")?;

    let proxies_val = pool.get("proxies")?;
    let proxies: Vec<ProxyConfig> = if let Some(arr) = proxies_val.as_array() {
        arr.iter()
            .filter_map(|v| {
                if let Some(url) = v.as_str() {
                    Some(ProxyConfig::http(url))
                } else if let Some(obj) = v.as_object() {
                    let url = obj.get("url").and_then(|u| u.as_str())?;
                    let username = obj.get("username").and_then(|u| u.as_str()).map(String::from);
                    let password = obj.get("password").and_then(|u| u.as_str()).map(String::from);
                    Some(ProxyConfig {
                        url: Some(url.to_string()),
                        username,
                        password,
                        bypass: None,
                    })
                } else {
                    None
                }
            })
            .collect()
    } else {
        return None;
    };

    if proxies.is_empty() {
        return None;
    }

    let strategy = pool
        .get("strategy")
        .and_then(|v| v.as_str())
        .map(|s| match s {
            "random" => RotationStrategy::Random,
            "sticky" | "sticky_per_domain" => RotationStrategy::StickyPerDomain,
            _ => RotationStrategy::RoundRobin,
        })
        .unwrap_or(RotationStrategy::RoundRobin);

    Some(ProxyPool::new(proxies, strategy))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_none() {
        let config = ProxyConfig::none();
        assert!(!config.is_enabled());
    }

    #[test]
    fn test_proxy_http() {
        let config = ProxyConfig::http("http://proxy.example.com:8080");
        assert!(config.is_enabled());
        assert_eq!(config.url.as_deref(), Some("http://proxy.example.com:8080"));
    }

    #[test]
    fn test_proxy_socks5() {
        let config = ProxyConfig::socks5("127.0.0.1:1080");
        assert!(config.is_enabled());
        assert_eq!(config.url.as_deref(), Some("socks5://127.0.0.1:1080"));
    }

    #[test]
    fn test_proxy_with_auth() {
        let config = ProxyConfig::http("http://proxy:8080")
            .with_auth("user", "pass");
        assert_eq!(config.username.as_deref(), Some("user"));
        assert_eq!(config.password.as_deref(), Some("pass"));
    }

    #[test]
    fn test_proxy_from_params_string() {
        let params = serde_json::json!({
            "proxy": "http://proxy:8080"
        });
        let config = proxy_from_params(&params).unwrap();
        assert_eq!(config.url.as_deref(), Some("http://proxy:8080"));
    }

    #[test]
    fn test_proxy_from_params_object() {
        let params = serde_json::json!({
            "proxy": {
                "url": "socks5://proxy:1080",
                "username": "user",
                "password": "secret",
                "bypass": ["localhost", "127.0.0.1"]
            }
        });
        let config = proxy_from_params(&params).unwrap();
        assert_eq!(config.url.as_deref(), Some("socks5://proxy:1080"));
        assert_eq!(config.username.as_deref(), Some("user"));
        assert_eq!(config.password.as_deref(), Some("secret"));
        assert_eq!(config.bypass, Some(vec!["localhost".to_string(), "127.0.0.1".to_string()]));
    }

    #[test]
    fn test_proxy_pool_round_robin() {
        let pool = ProxyPool::from_urls(&["http://p1:8080", "http://p2:8080", "http://p3:8080"]);
        assert_eq!(pool.len(), 3);
        assert!(!pool.is_empty());

        // Round robin should cycle through proxies
        let p1 = pool.next().unwrap();
        let p2 = pool.next().unwrap();
        let p3 = pool.next().unwrap();
        let p4 = pool.next().unwrap();

        assert_eq!(p1.url.as_deref(), Some("http://p1:8080"));
        assert_eq!(p2.url.as_deref(), Some("http://p2:8080"));
        assert_eq!(p3.url.as_deref(), Some("http://p3:8080"));
        assert_eq!(p4.url.as_deref(), Some("http://p1:8080")); // Wraps around
    }

    #[test]
    fn test_proxy_pool_health_tracking() {
        let pool = ProxyPool::from_urls(&["http://p1:8080", "http://p2:8080"])
            .with_failure_threshold(2);

        let p1 = pool.next().unwrap().clone();

        // Report failures
        pool.report_failure(&p1);
        pool.report_failure(&p1);

        // p1 should now be skipped (unhealthy)
        let next = pool.next().unwrap();
        assert_eq!(next.url.as_deref(), Some("http://p2:8080"));

        // Report success to recover
        pool.report_success(&p1);
        let stats = pool.stats();
        assert_eq!(stats.healthy, 2);
    }

    #[test]
    fn test_proxy_pool_sticky() {
        let pool = ProxyPool::from_urls(&["http://p1:8080", "http://p2:8080", "http://p3:8080"])
            .with_strategy(RotationStrategy::StickyPerDomain);

        // Same domain should get same proxy
        let domain = "example.com";
        let p1 = pool.next_for_domain(Some(domain)).unwrap();
        let p2 = pool.next_for_domain(Some(domain)).unwrap();
        assert_eq!(p1.url, p2.url);

        // Different domain might get different proxy
        let _p3 = pool.next_for_domain(Some("other.com")).unwrap();
    }

    #[test]
    fn test_pool_from_params() {
        let params = serde_json::json!({
            "proxy_pool": {
                "proxies": [
                    "http://p1:8080",
                    {"url": "http://p2:8080", "username": "user", "password": "pass"}
                ],
                "strategy": "random"
            }
        });

        let pool = pool_from_params(&params).unwrap();
        assert_eq!(pool.len(), 2);
        assert_eq!(pool.strategy, RotationStrategy::Random);
    }
}
