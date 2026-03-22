//! TLS configuration for Plasmate's HTTP client.
//!
//! Provides control over TLS version, cipher suites, certificate verification,
//! custom CA certificates, ALPN protocols, and other TLS parameters that affect
//! the connection fingerprint (JA3/JA4).
//!
//! Like curl's `--ciphers`, `--tls-max`, `--tls13-ciphers`, and `-k` flags.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use rustls::crypto::ring as ring_provider;
use rustls::crypto::CryptoProvider;

/// Global TLS configuration, set once at startup.
static TLS_CONFIG: OnceLock<TlsConfig> = OnceLock::new();

/// Set the global TLS config (call once at startup).
pub fn set_global(config: TlsConfig) {
    let _ = TLS_CONFIG.set(config);
}

/// Get the global TLS config, if set.
pub fn global() -> Option<&'static TlsConfig> {
    TLS_CONFIG.get()
}

/// TLS version specifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsVersion {
    Tls12,
    Tls13,
}

impl TlsVersion {
    /// Parse from string (e.g., "1.2", "1.3", "tls1.2", "tls1.3").
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_lowercase().trim_start_matches("tls").trim_start_matches("v") {
            "1.2" | "12" => Ok(TlsVersion::Tls12),
            "1.3" | "13" => Ok(TlsVersion::Tls13),
            _ => Err(format!("Invalid TLS version '{}'. Use: 1.2 or 1.3", s)),
        }
    }

    fn to_rustls(self) -> &'static rustls::SupportedProtocolVersion {
        match self {
            TlsVersion::Tls12 => &rustls::version::TLS12,
            TlsVersion::Tls13 => &rustls::version::TLS13,
        }
    }

    fn to_reqwest(self) -> reqwest::tls::Version {
        match self {
            TlsVersion::Tls12 => reqwest::tls::Version::TLS_1_2,
            TlsVersion::Tls13 => reqwest::tls::Version::TLS_1_3,
        }
    }
}

impl std::fmt::Display for TlsVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TlsVersion::Tls12 => write!(f, "1.2"),
            TlsVersion::Tls13 => write!(f, "1.3"),
        }
    }
}

/// TLS configuration for the HTTP client.
///
/// When `needs_custom_rustls()` returns true, a custom `rustls::ClientConfig`
/// is built and passed to reqwest via `use_preconfigured_tls`. Otherwise,
/// reqwest's built-in TLS methods are used.
#[derive(Debug, Clone, Default)]
pub struct TlsConfig {
    /// Minimum TLS version (default: TLS 1.2).
    pub min_version: Option<TlsVersion>,
    /// Maximum TLS version (default: TLS 1.3).
    pub max_version: Option<TlsVersion>,
    /// Skip certificate verification (like curl -k / --insecure).
    pub danger_accept_invalid_certs: bool,
    /// Path to a PEM file containing custom CA certificates.
    pub ca_cert_path: Option<PathBuf>,
    /// Cipher suite names for TLS 1.2 (e.g., "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256").
    /// Controls cipher order in the ClientHello, affecting JA3/JA4 fingerprint.
    pub cipher_suites_tls12: Vec<String>,
    /// Cipher suite names for TLS 1.3 (e.g., "TLS13_AES_128_GCM_SHA256").
    pub cipher_suites_tls13: Vec<String>,
    /// ALPN protocol names (e.g., ["h2", "http/1.1"]).
    pub alpn_protocols: Vec<String>,
    /// Named groups / supported curves (e.g., "x25519", "secp256r1", "secp384r1").
    pub supported_groups: Vec<String>,
    /// Enable/disable Server Name Indication.
    pub enable_sni: Option<bool>,
}

impl TlsConfig {
    /// Returns true if no TLS options are configured.
    pub fn is_default(&self) -> bool {
        self.min_version.is_none()
            && self.max_version.is_none()
            && !self.danger_accept_invalid_certs
            && self.ca_cert_path.is_none()
            && self.cipher_suites_tls12.is_empty()
            && self.cipher_suites_tls13.is_empty()
            && self.alpn_protocols.is_empty()
            && self.supported_groups.is_empty()
            && self.enable_sni.is_none()
    }

    /// Returns true if we need to build a custom rustls::ClientConfig
    /// (rather than using reqwest's built-in TLS methods).
    pub fn needs_custom_rustls(&self) -> bool {
        !self.cipher_suites_tls12.is_empty()
            || !self.cipher_suites_tls13.is_empty()
            || !self.alpn_protocols.is_empty()
            || !self.supported_groups.is_empty()
    }

    /// Apply simple TLS settings to a reqwest ClientBuilder.
    /// Used when `needs_custom_rustls()` is false.
    pub fn apply_to_reqwest(
        &self,
        mut builder: reqwest::ClientBuilder,
    ) -> Result<reqwest::ClientBuilder, String> {
        if let Some(min) = self.min_version {
            builder = builder.min_tls_version(min.to_reqwest());
        }
        if let Some(max) = self.max_version {
            builder = builder.max_tls_version(max.to_reqwest());
        }
        if self.danger_accept_invalid_certs {
            builder = builder.danger_accept_invalid_certs(true);
        }
        if let Some(ref path) = self.ca_cert_path {
            let cert = load_pem_certs_reqwest(path)?;
            for c in cert {
                builder = builder.add_root_certificate(c);
            }
        }
        if let Some(sni) = self.enable_sni {
            builder = builder.tls_sni(sni);
        }
        Ok(builder)
    }

    /// Build a custom rustls::ClientConfig for fingerprint-level control.
    /// Used when `needs_custom_rustls()` is true.
    pub fn build_rustls_config(&self) -> Result<rustls::ClientConfig, String> {
        let provider = build_crypto_provider(self)?;

        // Determine protocol versions
        let versions: Vec<&'static rustls::SupportedProtocolVersion> =
            self.protocol_versions();

        let config_builder = rustls::ClientConfig::builder_with_provider(Arc::new(provider))
            .with_protocol_versions(&versions)
            .map_err(|e| format!("Failed to configure TLS versions: {e}"))?;

        // Configure certificate verification
        let mut config = if self.danger_accept_invalid_certs {
            config_builder
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(NoVerifier))
                .with_no_client_auth()
        } else {
            let root_store = build_root_store(&self.ca_cert_path)?;
            config_builder
                .with_root_certificates(root_store)
                .with_no_client_auth()
        };

        // ALPN protocols
        if !self.alpn_protocols.is_empty() {
            config.alpn_protocols = self
                .alpn_protocols
                .iter()
                .map(|p| p.as_bytes().to_vec())
                .collect();
        }

        // SNI
        if let Some(false) = self.enable_sni {
            config.enable_sni = false;
        }

        Ok(config)
    }

    /// Determine the TLS protocol versions to support.
    fn protocol_versions(&self) -> Vec<&'static rustls::SupportedProtocolVersion> {
        let min = self.min_version.unwrap_or(TlsVersion::Tls12);
        let max = self.max_version.unwrap_or(TlsVersion::Tls13);

        let mut versions = Vec::new();
        if min == TlsVersion::Tls12 || max == TlsVersion::Tls12 {
            versions.push(TlsVersion::Tls12.to_rustls());
        }
        if min != TlsVersion::Tls13 && max != TlsVersion::Tls12 {
            // Both TLS 1.2 and 1.3 are in range
            if !versions.contains(&TlsVersion::Tls12.to_rustls()) {
                versions.push(TlsVersion::Tls12.to_rustls());
            }
        }
        if max == TlsVersion::Tls13 || min == TlsVersion::Tls13 {
            versions.push(TlsVersion::Tls13.to_rustls());
        }

        // Deduplicate
        versions.dedup();
        versions
    }

    /// Summary for logging.
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        if let Some(min) = self.min_version {
            parts.push(format!("min_tls={min}"));
        }
        if let Some(max) = self.max_version {
            parts.push(format!("max_tls={max}"));
        }
        if self.danger_accept_invalid_certs {
            parts.push("insecure".to_string());
        }
        if self.ca_cert_path.is_some() {
            parts.push("custom_ca".to_string());
        }
        if !self.cipher_suites_tls12.is_empty() {
            parts.push(format!("tls12_ciphers={}", self.cipher_suites_tls12.len()));
        }
        if !self.cipher_suites_tls13.is_empty() {
            parts.push(format!("tls13_ciphers={}", self.cipher_suites_tls13.len()));
        }
        if !self.alpn_protocols.is_empty() {
            parts.push(format!("alpn={}", self.alpn_protocols.join(",")));
        }
        if !self.supported_groups.is_empty() {
            parts.push(format!("groups={}", self.supported_groups.join(",")));
        }
        if parts.is_empty() {
            "default".to_string()
        } else {
            parts.join(", ")
        }
    }
}

// ---------------------------------------------------------------------------
// Cipher suite resolution
// ---------------------------------------------------------------------------

/// All cipher suites available from the ring provider, mapped by IANA name.
fn resolve_cipher_suites(
    names: &[String],
    kind: &str,
) -> Result<Vec<rustls::SupportedCipherSuite>, String> {
    let all = ring_provider::ALL_CIPHER_SUITES;
    let mut result = Vec::new();

    for name in names {
        let upper = name.to_uppercase();
        let found = all
            .iter()
            .find(|cs| {
                let suite_name = format!("{:?}", cs.suite());
                suite_name.to_uppercase() == upper
            });
        match found {
            Some(cs) => result.push(*cs),
            None => {
                let available: Vec<String> = all
                    .iter()
                    .map(|cs| format!("{:?}", cs.suite()))
                    .collect();
                return Err(format!(
                    "Unknown {kind} cipher suite '{name}'. Available: {}",
                    available.join(", ")
                ));
            }
        }
    }

    Ok(result)
}

/// Build a CryptoProvider with the configured cipher suites and key exchange groups.
fn build_crypto_provider(config: &TlsConfig) -> Result<CryptoProvider, String> {
    let mut provider = ring_provider::default_provider();

    // Cipher suites: merge TLS 1.2 + TLS 1.3 selections, or use defaults
    if !config.cipher_suites_tls12.is_empty() || !config.cipher_suites_tls13.is_empty() {
        let mut suites = Vec::new();
        // TLS 1.3 ciphers first (standard ordering)
        if !config.cipher_suites_tls13.is_empty() {
            suites.extend(resolve_cipher_suites(&config.cipher_suites_tls13, "TLS 1.3")?);
        } else {
            // Keep default TLS 1.3 ciphers
            suites.extend(
                ring_provider::ALL_CIPHER_SUITES
                    .iter()
                    .filter(|cs| matches!(cs, rustls::SupportedCipherSuite::Tls13(_))),
            );
        }
        if !config.cipher_suites_tls12.is_empty() {
            suites.extend(resolve_cipher_suites(&config.cipher_suites_tls12, "TLS 1.2")?);
        } else {
            // Keep default TLS 1.2 ciphers
            suites.extend(
                ring_provider::ALL_CIPHER_SUITES
                    .iter()
                    .filter(|cs| matches!(cs, rustls::SupportedCipherSuite::Tls12(_))),
            );
        }
        provider.cipher_suites = suites;
    }

    // Key exchange groups / supported curves
    if !config.supported_groups.is_empty() {
        let default_groups = ring_provider::default_provider().kx_groups;
        let mut groups = Vec::new();
        for name in &config.supported_groups {
            let lower = name.to_lowercase();
            let found = default_groups.iter().find(|g| {
                let group_name = format!("{:?}", g.name());
                group_name.to_lowercase() == lower
            });
            match found {
                Some(g) => groups.push(*g),
                None => {
                    let available: Vec<String> = default_groups
                        .iter()
                        .map(|g| format!("{:?}", g.name()))
                        .collect();
                    return Err(format!(
                        "Unknown supported group '{name}'. Available: {}",
                        available.join(", ")
                    ));
                }
            }
        }
        provider.kx_groups = groups;
    }

    Ok(provider)
}

// ---------------------------------------------------------------------------
// Certificate loading
// ---------------------------------------------------------------------------

/// Load PEM certificates for the root store.
fn build_root_store(
    ca_cert_path: &Option<PathBuf>,
) -> Result<rustls::RootCertStore, String> {
    let mut root_store = rustls::RootCertStore::empty();

    // Load native/system root certificates
    let native_certs = rustls_native_certs::load_native_certs();
    for cert in native_certs.certs {
        // Ignore individual cert errors (some system certs may be malformed)
        let _ = root_store.add(cert);
    }

    // Load custom CA certificates if specified
    if let Some(path) = ca_cert_path {
        let pem_data = std::fs::read(path)
            .map_err(|e| format!("Failed to read CA cert file '{}': {e}", path.display()))?;
        let mut cursor = std::io::BufReader::new(&pem_data[..]);
        let certs: Vec<_> = rustls_pemfile::certs(&mut cursor)
            .filter_map(|r| r.ok())
            .collect();
        if certs.is_empty() {
            return Err(format!(
                "No valid PEM certificates found in '{}'",
                path.display()
            ));
        }
        for cert in certs {
            root_store
                .add(cert)
                .map_err(|e| format!("Failed to add CA cert: {e}"))?;
        }
    }

    Ok(root_store)
}

/// Load PEM certificates as reqwest::Certificate (for the simple reqwest path).
fn load_pem_certs_reqwest(path: &PathBuf) -> Result<Vec<reqwest::Certificate>, String> {
    let pem_data = std::fs::read(path)
        .map_err(|e| format!("Failed to read CA cert file '{}': {e}", path.display()))?;
    let mut certs = Vec::new();
    let mut cursor = std::io::BufReader::new(&pem_data[..]);
    for cert_result in rustls_pemfile::certs(&mut cursor) {
        let cert_der = cert_result
            .map_err(|e| format!("Failed to parse PEM cert: {e}"))?;
        let cert = reqwest::Certificate::from_der(cert_der.as_ref())
            .map_err(|e| format!("Failed to create reqwest cert: {e}"))?;
        certs.push(cert);
    }
    if certs.is_empty() {
        return Err(format!(
            "No valid PEM certificates found in '{}'",
            path.display()
        ));
    }
    Ok(certs)
}

// ---------------------------------------------------------------------------
// Certificate verification bypass (--insecure / -k)
// ---------------------------------------------------------------------------

/// A certificate verifier that accepts anything (equivalent to curl -k).
#[derive(Debug)]
struct NoVerifier;

impl rustls::client::danger::ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

// ---------------------------------------------------------------------------
// Listing available options
// ---------------------------------------------------------------------------

/// List all cipher suite names available from the ring provider.
pub fn available_cipher_suites() -> Vec<String> {
    ring_provider::ALL_CIPHER_SUITES
        .iter()
        .map(|cs| format!("{:?}", cs.suite()))
        .collect()
}

/// List all supported key exchange group names.
pub fn available_kx_groups() -> Vec<String> {
    ring_provider::default_provider()
        .kx_groups
        .iter()
        .map(|g| format!("{:?}", g.name()))
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_version_parse() {
        assert_eq!(TlsVersion::parse("1.2").unwrap(), TlsVersion::Tls12);
        assert_eq!(TlsVersion::parse("1.3").unwrap(), TlsVersion::Tls13);
        assert_eq!(TlsVersion::parse("tls1.2").unwrap(), TlsVersion::Tls12);
        assert_eq!(TlsVersion::parse("TLS1.3").unwrap(), TlsVersion::Tls13);
        assert_eq!(TlsVersion::parse("v1.2").unwrap(), TlsVersion::Tls12);
        assert!(TlsVersion::parse("1.1").is_err());
        assert!(TlsVersion::parse("ssl3").is_err());
    }

    #[test]
    fn test_tls_version_display() {
        assert_eq!(format!("{}", TlsVersion::Tls12), "1.2");
        assert_eq!(format!("{}", TlsVersion::Tls13), "1.3");
    }

    #[test]
    fn test_default_config_is_default() {
        let config = TlsConfig::default();
        assert!(config.is_default());
        assert!(!config.needs_custom_rustls());
    }

    #[test]
    fn test_insecure_does_not_need_custom_rustls() {
        let config = TlsConfig {
            danger_accept_invalid_certs: true,
            ..Default::default()
        };
        assert!(!config.is_default());
        assert!(!config.needs_custom_rustls());
    }

    #[test]
    fn test_cipher_suites_need_custom_rustls() {
        let config = TlsConfig {
            cipher_suites_tls13: vec!["TLS13_AES_128_GCM_SHA256".to_string()],
            ..Default::default()
        };
        assert!(config.needs_custom_rustls());
    }

    #[test]
    fn test_alpn_needs_custom_rustls() {
        let config = TlsConfig {
            alpn_protocols: vec!["h2".to_string()],
            ..Default::default()
        };
        assert!(config.needs_custom_rustls());
    }

    #[test]
    fn test_available_cipher_suites_not_empty() {
        let suites = available_cipher_suites();
        assert!(!suites.is_empty());
        // Should contain common TLS 1.3 cipher
        assert!(suites.iter().any(|s| s.contains("AES_128_GCM")));
    }

    #[test]
    fn test_available_kx_groups_not_empty() {
        let groups = available_kx_groups();
        assert!(!groups.is_empty());
    }

    #[test]
    fn test_resolve_valid_cipher_suite() {
        let suites = resolve_cipher_suites(
            &["TLS13_AES_128_GCM_SHA256".to_string()],
            "TLS 1.3",
        );
        assert!(suites.is_ok());
        assert_eq!(suites.unwrap().len(), 1);
    }

    #[test]
    fn test_resolve_invalid_cipher_suite() {
        let suites = resolve_cipher_suites(
            &["BOGUS_CIPHER".to_string()],
            "test",
        );
        assert!(suites.is_err());
        let err = suites.unwrap_err();
        assert!(err.contains("Unknown test cipher suite"));
        assert!(err.contains("Available:"));
    }

    #[test]
    fn test_build_rustls_config_default() {
        let config = TlsConfig {
            cipher_suites_tls13: vec!["TLS13_AES_128_GCM_SHA256".to_string()],
            ..Default::default()
        };
        let result = config.build_rustls_config();
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_rustls_config_insecure() {
        let config = TlsConfig {
            cipher_suites_tls13: vec!["TLS13_AES_128_GCM_SHA256".to_string()],
            danger_accept_invalid_certs: true,
            ..Default::default()
        };
        let result = config.build_rustls_config();
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_rustls_config_tls13_only() {
        let config = TlsConfig {
            min_version: Some(TlsVersion::Tls13),
            max_version: Some(TlsVersion::Tls13),
            cipher_suites_tls13: vec!["TLS13_AES_256_GCM_SHA384".to_string()],
            ..Default::default()
        };
        let result = config.build_rustls_config();
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_rustls_config_with_alpn() {
        let config = TlsConfig {
            alpn_protocols: vec!["h2".to_string(), "http/1.1".to_string()],
            ..Default::default()
        };
        let rustls_config = config.build_rustls_config().unwrap();
        assert_eq!(rustls_config.alpn_protocols.len(), 2);
        assert_eq!(rustls_config.alpn_protocols[0], b"h2");
        assert_eq!(rustls_config.alpn_protocols[1], b"http/1.1");
    }

    #[test]
    fn test_protocol_versions_default() {
        let config = TlsConfig::default();
        let versions = config.protocol_versions();
        assert_eq!(versions.len(), 2); // TLS 1.2 + 1.3
    }

    #[test]
    fn test_protocol_versions_tls13_only() {
        let config = TlsConfig {
            min_version: Some(TlsVersion::Tls13),
            ..Default::default()
        };
        let versions = config.protocol_versions();
        assert!(versions.contains(&TlsVersion::Tls13.to_rustls()));
    }

    #[test]
    fn test_protocol_versions_tls12_only() {
        let config = TlsConfig {
            max_version: Some(TlsVersion::Tls12),
            ..Default::default()
        };
        let versions = config.protocol_versions();
        assert!(versions.contains(&TlsVersion::Tls12.to_rustls()));
    }

    #[test]
    fn test_summary_default() {
        let config = TlsConfig::default();
        assert_eq!(config.summary(), "default");
    }

    #[test]
    fn test_summary_with_options() {
        let config = TlsConfig {
            min_version: Some(TlsVersion::Tls13),
            danger_accept_invalid_certs: true,
            alpn_protocols: vec!["h2".to_string()],
            ..Default::default()
        };
        let summary = config.summary();
        assert!(summary.contains("min_tls=1.3"));
        assert!(summary.contains("insecure"));
        assert!(summary.contains("alpn=h2"));
    }

    #[test]
    fn test_ca_cert_nonexistent_file() {
        let config = TlsConfig {
            ca_cert_path: Some(PathBuf::from("/nonexistent/cert.pem")),
            cipher_suites_tls13: vec!["TLS13_AES_128_GCM_SHA256".to_string()],
            ..Default::default()
        };
        let result = config.build_rustls_config();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read CA cert file"));
    }
}
