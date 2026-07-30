//! Opt-in, privacy-safe measurements for page representations delivered to agents.
//!
//! Measurements are disabled unless `PLASMATE_MEASUREMENTS_PATH` is set. The
//! append-only JSONL file contains byte counts and a one-way URL hash, never
//! page content or a raw URL.

use fs2::FileExt;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(unix)]
use std::{
    fs::Permissions,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
};

pub const SCHEMA_VERSION: &str = "plasmate.measurement.v1";

#[derive(Debug, Serialize)]
pub struct DeliveryMeasurement<'a> {
    pub schema_version: &'static str,
    pub recorded_at_unix_ms: u64,
    pub source_id: &'a str,
    pub operation: &'a str,
    pub representation: &'a str,
    pub url_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<&'a str>,
    pub source_html_bytes: usize,
    pub delivered_bytes: usize,
    pub bytes_not_delivered: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_restored: Option<bool>,
}

/// Append one successful page-delivery measurement when measurement is enabled.
///
/// Measurement failures are deliberately non-fatal: browsing must continue
/// even if the local metrics path is unavailable.
pub fn record_delivery(
    operation: &str,
    representation: &str,
    url: &str,
    selector: Option<&str>,
    source_html_bytes: usize,
    delivered_text: &str,
    cache_restored: Option<bool>,
) {
    let Ok(path) = std::env::var("PLASMATE_MEASUREMENTS_PATH") else {
        return;
    };
    if path.trim().is_empty() {
        return;
    }

    let source_id =
        std::env::var("PLASMATE_SOURCE_ID").unwrap_or_else(|_| "unassigned".to_string());
    let measurement = DeliveryMeasurement {
        schema_version: SCHEMA_VERSION,
        recorded_at_unix_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
        source_id: &source_id,
        operation,
        representation,
        url_sha256: hash_url(url),
        selector,
        source_html_bytes,
        delivered_bytes: delivered_text.len(),
        bytes_not_delivered: source_html_bytes.saturating_sub(delivered_text.len()),
        cache_restored,
    };

    if let Err(error) = append_measurement(Path::new(&path), &measurement) {
        tracing::warn!(%error, "failed to append Plasmate measurement");
    }
}

fn hash_url(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    hex::encode(hasher.finalize())
}

fn append_measurement(path: &Path, measurement: &DeliveryMeasurement<'_>) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }

    let mut options = OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options.open(path)?;
    #[cfg(unix)]
    file.set_permissions(Permissions::from_mode(0o600))?;
    file.lock_exclusive()?;
    let write_result = (|| {
        serde_json::to_writer(&mut file, measurement).map_err(std::io::Error::other)?;
        file.write_all(b"\n")?;
        file.flush()
    })();
    let unlock_result = FileExt::unlock(&file);
    write_result.and(unlock_result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measurement_keeps_counts_but_not_the_raw_url_or_content() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("measurements.jsonl");
        let measurement = DeliveryMeasurement {
            schema_version: SCHEMA_VERSION,
            recorded_at_unix_ms: 1_785_000_000_000,
            source_id: "db",
            operation: "fetch_page",
            representation: "som",
            url_sha256: hash_url("https://example.com/private/path"),
            selector: Some("main"),
            source_html_bytes: 1_000,
            delivered_bytes: 200,
            bytes_not_delivered: 800,
            cache_restored: Some(false),
        };

        append_measurement(&path, &measurement).unwrap();
        let line = std::fs::read_to_string(path).unwrap();

        assert!(line.contains("\"source_html_bytes\":1000"));
        assert!(line.contains("\"delivered_bytes\":200"));
        assert!(line.contains("\"bytes_not_delivered\":800"));
        assert!(!line.contains("example.com"));
        assert!(!line.contains("private/path"));
    }
}
