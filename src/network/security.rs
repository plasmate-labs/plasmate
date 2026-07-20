//! Network security policy shared by every outbound HTTP client.
//!
//! The default is intentionally fail-closed: only HTTP(S) destinations whose
//! resolved addresses are globally routable are allowed.  This protects CLI,
//! daemon, AWP, CDP, MCP, external-script and page-JavaScript fetches from SSRF.

use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use url::{Host, Url};

pub const UNSAFE_PRIVATE_NETWORK_ENV: &str = "PLASMATE_UNSAFE_ALLOW_PRIVATE_NETWORK";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutboundUrlPolicy {
    allow_private_network: bool,
}

impl Default for OutboundUrlPolicy {
    fn default() -> Self {
        Self::from_environment()
    }
}

impl OutboundUrlPolicy {
    pub fn from_environment() -> Self {
        let allow_private_network = std::env::var(UNSAFE_PRIVATE_NETWORK_ENV)
            .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE"))
            .unwrap_or(false);
        Self {
            allow_private_network,
        }
    }

    /// Fail-closed policy for discovery and other product surfaces that must
    /// never inherit the process-wide private-network development escape hatch.
    pub(crate) const fn public_network_only() -> Self {
        Self {
            allow_private_network: false,
        }
    }

    /// Explicit policy for deterministic local fixtures. Never use in product paths.
    pub(crate) const fn for_local_fixtures() -> Self {
        Self {
            allow_private_network: true,
        }
    }

    #[cfg(test)]
    pub(crate) const fn deny_private_network() -> Self {
        Self::public_network_only()
    }

    #[cfg(test)]
    pub const fn for_test_fixtures() -> Self {
        Self::for_local_fixtures()
    }

    pub fn allows_private_network(&self) -> bool {
        self.allow_private_network
    }

    pub fn validate_url_syntax(&self, value: &str) -> Result<Url, String> {
        let url = Url::parse(value).map_err(|e| format!("invalid URL: {e}"))?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err(format!(
                "outbound URL scheme '{}' is not allowed; only http and https are permitted",
                url.scheme()
            ));
        }
        if !url.username().is_empty() || url.password().is_some() {
            return Err("outbound URLs containing credentials are not allowed".to_string());
        }
        let host = url
            .host_str()
            .ok_or_else(|| "outbound URL must contain a host".to_string())?;
        if is_metadata_hostname(host) && !self.allow_private_network {
            return Err(format!(
                "outbound URL host '{host}' is a cloud metadata target"
            ));
        }
        if let Some(ip) = literal_ip(&url) {
            self.validate_ip(ip)?;
        }
        Ok(url)
    }

    pub async fn validate_url(&self, value: &str) -> Result<Url, String> {
        let url = self.validate_url_syntax(value)?;
        if self.allow_private_network || literal_ip(&url).is_some() {
            return Ok(url);
        }
        let host = url.host_str().expect("validated host");
        let port = url.port_or_known_default().unwrap_or(443);
        let addresses: Vec<SocketAddr> = tokio::net::lookup_host((host, port))
            .await
            .map_err(|e| format!("DNS resolution failed for '{host}': {e}"))?
            .collect();
        self.validate_resolved(host, &addresses)?;
        Ok(url)
    }

    pub fn validate_url_blocking(&self, value: &str) -> Result<Url, String> {
        use std::net::ToSocketAddrs;
        let url = self.validate_url_syntax(value)?;
        if self.allow_private_network || literal_ip(&url).is_some() {
            return Ok(url);
        }
        let host = url.host_str().expect("validated host");
        let port = url.port_or_known_default().unwrap_or(443);
        let addresses: Vec<SocketAddr> = (host, port)
            .to_socket_addrs()
            .map_err(|e| format!("DNS resolution failed for '{host}': {e}"))?
            .collect();
        self.validate_resolved(host, &addresses)?;
        Ok(url)
    }

    fn validate_resolved(&self, host: &str, addresses: &[SocketAddr]) -> Result<(), String> {
        if addresses.is_empty() {
            return Err(format!("DNS resolution returned no addresses for '{host}'"));
        }
        for address in addresses {
            self.validate_ip(address.ip()).map_err(|reason| {
                format!(
                    "outbound host '{host}' resolved to blocked address {}: {reason}",
                    address.ip()
                )
            })?;
        }
        Ok(())
    }

    pub fn validate_ip(&self, ip: IpAddr) -> Result<(), String> {
        if self.allow_private_network || is_global_ip(ip) {
            Ok(())
        } else {
            Err(format!("address {ip} is not globally routable"))
        }
    }
}

/// Reqwest resolver that applies policy to the addresses used for the actual
/// connection. This closes the validation/connection DNS-rebinding gap.
#[derive(Debug, Clone)]
pub struct PolicyDnsResolver {
    policy: OutboundUrlPolicy,
}

impl PolicyDnsResolver {
    pub fn from_environment() -> Self {
        Self {
            policy: OutboundUrlPolicy::from_environment(),
        }
    }

    pub(crate) const fn with_policy(policy: OutboundUrlPolicy) -> Self {
        Self { policy }
    }
}

impl Resolve for PolicyDnsResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let host = name.as_str().to_string();
        let policy = self.policy;
        Box::pin(async move {
            let addresses: Vec<SocketAddr> = tokio::net::lookup_host((host.as_str(), 0))
                .await
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?
                .collect();
            policy.validate_resolved(&host, &addresses).map_err(
                |e| -> Box<dyn std::error::Error + Send + Sync> {
                    Box::new(io::Error::new(io::ErrorKind::PermissionDenied, e))
                },
            )?;
            Ok(Box::new(addresses.into_iter()) as Addrs)
        })
    }
}

pub fn is_loopback_bind_host(host: &str) -> bool {
    let trimmed = host.trim_matches(['[', ']']);
    trimmed.eq_ignore_ascii_case("localhost")
        || trimmed
            .parse::<IpAddr>()
            .map(|ip| ip.is_loopback())
            .unwrap_or(false)
}

fn is_metadata_hostname(host: &str) -> bool {
    matches!(
        host.trim_end_matches('.').to_ascii_lowercase().as_str(),
        "metadata.google.internal" | "metadata.azure.internal"
    )
}

fn literal_ip(url: &Url) -> Option<IpAddr> {
    match url.host()? {
        Host::Ipv4(ip) => Some(IpAddr::V4(ip)),
        Host::Ipv6(ip) => Some(IpAddr::V6(ip)),
        Host::Domain(_) => None,
    }
}

fn is_global_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => is_global_v4(ip),
        IpAddr::V6(ip) => is_global_v6(ip),
    }
}

fn is_global_v4(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();
    !(ip.is_unspecified()
        || ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_multicast()
        || ip.is_broadcast()
        || a == 0
        || a >= 224
        || (a == 100 && (64..=127).contains(&b))
        || (a == 192 && b == 0 && c == 0)
        || (a == 192 && b == 0 && c == 2)
        || (a == 198 && (b == 18 || b == 19))
        || (a == 198 && b == 51 && c == 100)
        || (a == 203 && b == 0 && c == 113))
}

fn is_global_v6(ip: Ipv6Addr) -> bool {
    if let Some(v4) = ip.to_ipv4_mapped() {
        return is_global_v4(v4);
    }
    let segments = ip.segments();
    // Public IPv6 unicast is allocated from 2000::/3. Fail closed for
    // unallocated/reserved blocks rather than attempting to enumerate them.
    (segments[0] & 0xe000) == 0x2000
        && !(ip.is_unspecified()
            || ip.is_loopback()
            || ip.is_multicast()
            || (segments[0] & 0xfe00) == 0xfc00
            || (segments[0] & 0xffc0) == 0xfe80
            || (segments[0] == 0x2001 && segments[1] == 0x0db8))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_http_schemes_and_credentials() {
        let policy = OutboundUrlPolicy {
            allow_private_network: false,
        };
        assert!(policy.validate_url_syntax("file:///etc/passwd").is_err());
        assert!(policy.validate_url_syntax("ftp://example.com/a").is_err());
        assert!(policy
            .validate_url_syntax("data:text/plain,secret")
            .is_err());
        assert!(policy
            .validate_url_syntax("unix:///var/run/docker.sock")
            .is_err());
        assert!(policy
            .validate_url_syntax("http+unix://%2Fvar%2Frun%2Fdocker.sock/info")
            .is_err());
        assert!(policy.validate_url_syntax("not a URL").is_err());
        assert!(policy.validate_url_syntax("http://[::1").is_err());
        assert!(policy
            .validate_url_syntax("https://user:pass@example.com")
            .is_err());
    }

    #[test]
    fn rejects_whatwg_numeric_ipv4_forms_that_normalize_to_loopback() {
        let policy = OutboundUrlPolicy::deny_private_network();
        for value in [
            "http://2130706433/",
            "http://0x7f000001/",
            "http://017700000001/",
            "http://127.1/",
        ] {
            assert!(
                policy.validate_url_syntax(value).is_err(),
                "allowed numeric loopback form {value}"
            );
        }
    }

    #[test]
    fn rejects_private_and_special_ipv4_ranges() {
        let policy = OutboundUrlPolicy {
            allow_private_network: false,
        };
        for value in [
            "http://127.0.0.1",
            "http://10.0.0.1",
            "http://169.254.169.254/latest/meta-data",
            "http://100.100.100.200",
            "http://192.0.2.1",
            "http://224.0.0.1",
            "http://255.255.255.255",
        ] {
            assert!(
                policy.validate_url_syntax(value).is_err(),
                "allowed {value}"
            );
        }
    }

    #[test]
    fn rejects_private_and_special_ipv6_ranges() {
        let policy = OutboundUrlPolicy {
            allow_private_network: false,
        };
        for value in [
            "http://[::1]",
            "http://[::]",
            "http://[fe80::1]",
            "http://[fd00:ec2::254]",
            "http://[ff02::1]",
            "http://[::ffff:127.0.0.1]",
        ] {
            assert!(
                policy.validate_url_syntax(value).is_err(),
                "allowed {value}"
            );
        }
    }

    #[test]
    fn explicit_fixture_policy_allows_loopback() {
        let policy = OutboundUrlPolicy::for_test_fixtures();
        assert!(policy.validate_url_syntax("http://127.0.0.1:1234").is_ok());
    }

    #[test]
    fn public_network_only_policy_cannot_be_relaxed() {
        let policy = OutboundUrlPolicy::public_network_only();
        assert!(!policy.allows_private_network());
        assert!(policy.validate_url_syntax("https://127.0.0.1/").is_err());
        assert!(policy.validate_url_syntax("https://10.0.0.1/").is_err());
    }

    #[test]
    fn recognizes_only_loopback_bind_hosts() {
        assert!(is_loopback_bind_host("127.0.0.1"));
        assert!(is_loopback_bind_host("::1"));
        assert!(is_loopback_bind_host("localhost"));
        assert!(!is_loopback_bind_host("0.0.0.0"));
        assert!(!is_loopback_bind_host("192.168.1.2"));
    }

    #[tokio::test]
    async fn rejects_hostname_that_resolves_to_loopback() {
        let policy = OutboundUrlPolicy {
            allow_private_network: false,
        };
        let error = policy
            .validate_url("http://localhost:9271")
            .await
            .unwrap_err();
        assert!(
            error.contains("blocked address"),
            "unexpected error: {error}"
        );
    }
}
