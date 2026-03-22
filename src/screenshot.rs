//! Screenshot capture module.
//!
//! Provides the interface for capturing visual representations of web pages.
//! Currently returns the SOM as structured data while a built-in layout/render
//! engine is under development.
//!
//! Design: pure library, zero runtime dependencies. No shelling out to Chrome,
//! no cloud services. When the renderer lands it will use the `image` crate to
//! rasterise a simplified layout of the SOM directly to PNG — like a built-in
//! wkhtmltoimage.

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
    #[error("Screenshot rendering is not yet implemented. Plasmate does not have a built-in layout engine yet. Use `plasmate fetch <url>` or `Plasmate.getSom` for structured content extraction.")]
    NotImplemented,
}

/// Attempt to capture a screenshot of a URL.
///
/// Currently returns [`ScreenshotError::NotImplemented`] — the built-in
/// rasteriser is not yet available. Callers should fall back to returning
/// the SOM as structured data.
pub fn capture_url(_url: &str, _opts: &ScreenshotOptions) -> Result<Vec<u8>, ScreenshotError> {
    Err(ScreenshotError::NotImplemented)
}

/// Attempt to capture a screenshot from HTML content.
///
/// Currently returns [`ScreenshotError::NotImplemented`].
pub fn capture_html(
    _html: &str,
    _base_url: &str,
    _opts: &ScreenshotOptions,
) -> Result<Vec<u8>, ScreenshotError> {
    Err(ScreenshotError::NotImplemented)
}

/// Build a structured fallback response when screenshot is unavailable.
///
/// Returns the SOM as JSON so callers still get useful data. This is the
/// honest alternative to faking a screenshot.
pub fn som_fallback(som: &crate::som::types::Som) -> serde_json::Value {
    let som_json = serde_json::to_value(som).unwrap_or(json!(null));
    json!({
        "error": "screenshot_not_implemented",
        "message": "Plasmate does not have a built-in layout engine yet. The page SOM is returned as structured data instead.",
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
    fn test_capture_url_not_implemented() {
        let opts = ScreenshotOptions::default();
        let result = capture_url("https://example.com", &opts);
        assert!(result.is_err());
        match result.unwrap_err() {
            ScreenshotError::NotImplemented => {} // expected
        }
    }

    #[test]
    fn test_capture_html_not_implemented() {
        let opts = ScreenshotOptions::default();
        let result = capture_html("<html></html>", "https://example.com", &opts);
        assert!(result.is_err());
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
        assert!(fallback["message"]
            .as_str()
            .unwrap()
            .contains("layout engine"));
        assert!(fallback["hint"]
            .as_str()
            .unwrap()
            .contains("Plasmate.getSom"));
    }
}
