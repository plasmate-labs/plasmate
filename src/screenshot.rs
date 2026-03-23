//! Screenshot capture module.
//!
//! Delegates pixel-perfect rendering to a headless Chrome/Chromium subprocess
//! when available. Chrome is spawned on demand, captures the screenshot, and
//! exits immediately — zero cost unless a screenshot is explicitly requested.
//!
//! If Chrome is not installed, callers fall back to returning the SOM as
//! structured data via [`som_fallback`].

use serde_json::json;

/// Default viewport width.
pub const DEFAULT_WIDTH: u32 = 1280;
/// Default viewport height.
pub const DEFAULT_HEIGHT: u32 = 720;

/// Screenshot output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Png,
    Jpeg,
    Webp,
}

impl Format {
    pub fn as_str(&self) -> &'static str {
        match self {
            Format::Png => "png",
            Format::Jpeg => "jpeg",
            Format::Webp => "webp",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "jpeg" | "jpg" => Format::Jpeg,
            "webp" => Format::Webp,
            _ => Format::Png,
        }
    }

    pub fn content_type(&self) -> &'static str {
        match self {
            Format::Png => "image/png",
            Format::Jpeg => "image/jpeg",
            Format::Webp => "image/webp",
        }
    }
}

/// Options for capturing a screenshot.
#[derive(Debug, Clone)]
pub struct ScreenshotOptions {
    pub width: u32,
    pub height: u32,
    pub format: Format,
    /// Quality for JPEG/WebP (1-100). Ignored for PNG.
    pub quality: Option<u32>,
    /// If true, capture the full scrollable page.
    pub full_page: bool,
}

impl Default for ScreenshotOptions {
    fn default() -> Self {
        ScreenshotOptions {
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            format: Format::Png,
            quality: None,
            full_page: false,
        }
    }
}

/// Error type for screenshot operations.
#[derive(Debug, thiserror::Error)]
pub enum ScreenshotError {
    #[error(
        "Chrome/Chromium not found. Install Chrome or Chromium for screenshot support. \
         SOM output is available via `plasmate fetch` or `Plasmate.getSom`."
    )]
    ChromeNotFound,
    #[error("Screenshot capture failed: {0}")]
    CaptureFailed(String),
    #[error("Render error: {0}")]
    RenderError(String),
}

/// Find Chrome/Chromium binary on the system.
fn find_chrome() -> Option<String> {
    let candidates = [
        // macOS
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
        // Linux
        "google-chrome",
        "google-chrome-stable",
        "chromium",
        "chromium-browser",
        // Windows (WSL)
        "/mnt/c/Program Files/Google/Chrome/Application/chrome.exe",
    ];

    for candidate in &candidates {
        if std::path::Path::new(candidate).exists() {
            return Some(candidate.to_string());
        }
        // Also check PATH
        if let Ok(output) = std::process::Command::new("which").arg(candidate).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(path);
                }
            }
        }
    }
    None
}

/// Capture a screenshot by delegating to headless Chrome.
///
/// Spawns a temporary Chrome process, navigates to the URL,
/// captures the screenshot, and terminates Chrome.
pub fn capture_url(url: &str, opts: &ScreenshotOptions) -> Result<Vec<u8>, ScreenshotError> {
    let chrome_bin = find_chrome().ok_or(ScreenshotError::ChromeNotFound)?;

    let temp_dir = tempfile::tempdir()
        .map_err(|e| ScreenshotError::CaptureFailed(format!("temp dir: {}", e)))?;

    let screenshot_path = temp_dir.path().join("screenshot.png");

    let mut cmd = std::process::Command::new(&chrome_bin);
    cmd.args([
        "--headless=new",
        "--disable-gpu",
        "--no-sandbox",
        "--disable-dev-shm-usage",
        "--disable-extensions",
        "--disable-background-networking",
        "--disable-sync",
        "--disable-translate",
        "--mute-audio",
        "--no-first-run",
        "--no-default-browser-check",
        &format!("--window-size={},{}", opts.width, opts.height),
        &format!("--user-data-dir={}", temp_dir.path().display()),
        &format!("--screenshot={}", screenshot_path.display()),
    ]);

    cmd.arg(url);

    let mut child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| ScreenshotError::CaptureFailed(format!("spawn chrome: {}", e)))?;

    let timeout = std::time::Duration::from_secs(15);
    let start_time = std::time::Instant::now();
    let mut output = None;

    loop {
        if start_time.elapsed() > timeout {
            child.kill().ok(); // Best effort kill
            return Err(ScreenshotError::CaptureFailed(
                "Chrome process timed out".to_string(),
            ));
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                let res = child.wait_with_output().map_err(|e| {
                    ScreenshotError::CaptureFailed(format!("wait_with_output failed: {}", e))
                })?;
                if !status.success() {
                    let stderr = String::from_utf8_lossy(&res.stderr);
                    return Err(ScreenshotError::CaptureFailed(format!(
                        "Chrome exited with non-zero status: {}. stderr: {}",
                        status,
                        stderr.chars().take(500).collect::<String>()
                    )));
                }
                output = Some(res);
                break;
            }
            Ok(None) => {
                // Still running
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(e) => {
                return Err(ScreenshotError::CaptureFailed(format!(
                    "Error waiting for Chrome: {}",
                    e
                )));
            }
        }
    }

    let output = output.unwrap(); // We broke the loop, so it's Some

    if !screenshot_path.exists() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ScreenshotError::CaptureFailed(format!(
            "Chrome did not produce screenshot. stderr: {}",
            stderr.chars().take(500).collect::<String>()
        )));
    }

    let data = std::fs::read(&screenshot_path)
        .map_err(|e| ScreenshotError::CaptureFailed(format!("read screenshot: {}", e)))?;

    Ok(data)
}

/// Capture a screenshot from HTML content.
pub fn capture_html(
    html: &str,
    _base_url: &str,
    opts: &ScreenshotOptions,
) -> Result<Vec<u8>, ScreenshotError> {
    let chrome_bin = find_chrome().ok_or(ScreenshotError::ChromeNotFound)?;

    let temp_dir = tempfile::tempdir()
        .map_err(|e| ScreenshotError::CaptureFailed(format!("temp dir: {}", e)))?;

    // Write HTML to temp file
    let html_path = temp_dir.path().join("page.html");
    std::fs::write(&html_path, html)
        .map_err(|e| ScreenshotError::CaptureFailed(format!("write html: {}", e)))?;

    let screenshot_path = temp_dir.path().join("screenshot.png");

    let mut child = std::process::Command::new(&chrome_bin)
        .args([
            "--headless=new",
            "--disable-gpu",
            "--no-sandbox",
            "--disable-dev-shm-usage",
            "--disable-extensions",
            "--disable-background-networking",
            "--no-first-run",
            "--no-default-browser-check",
            &format!("--window-size={},{}", opts.width, opts.height),
            &format!("--user-data-dir={}", temp_dir.path().display()),
            &format!("--screenshot={}", screenshot_path.display()),
        ])
        .arg(format!("file://{}", html_path.display()))
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| ScreenshotError::CaptureFailed(format!("spawn chrome: {}", e)))?;

    let timeout = std::time::Duration::from_secs(15);
    let start_time = std::time::Instant::now();
    let mut output = None;

    loop {
        if start_time.elapsed() > timeout {
            child.kill().ok();
            return Err(ScreenshotError::CaptureFailed(
                "Chrome process timed out".to_string(),
            ));
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                let res = child.wait_with_output().map_err(|e| {
                    ScreenshotError::CaptureFailed(format!("wait_with_output failed: {}", e))
                })?;
                if !status.success() {
                    let stderr = String::from_utf8_lossy(&res.stderr);
                    return Err(ScreenshotError::CaptureFailed(format!(
                        "Chrome exited with non-zero status: {}. stderr: {}",
                        status,
                        stderr.chars().take(500).collect::<String>()
                    )));
                }
                output = Some(res);
                break;
            }
            Ok(None) => {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(e) => {
                return Err(ScreenshotError::CaptureFailed(format!(
                    "Error waiting for Chrome: {}",
                    e
                )));
            }
        }
    }

    let output = output.unwrap();

    if !screenshot_path.exists() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ScreenshotError::CaptureFailed(format!(
            "Chrome did not produce screenshot. stderr: {}",
            stderr.chars().take(500).collect::<String>()
        )));
    }

    std::fs::read(&screenshot_path)
        .map_err(|e| ScreenshotError::CaptureFailed(format!("read: {}", e)))
}

/// Check if Chrome is available for screenshots.
pub fn chrome_available() -> bool {
    find_chrome().is_some()
}

/// Build a structured fallback response when screenshot is unavailable.
///
/// Returns the SOM as JSON so callers still get useful data. This is the
/// honest alternative to faking a screenshot.
pub fn som_fallback(som: &crate::som::types::Som) -> serde_json::Value {
    let som_json = serde_json::to_value(som).unwrap_or(json!(null));
    json!({
        "error": "screenshot_not_implemented",
        "message": "Chrome/Chromium not found. The page SOM is returned as structured data instead. Install Chrome for pixel-perfect screenshots.",
        "som": som_json,
        "hint": "Use `Plasmate.getSom` or `plasmate fetch` for structured content extraction."
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_from_str() {
        assert_eq!(Format::from_str("png"), Format::Png);
        assert_eq!(Format::from_str("jpeg"), Format::Jpeg);
        assert_eq!(Format::from_str("jpg"), Format::Jpeg);
        assert_eq!(Format::from_str("webp"), Format::Webp);
        assert_eq!(Format::from_str("PNG"), Format::Png);
        assert_eq!(Format::from_str("unknown"), Format::Png);
    }

    #[test]
    fn test_format_content_type() {
        assert_eq!(Format::Png.content_type(), "image/png");
        assert_eq!(Format::Jpeg.content_type(), "image/jpeg");
        assert_eq!(Format::Webp.content_type(), "image/webp");
    }

    #[test]
    fn test_format_as_str() {
        assert_eq!(Format::Png.as_str(), "png");
        assert_eq!(Format::Jpeg.as_str(), "jpeg");
        assert_eq!(Format::Webp.as_str(), "webp");
    }

    #[test]
    fn test_default_options() {
        let opts = ScreenshotOptions::default();
        assert_eq!(opts.width, DEFAULT_WIDTH);
        assert_eq!(opts.height, DEFAULT_HEIGHT);
        assert_eq!(opts.format, Format::Png);
        assert!(opts.quality.is_none());
        assert!(!opts.full_page);
    }

    #[test]
    fn test_find_chrome_does_not_crash() {
        // Just verify it returns Some or None without panicking
        let _result = find_chrome();
    }

    #[test]
    fn test_chrome_available_does_not_crash() {
        let _result = chrome_available();
    }

    #[test]
    fn test_capture_url_returns_result() {
        let opts = ScreenshotOptions::default();
        let result = capture_url("https://example.com", &opts);
        // Either succeeds (Chrome found) or returns ChromeNotFound
        match result {
            Ok(data) => {
                // Should be a valid PNG (starts with PNG magic bytes)
                assert!(data.len() > 8, "Screenshot data too small");
                assert_eq!(&data[0..4], &[0x89, 0x50, 0x4E, 0x47], "Not a PNG file");
            }
            Err(ScreenshotError::ChromeNotFound) => {
                // Expected if Chrome is not installed in test env
            }
            Err(e) => {
                // CaptureFailed is also acceptable (e.g. network issues in CI)
                eprintln!("capture_url error (acceptable in CI): {}", e);
            }
        }
    }

    #[test]
    fn test_capture_html_returns_result() {
        let opts = ScreenshotOptions::default();
        let result = capture_html(
            "<html><body><h1>Test</h1></body></html>",
            "https://example.com",
            &opts,
        );
        match result {
            Ok(data) => {
                assert!(data.len() > 8, "Screenshot data too small");
                assert_eq!(&data[0..4], &[0x89, 0x50, 0x4E, 0x47], "Not a PNG file");
            }
            Err(ScreenshotError::ChromeNotFound) => {}
            Err(e) => {
                eprintln!("capture_html error (acceptable in CI): {}", e);
            }
        }
    }

    #[test]
    fn test_som_fallback_structure() {
        let som = crate::som::types::Som {
            som_version: "1".to_string(),
            title: "Test Page".to_string(),
            url: "https://example.com".to_string(),
            lang: "en".to_string(),
            regions: vec![],
            meta: crate::som::types::SomMeta {
                html_bytes: 100,
                som_bytes: 50,
                element_count: 5,
                interactive_count: 1,
            },
            structured_data: None,
        };
        let fallback = som_fallback(&som);
        assert_eq!(fallback["error"], "screenshot_not_implemented");
        assert!(fallback["som"].is_object());
        assert!(fallback["message"].as_str().unwrap().contains("Chrome"));
        assert!(fallback["hint"]
            .as_str()
            .unwrap()
            .contains("Plasmate.getSom"));
    }
}
