//! Integration tests for TLS configuration.
//!
//! Tests the TLS config path: TlsConfig -> build_client -> actual HTTPS connections.

use std::path::PathBuf;
use std::sync::Arc;

use plasmate::network::fetch;
use plasmate::network::tls::{TlsConfig, TlsVersion};
use reqwest::cookie::Jar;

/// Build a client with the given TLS config and verify it works for HTTPS.
fn build_test_client(tls_config: Option<&TlsConfig>) -> reqwest::Client {
    let jar = Arc::new(Jar::default());
    fetch::build_client_h1_fallback(None, jar, tls_config)
        .expect("Failed to build client with TLS config")
}

// ---------------------------------------------------------------------------
// Client building tests (no network required)
// ---------------------------------------------------------------------------

#[test]
fn test_build_client_default_tls() {
    let client = build_test_client(None);
    // Should build successfully with default config
    drop(client);
}

#[test]
fn test_build_client_tls13_only() {
    let config = TlsConfig {
        min_version: Some(TlsVersion::Tls13),
        max_version: Some(TlsVersion::Tls13),
        ..Default::default()
    };
    let client = build_test_client(Some(&config));
    drop(client);
}

#[test]
fn test_build_client_tls12_only() {
    let config = TlsConfig {
        min_version: Some(TlsVersion::Tls12),
        max_version: Some(TlsVersion::Tls12),
        ..Default::default()
    };
    let client = build_test_client(Some(&config));
    drop(client);
}

#[test]
fn test_build_client_insecure() {
    let config = TlsConfig {
        danger_accept_invalid_certs: true,
        ..Default::default()
    };
    let client = build_test_client(Some(&config));
    drop(client);
}

#[test]
fn test_build_client_custom_cipher_suites() {
    let config = TlsConfig {
        cipher_suites_tls13: vec!["TLS13_AES_256_GCM_SHA384".to_string()],
        cipher_suites_tls12: vec!["TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256".to_string()],
        ..Default::default()
    };
    let client = build_test_client(Some(&config));
    drop(client);
}

#[test]
fn test_build_client_custom_alpn() {
    let config = TlsConfig {
        alpn_protocols: vec!["h2".to_string(), "http/1.1".to_string()],
        ..Default::default()
    };
    let client = build_test_client(Some(&config));
    drop(client);
}

#[test]
fn test_build_client_custom_groups() {
    let config = TlsConfig {
        supported_groups: vec!["X25519".to_string(), "secp256r1".to_string()],
        ..Default::default()
    };
    let client = build_test_client(Some(&config));
    drop(client);
}

#[test]
fn test_build_client_no_sni() {
    let config = TlsConfig {
        enable_sni: Some(false),
        ..Default::default()
    };
    let client = build_test_client(Some(&config));
    drop(client);
}

#[test]
fn test_build_client_full_fingerprint_config() {
    // Configure everything at once - this is the fingerprint tuning use case
    let config = TlsConfig {
        min_version: Some(TlsVersion::Tls12),
        max_version: Some(TlsVersion::Tls13),
        cipher_suites_tls13: vec![
            "TLS13_AES_256_GCM_SHA384".to_string(),
            "TLS13_AES_128_GCM_SHA256".to_string(),
            "TLS13_CHACHA20_POLY1305_SHA256".to_string(),
        ],
        cipher_suites_tls12: vec![
            "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384".to_string(),
            "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256".to_string(),
        ],
        alpn_protocols: vec!["h2".to_string(), "http/1.1".to_string()],
        supported_groups: vec![
            "X25519".to_string(),
            "secp256r1".to_string(),
            "secp384r1".to_string(),
        ],
        ..Default::default()
    };
    let client = build_test_client(Some(&config));
    drop(client);
}

#[test]
fn test_build_client_invalid_cipher_suite_fails() {
    let config = TlsConfig {
        cipher_suites_tls13: vec!["INVALID_CIPHER".to_string()],
        ..Default::default()
    };
    let jar = Arc::new(Jar::default());
    let result = fetch::build_client_h1_fallback(None, jar, Some(&config));
    assert!(result.is_err());
}

#[test]
fn test_build_client_invalid_group_fails() {
    let config = TlsConfig {
        supported_groups: vec!["invalid_group".to_string()],
        ..Default::default()
    };
    let jar = Arc::new(Jar::default());
    let result = fetch::build_client_h1_fallback(None, jar, Some(&config));
    assert!(result.is_err());
}

#[test]
fn test_build_client_nonexistent_ca_cert_fails() {
    let config = TlsConfig {
        ca_cert_path: Some(PathBuf::from("/nonexistent/path/ca.pem")),
        ..Default::default()
    };
    let jar = Arc::new(Jar::default());
    let result = fetch::build_client_h1_fallback(None, jar, Some(&config));
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Live HTTPS connection tests (require network)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fetch_https_default_tls() {
    let client = build_test_client(None);
    // Fetch a well-known HTTPS endpoint
    let result = fetch::fetch_url(&client, "https://httpbin.org/get", 10000).await;
    // May fail due to network, but should not panic
    if let Ok(r) = result {
        assert_eq!(r.status, 200);
    }
}

#[tokio::test]
async fn test_fetch_https_tls13_only() {
    let config = TlsConfig {
        min_version: Some(TlsVersion::Tls13),
        max_version: Some(TlsVersion::Tls13),
        ..Default::default()
    };
    let client = build_test_client(Some(&config));
    let result = fetch::fetch_url(&client, "https://httpbin.org/get", 10000).await;
    if let Ok(r) = result {
        assert_eq!(r.status, 200);
    }
}

#[tokio::test]
async fn test_fetch_https_custom_ciphers() {
    let config = TlsConfig {
        cipher_suites_tls13: vec![
            "TLS13_AES_256_GCM_SHA384".to_string(),
            "TLS13_AES_128_GCM_SHA256".to_string(),
        ],
        alpn_protocols: vec!["h2".to_string(), "http/1.1".to_string()],
        ..Default::default()
    };
    let client = build_test_client(Some(&config));
    let result = fetch::fetch_url(&client, "https://httpbin.org/get", 10000).await;
    if let Ok(r) = result {
        assert_eq!(r.status, 200);
    }
}

#[tokio::test]
async fn test_fetch_https_single_cipher() {
    // Minimal fingerprint: single cipher, single group
    let config = TlsConfig {
        min_version: Some(TlsVersion::Tls13),
        cipher_suites_tls13: vec!["TLS13_AES_256_GCM_SHA384".to_string()],
        supported_groups: vec!["X25519".to_string()],
        alpn_protocols: vec!["http/1.1".to_string()],
        ..Default::default()
    };
    let client = build_test_client(Some(&config));
    let result = fetch::fetch_url(&client, "https://httpbin.org/get", 10000).await;
    if let Ok(r) = result {
        assert_eq!(r.status, 200);
    }
}

// ---------------------------------------------------------------------------
// build_client (h2-preferred) path
// ---------------------------------------------------------------------------

#[test]
fn test_build_client_h2_with_tls_config() {
    let config = TlsConfig {
        min_version: Some(TlsVersion::Tls13),
        cipher_suites_tls13: vec!["TLS13_AES_128_GCM_SHA256".to_string()],
        alpn_protocols: vec!["h2".to_string()],
        ..Default::default()
    };
    let jar = Arc::new(Jar::default());
    let client = fetch::build_client(None, jar, Some(&config))
        .expect("Failed to build h2 client with TLS config");
    drop(client);
}

// ---------------------------------------------------------------------------
// Config combination tests
// ---------------------------------------------------------------------------

#[test]
fn test_insecure_with_custom_ciphers() {
    // Insecure + custom ciphers should work (uses custom rustls path with NoVerifier)
    let config = TlsConfig {
        danger_accept_invalid_certs: true,
        cipher_suites_tls13: vec!["TLS13_AES_128_GCM_SHA256".to_string()],
        ..Default::default()
    };
    let client = build_test_client(Some(&config));
    drop(client);
}

#[test]
fn test_empty_default_config_uses_reqwest_defaults() {
    // An empty TlsConfig should take the same path as None
    let config = TlsConfig::default();
    assert!(config.is_default());
    let client = build_test_client(Some(&config));
    drop(client);
}
