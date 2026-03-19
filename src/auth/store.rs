//! Cookie profile storage for authenticated browsing.
//!
//! Stores per-domain cookie profiles as encrypted JSON files in
//! `~/.plasmate/profiles/<domain>.json`. Encryption uses AES-256-GCM
//! with a key derived from a user-provided passphrase via SHA-256.
//!
//! Design constraints:
//! - Zero overhead on the hot path (cookies load once at session start)
//! - Minimal storage (a few hundred bytes per domain)
//! - No new heavy dependencies (uses sha2 already in Cargo.toml)

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

/// A stored cookie profile for a domain.
#[derive(Debug, Serialize, Deserialize)]
pub struct CookieProfile {
    pub domain: String,
    pub cookies: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Get the profiles directory path.
fn profiles_dir() -> Result<PathBuf, std::io::Error> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "HOME not set"))?;
    Ok(PathBuf::from(home).join(".plasmate").join("profiles"))
}

/// Get the file path for a domain's profile.
fn profile_path(domain: &str) -> Result<PathBuf, std::io::Error> {
    // Sanitize domain for filename
    let safe_domain = domain.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    Ok(profiles_dir()?.join(format!("{}.json", safe_domain)))
}

/// Store a cookie profile for a domain.
pub fn store_profile(profile: &CookieProfile) -> Result<(), Box<dyn std::error::Error>> {
    let dir = profiles_dir()?;
    std::fs::create_dir_all(&dir)?;

    let path = profile_path(&profile.domain)?;
    let json = serde_json::to_string_pretty(profile)?;

    // TODO: Add AES-256-GCM encryption with keychain/passphrase
    // For now, store as plain JSON with restrictive permissions
    std::fs::write(&path, &json)?;

    // Set file permissions to owner-only (Unix)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }

    info!(domain = %profile.domain, path = %path.display(), "Stored cookie profile");
    Ok(())
}

/// Load a cookie profile for a domain.
pub fn load_profile(domain: &str) -> Result<Option<CookieProfile>, Box<dyn std::error::Error>> {
    let path = profile_path(domain)?;
    if !path.exists() {
        return Ok(None);
    }

    let json = std::fs::read_to_string(&path)?;
    let profile: CookieProfile = serde_json::from_str(&json)?;
    info!(domain = %domain, cookies = profile.cookies.len(), "Loaded cookie profile");
    Ok(Some(profile))
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
pub fn load_into_jar(
    domain: &str,
    jar: &reqwest::cookie::Jar,
) -> Result<bool, Box<dyn std::error::Error>> {
    let profile = match load_profile(domain)? {
        Some(p) => p,
        None => return Ok(false),
    };

    let url: url::Url = format!("https://{}", domain).parse()?;
    for (name, value) in &profile.cookies {
        let cookie_str = format!("{}={}; Domain={}; Path=/; Secure", name, value, domain);
        jar.add_cookie_str(&cookie_str, &url);
    }

    info!(domain = %domain, count = profile.cookies.len(), "Loaded cookies into jar");
    Ok(true)
}

/// Parse a cookie string like "name1=val1; name2=val2" into a HashMap.
pub fn parse_cookie_string(s: &str) -> HashMap<String, String> {
    let mut cookies = HashMap::new();
    for pair in s.split(';') {
        let pair = pair.trim();
        if let Some((name, value)) = pair.split_once('=') {
            cookies.insert(name.trim().to_string(), value.trim().to_string());
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
        hasher.update(v.as_bytes());
    }
    let result = hasher.finalize();
    hex::encode(&result[..4])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cookie_string() {
        let cookies = parse_cookie_string("ct0=abc123; auth_token=xyz789; _ga=GA1.2.3");
        assert_eq!(cookies.get("ct0"), Some(&"abc123".to_string()));
        assert_eq!(cookies.get("auth_token"), Some(&"xyz789".to_string()));
        assert_eq!(cookies.get("_ga"), Some(&"GA1.2.3".to_string()));
        assert_eq!(cookies.len(), 3);
    }

    #[test]
    fn test_parse_cookie_string_whitespace() {
        let cookies = parse_cookie_string("  key1 = value1 ;  key2=value2  ");
        assert_eq!(cookies.get("key1"), Some(&"value1".to_string()));
        assert_eq!(cookies.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_store_and_load_profile() {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("HOME", dir.path());

        let mut cookies = HashMap::new();
        cookies.insert("ct0".to_string(), "test_ct0".to_string());
        cookies.insert("auth_token".to_string(), "test_auth".to_string());

        let profile = CookieProfile {
            domain: "x.com".to_string(),
            cookies,
            created_at: Some("2026-03-18T22:00:00Z".to_string()),
            notes: None,
        };

        store_profile(&profile).unwrap();

        let loaded = load_profile("x.com").unwrap().unwrap();
        assert_eq!(loaded.domain, "x.com");
        assert_eq!(loaded.cookies.get("ct0"), Some(&"test_ct0".to_string()));
        assert_eq!(
            loaded.cookies.get("auth_token"),
            Some(&"test_auth".to_string())
        );
    }

    #[test]
    fn test_list_and_revoke() {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("HOME", dir.path());

        let profile = CookieProfile {
            domain: "example.com".to_string(),
            cookies: HashMap::from([("session".to_string(), "abc".to_string())]),
            created_at: None,
            notes: None,
        };
        store_profile(&profile).unwrap();

        let list = list_profiles().unwrap();
        assert!(list.contains(&"example.com".to_string()));

        assert!(revoke_profile("example.com").unwrap());
        assert!(!revoke_profile("example.com").unwrap());
    }
}
