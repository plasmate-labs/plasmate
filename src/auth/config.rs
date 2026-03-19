//! Global auth profile configuration.
//!
//! Set once at startup, read by session constructors.

use std::sync::OnceLock;

static AUTH_PROFILES: OnceLock<Vec<String>> = OnceLock::new();

/// Set the global auth profiles (call once at startup).
pub fn set_profiles(profiles: Vec<String>) {
    let _ = AUTH_PROFILES.set(profiles);
}

/// Get the configured auth profiles.
pub fn profiles() -> &'static [String] {
    AUTH_PROFILES.get().map(|v| v.as_slice()).unwrap_or(&[])
}
