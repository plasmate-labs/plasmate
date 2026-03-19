//! Cookie profile storage for authenticated browsing.
//!
//! Stores per-domain cookie profiles as encrypted JSON files in
//! `~/.plasmate/profiles/<domain>.json`. Encryption uses AES-256-GCM
//! with a random master key stored in `~/.plasmate/master.key`.
//!
//! Design constraints:
//! - Zero overhead on the hot path (cookies load once at session start)
//! - Minimal storage (a few hundred bytes per domain)
//! - Transparent encryption/decryption on store/load

use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{info, warn};

/// Nonce size for AES-256-GCM (96 bits = 12 bytes)
const NONCE_SIZE: usize = 12;
/// Master key size (256 bits = 32 bytes)
const KEY_SIZE: usize = 32;

/// A stored cookie profile for a domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieProfile {
    pub domain: String,
    pub cookies: HashMap<String, CookieEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// A single cookie entry with optional expiry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieEntry {
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
}

impl CookieEntry {
    pub fn new(value: String) -> Self {
        Self {
            value,
            expires_at: None,
        }
    }

    pub fn with_expiry(value: String, expires_at: Option<i64>) -> Self {
        Self { value, expires_at }
    }
}

/// Get the plasmate config directory path.
fn config_dir() -> Result<PathBuf, std::io::Error> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "HOME not set"))?;
    Ok(PathBuf::from(home).join(".plasmate"))
}

/// Get the profiles directory path.
fn profiles_dir() -> Result<PathBuf, std::io::Error> {
    Ok(config_dir()?.join("profiles"))
}

/// Get the master key file path.
fn master_key_path() -> Result<PathBuf, std::io::Error> {
    Ok(config_dir()?.join("master.key"))
}

/// Get the file path for a domain's profile.
fn profile_path(domain: &str) -> Result<PathBuf, std::io::Error> {
    // Sanitize domain for filename
    let safe_domain = domain.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    Ok(profiles_dir()?.join(format!("{}.json", safe_domain)))
}

/// Load or generate the master encryption key.
fn get_or_create_master_key() -> Result<[u8; KEY_SIZE], Box<dyn std::error::Error>> {
    let key_path = master_key_path()?;
    let config = config_dir()?;

    // Ensure config directory exists
    std::fs::create_dir_all(&config)?;

    if key_path.exists() {
        // Load existing key
        let key_bytes = std::fs::read(&key_path)?;
        if key_bytes.len() != KEY_SIZE {
            return Err(format!(
                "Invalid master key size: expected {}, got {}",
                KEY_SIZE,
                key_bytes.len()
            )
            .into());
        }
        let mut key = [0u8; KEY_SIZE];
        key.copy_from_slice(&key_bytes);
        Ok(key)
    } else {
        // Generate new key
        let mut key = [0u8; KEY_SIZE];
        OsRng.fill_bytes(&mut key);

        // Write key with restrictive permissions
        std::fs::write(&key_path, &key)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600))?;
        }

        info!(path = %key_path.display(), "Generated new master encryption key");
        Ok(key)
    }
}

/// Encrypt data using AES-256-GCM.
fn encrypt(plaintext: &[u8], key: &[u8; KEY_SIZE]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let cipher = Aes256Gcm::new_from_slice(key)?;

    // Generate random nonce
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // Prepend nonce to ciphertext
    let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// Decrypt data using AES-256-GCM.
fn decrypt(data: &[u8], key: &[u8; KEY_SIZE]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if data.len() < NONCE_SIZE {
        return Err("Data too short to contain nonce".into());
    }

    let cipher = Aes256Gcm::new_from_slice(key)?;
    let nonce = Nonce::from_slice(&data[..NONCE_SIZE]);
    let ciphertext = &data[NONCE_SIZE..];

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e).into())
}

/// Check if data looks like valid JSON (plaintext profile).
fn is_plaintext_json(data: &[u8]) -> bool {
    // Valid JSON profile should start with '{' (possibly after whitespace)
    data.iter()
        .find(|&&b| !b.is_ascii_whitespace())
        .map(|&b| b == b'{')
        .unwrap_or(false)
}

/// Store a cookie profile for a domain (encrypted).
pub fn store_profile(profile: &CookieProfile) -> Result<(), Box<dyn std::error::Error>> {
    let dir = profiles_dir()?;
    std::fs::create_dir_all(&dir)?;

    let key = get_or_create_master_key()?;
    let path = profile_path(&profile.domain)?;
    let json = serde_json::to_string_pretty(profile)?;

    // Encrypt the JSON
    let encrypted = encrypt(json.as_bytes(), &key)?;

    std::fs::write(&path, &encrypted)?;

    // Set file permissions to owner-only (Unix)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }

    info!(domain = %profile.domain, path = %path.display(), encrypted = true, "Stored cookie profile");
    Ok(())
}

/// Load a cookie profile for a domain (decrypted).
/// Automatically migrates plaintext profiles to encrypted format.
pub fn load_profile(domain: &str) -> Result<Option<CookieProfile>, Box<dyn std::error::Error>> {
    let path = profile_path(domain)?;
    if !path.exists() {
        return Ok(None);
    }

    let data = std::fs::read(&path)?;

    // Check if this is a plaintext (legacy) profile
    if is_plaintext_json(&data) {
        // Try to parse as JSON directly
        let json_str = String::from_utf8(data)?;
        let profile: CookieProfile = match serde_json::from_str(&json_str) {
            Ok(p) => p,
            Err(_) => {
                // Try legacy format (HashMap<String, String> for cookies)
                let legacy: LegacyCookieProfile = serde_json::from_str(&json_str)?;
                legacy.into()
            }
        };

        // Migrate: re-encrypt the profile
        warn!(
            domain = %domain,
            "Migrating plaintext profile to encrypted format"
        );
        store_profile(&profile)?;

        info!(
            domain = %domain,
            cookies = profile.cookies.len(),
            "Loaded and migrated cookie profile"
        );
        return Ok(Some(profile));
    }

    // Decrypt the profile
    let key = get_or_create_master_key()?;
    let decrypted = decrypt(&data, &key)?;
    let json_str = String::from_utf8(decrypted)?;
    let profile: CookieProfile = serde_json::from_str(&json_str)?;

    info!(domain = %domain, cookies = profile.cookies.len(), "Loaded cookie profile");
    Ok(Some(profile))
}

/// Legacy profile format for migration support.
#[derive(Debug, Deserialize)]
struct LegacyCookieProfile {
    domain: String,
    cookies: HashMap<String, String>,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    notes: Option<String>,
}

impl From<LegacyCookieProfile> for CookieProfile {
    fn from(legacy: LegacyCookieProfile) -> Self {
        CookieProfile {
            domain: legacy.domain,
            cookies: legacy
                .cookies
                .into_iter()
                .map(|(k, v)| (k, CookieEntry::new(v)))
                .collect(),
            created_at: legacy.created_at,
            notes: legacy.notes,
        }
    }
}

/// List all stored profiles (domain names only, never cookie values).
pub fn list_profiles() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let dir = profiles_dir()?;
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut domains = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if let Some(name) = entry.path().file_stem() {
            if let Some(name_str) = name.to_str() {
                domains.push(name_str.to_string());
            }
        }
    }
    domains.sort();
    Ok(domains)
}

/// Delete a stored profile.
pub fn revoke_profile(domain: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let path = profile_path(domain)?;
    if path.exists() {
        std::fs::remove_file(&path)?;
        info!(domain = %domain, "Revoked cookie profile");
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Load cookies from a profile into a reqwest cookie jar.
/// Skips expired cookies with a warning.
pub fn load_into_jar(
    domain: &str,
    jar: &reqwest::cookie::Jar,
) -> Result<bool, Box<dyn std::error::Error>> {
    let profile = match load_profile(domain)? {
        Some(p) => p,
        None => return Ok(false),
    };

    let url: url::Url = format!("https://{}", domain).parse()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let mut loaded = 0;
    let mut skipped = 0;

    for (name, entry) in &profile.cookies {
        // Skip expired cookies
        if let Some(expires_at) = entry.expires_at {
            if expires_at < now {
                warn!(
                    domain = %domain,
                    cookie = %name,
                    expired_at = expires_at,
                    "Skipping expired cookie"
                );
                skipped += 1;
                continue;
            }
        }

        let cookie_str = format!(
            "{}={}; Domain={}; Path=/; Secure",
            name, entry.value, domain
        );
        jar.add_cookie_str(&cookie_str, &url);
        loaded += 1;
    }

    info!(
        domain = %domain,
        loaded = loaded,
        skipped = skipped,
        "Loaded cookies into jar"
    );
    Ok(true)
}

/// Parse a cookie string like "name1=val1; name2=val2" into a HashMap.
pub fn parse_cookie_string(s: &str) -> HashMap<String, CookieEntry> {
    let mut cookies = HashMap::new();
    for pair in s.split(';') {
        let pair = pair.trim();
        if let Some((name, value)) = pair.split_once('=') {
            cookies.insert(
                name.trim().to_string(),
                CookieEntry::new(value.trim().to_string()),
            );
        }
    }
    cookies
}

/// Parse a cookie string with optional TTL (seconds from now).
pub fn parse_cookie_string_with_ttl(
    s: &str,
    ttl_seconds: Option<i64>,
) -> HashMap<String, CookieEntry> {
    let expires_at = ttl_seconds.map(|ttl| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64 + ttl)
            .unwrap_or(0)
    });

    let mut cookies = HashMap::new();
    for pair in s.split(';') {
        let pair = pair.trim();
        if let Some((name, value)) = pair.split_once('=') {
            cookies.insert(
                name.trim().to_string(),
                CookieEntry::with_expiry(value.trim().to_string(), expires_at),
            );
        }
    }
    cookies
}

/// Generate a fingerprint hash for a profile (for display without leaking values).
pub fn profile_fingerprint(profile: &CookieProfile) -> String {
    let mut hasher = Sha256::new();
    hasher.update(profile.domain.as_bytes());
    for (k, v) in &profile.cookies {
        hasher.update(k.as_bytes());
        hasher.update(v.value.as_bytes());
    }
    let result = hasher.finalize();
    hex::encode(&result[..4])
}

/// Check if a profile file is encrypted.
pub fn is_profile_encrypted(domain: &str) -> Result<Option<bool>, Box<dyn std::error::Error>> {
    let path = profile_path(domain)?;
    if !path.exists() {
        return Ok(None);
    }
    let data = std::fs::read(&path)?;
    Ok(Some(!is_plaintext_json(&data)))
}

/// Get the expiry status for a cookie.
pub fn cookie_expiry_status(expires_at: Option<i64>) -> &'static str {
    let Some(exp) = expires_at else {
        return "no expiry";
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    if exp < now {
        "✗ expired"
    } else if exp < now + 86400 {
        // 24 hours
        "⚠ expires soon"
    } else {
        "✓ valid"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_parse_cookie_string() {
        let cookies = parse_cookie_string("ct0=abc123; auth_token=xyz789; _ga=GA1.2.3");
        assert_eq!(cookies.get("ct0").map(|e| e.value.as_str()), Some("abc123"));
        assert_eq!(
            cookies.get("auth_token").map(|e| e.value.as_str()),
            Some("xyz789")
        );
        assert_eq!(
            cookies.get("_ga").map(|e| e.value.as_str()),
            Some("GA1.2.3")
        );
        assert_eq!(cookies.len(), 3);
    }

    #[test]
    fn test_parse_cookie_string_whitespace() {
        let cookies = parse_cookie_string("  key1 = value1 ;  key2=value2  ");
        assert_eq!(
            cookies.get("key1").map(|e| e.value.as_str()),
            Some("value1")
        );
        assert_eq!(
            cookies.get("key2").map(|e| e.value.as_str()),
            Some("value2")
        );
    }

    #[test]
    fn test_encryption_roundtrip() {
        let key = [42u8; KEY_SIZE];
        let plaintext = b"Hello, World!";
        let encrypted = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();
        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    #[serial]
    fn test_store_and_load_profile() {
        let dir = tempfile::tempdir().unwrap();
        // Save original HOME to restore later
        let original_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", dir.path());

        let mut cookies = HashMap::new();
        cookies.insert("ct0".to_string(), CookieEntry::new("test_ct0".to_string()));
        cookies.insert(
            "auth_token".to_string(),
            CookieEntry::new("test_auth".to_string()),
        );

        let profile = CookieProfile {
            domain: "x.com".to_string(),
            cookies,
            created_at: Some("2026-03-18T22:00:00Z".to_string()),
            notes: None,
        };

        store_profile(&profile).unwrap();

        // Verify the file is encrypted
        let path = profile_path("x.com").unwrap();
        let data = std::fs::read(&path).unwrap();
        assert!(!is_plaintext_json(&data), "Profile should be encrypted");

        let loaded = load_profile("x.com").unwrap().unwrap();
        assert_eq!(loaded.domain, "x.com");
        assert_eq!(
            loaded.cookies.get("ct0").map(|e| e.value.as_str()),
            Some("test_ct0")
        );
        assert_eq!(
            loaded.cookies.get("auth_token").map(|e| e.value.as_str()),
            Some("test_auth")
        );

        // Restore original HOME
        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }
    }

    #[test]
    #[serial]
    fn test_legacy_migration() {
        let dir = tempfile::tempdir().unwrap();
        let original_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", dir.path());

        // Create profiles directory
        let profiles_dir = dir.path().join(".plasmate").join("profiles");
        std::fs::create_dir_all(&profiles_dir).unwrap();

        // Also create master key first to ensure it exists
        let config_dir = dir.path().join(".plasmate");
        let mut key = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut key);
        std::fs::write(config_dir.join("master.key"), &key).unwrap();

        // Write a legacy plaintext profile
        let legacy_json = r#"{
            "domain": "legacy.com",
            "cookies": {
                "session": "abc123"
            },
            "created_at": "2026-01-01T00:00:00Z"
        }"#;
        std::fs::write(profiles_dir.join("legacy.com.json"), legacy_json).unwrap();

        // Load should migrate to encrypted
        let loaded = load_profile("legacy.com").unwrap().unwrap();
        assert_eq!(loaded.domain, "legacy.com");
        assert_eq!(
            loaded.cookies.get("session").map(|e| e.value.as_str()),
            Some("abc123")
        );

        // Verify it's now encrypted
        let path = profiles_dir.join("legacy.com.json");
        let data = std::fs::read(&path).unwrap();
        assert!(!is_plaintext_json(&data), "Profile should now be encrypted");

        // Restore original HOME
        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }
    }

    #[test]
    #[serial]
    fn test_list_and_revoke() {
        let dir = tempfile::tempdir().unwrap();
        let original_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", dir.path());

        let profile = CookieProfile {
            domain: "example.com".to_string(),
            cookies: HashMap::from([("session".to_string(), CookieEntry::new("abc".to_string()))]),
            created_at: None,
            notes: None,
        };
        store_profile(&profile).unwrap();

        let list = list_profiles().unwrap();
        assert!(list.contains(&"example.com".to_string()));

        assert!(revoke_profile("example.com").unwrap());
        assert!(!revoke_profile("example.com").unwrap());

        // Restore original HOME
        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }
    }

    #[test]
    fn test_cookie_expiry_status() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        assert_eq!(cookie_expiry_status(None), "no expiry");
        assert_eq!(cookie_expiry_status(Some(now - 100)), "✗ expired");
        assert_eq!(cookie_expiry_status(Some(now + 3600)), "⚠ expires soon"); // 1 hour
        assert_eq!(cookie_expiry_status(Some(now + 100000)), "✓ valid"); // > 24h
    }

    #[test]
    fn test_parse_cookie_string_with_ttl() {
        let cookies = parse_cookie_string_with_ttl("foo=bar; baz=qux", Some(3600));

        let foo = cookies.get("foo").unwrap();
        assert_eq!(foo.value, "bar");
        assert!(foo.expires_at.is_some());

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let exp = foo.expires_at.unwrap();
        assert!(exp > now && exp <= now + 3600);
    }
}
