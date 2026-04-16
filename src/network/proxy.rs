//! Proxy configuration for HTTP client.
//!
//! Supports HTTP, HTTPS, and SOCKS5 proxies with optional authentication.

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
}
