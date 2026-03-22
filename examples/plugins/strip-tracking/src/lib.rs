//! Plasmate plugin: strip tracking and analytics elements from the SOM.
//!
//! Build with:
//!   cargo build --target wasm32-unknown-unknown --release
//!
//! Then load with:
//!   plasmate fetch https://example.com --plugin target/wasm32-unknown-unknown/release/strip_tracking.wasm
//!
//! This plugin hooks into `post_som` (hook bit 4) and removes elements whose
//! text or attributes contain known tracking/analytics patterns.

use serde::{Deserialize, Serialize};

// ---- Plugin ABI glue ----

static PLUGIN_NAME: &str = "strip-tracking";
static PLUGIN_VERSION: &str = "0.1.0";

/// Bump allocator for the host to write data into our memory.
static mut HEAP: usize = 0x10000; // start after 64 KiB of static data

/// Result buffer set by `on_hook`.
static mut RESULT_PTR: usize = 0;
static mut RESULT_LEN: usize = 0;

#[no_mangle]
pub extern "C" fn malloc(size: i32) -> i32 {
    unsafe {
        let ptr = HEAP;
        HEAP += size as usize;
        ptr as i32
    }
}

#[no_mangle]
pub extern "C" fn plugin_name_ptr() -> i32 {
    PLUGIN_NAME.as_ptr() as i32
}
#[no_mangle]
pub extern "C" fn plugin_name_len() -> i32 {
    PLUGIN_NAME.len() as i32
}
#[no_mangle]
pub extern "C" fn plugin_version_ptr() -> i32 {
    PLUGIN_VERSION.as_ptr() as i32
}
#[no_mangle]
pub extern "C" fn plugin_version_len() -> i32 {
    PLUGIN_VERSION.len() as i32
}

/// Hook bitmask: post_som = 4
#[no_mangle]
pub extern "C" fn get_hooks() -> i32 {
    4
}

#[no_mangle]
pub extern "C" fn get_result_ptr() -> i32 {
    unsafe { RESULT_PTR as i32 }
}
#[no_mangle]
pub extern "C" fn get_result_len() -> i32 {
    unsafe { RESULT_LEN as i32 }
}

#[no_mangle]
pub extern "C" fn on_hook(_hook_id: i32, ptr: i32, len: i32) -> i32 {
    let input = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };

    match process_som(input) {
        Ok(output) => {
            let boxed = output.into_boxed_slice();
            let out_ptr = boxed.as_ptr() as usize;
            let out_len = boxed.len();
            std::mem::forget(boxed);
            unsafe {
                RESULT_PTR = out_ptr;
                RESULT_LEN = out_len;
            }
            0 // success
        }
        Err(_) => {
            unsafe {
                RESULT_PTR = 0;
                RESULT_LEN = 0;
            }
            0 // return success but no modification
        }
    }
}

// ---- SOM types (minimal subset for deserialization) ----

#[derive(Debug, Serialize, Deserialize)]
struct Som {
    som_version: String,
    url: String,
    title: String,
    lang: String,
    regions: Vec<Region>,
    meta: SomMeta,
    #[serde(skip_serializing_if = "Option::is_none")]
    structured_data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SomMeta {
    html_bytes: usize,
    som_bytes: usize,
    element_count: usize,
    interactive_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct Region {
    id: String,
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    method: Option<String>,
    elements: Vec<Element>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Element {
    id: String,
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    actions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attrs: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    children: Option<Vec<Element>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hints: Option<Vec<String>>,
}

// ---- Tracking detection ----

/// Patterns that indicate tracking/analytics elements.
const TRACKING_PATTERNS: &[&str] = &[
    "google-analytics",
    "googletagmanager",
    "gtag",
    "ga-",
    "_ga",
    "facebook.com/tr",
    "fbevents",
    "pixel",
    "hotjar",
    "clarity.ms",
    "doubleclick",
    "adsbygoogle",
    "adsense",
    "tracking",
    "analytics",
    "intercom",
    "hubspot",
    "mixpanel",
    "segment.io",
    "segment.com",
    "amplitude",
    "newrelic",
    "sentry",
    "datadog",
];

fn is_tracking_element(element: &Element) -> bool {
    // Check text content
    if let Some(text) = &element.text {
        let lower = text.to_lowercase();
        for pattern in TRACKING_PATTERNS {
            if lower.contains(pattern) {
                return true;
            }
        }
    }

    // Check attributes (especially href, src)
    if let Some(attrs) = &element.attrs {
        let attrs_str = attrs.to_string().to_lowercase();
        for pattern in TRACKING_PATTERNS {
            if attrs_str.contains(pattern) {
                return true;
            }
        }
    }

    // Check hints
    if let Some(hints) = &element.hints {
        for hint in hints {
            let lower = hint.to_lowercase();
            if lower.contains("tracking") || lower.contains("analytics") {
                return true;
            }
        }
    }

    false
}

fn filter_elements(elements: Vec<Element>) -> Vec<Element> {
    elements
        .into_iter()
        .filter(|el| !is_tracking_element(el))
        .map(|mut el| {
            if let Some(children) = el.children.take() {
                let filtered = filter_elements(children);
                if !filtered.is_empty() {
                    el.children = Some(filtered);
                }
            }
            el
        })
        .collect()
}

fn process_som(input: &[u8]) -> Result<Vec<u8>, ()> {
    let mut som: Som = serde_json::from_slice(input).map_err(|_| ())?;

    let mut total_removed = 0usize;

    for region in &mut som.regions {
        let before = count_elements(&region.elements);
        region.elements = filter_elements(std::mem::take(&mut region.elements));
        let after = count_elements(&region.elements);
        total_removed += before - after;
    }

    // Remove empty regions
    som.regions.retain(|r| !r.elements.is_empty());

    // Update counts
    if total_removed > 0 {
        som.meta.element_count = som.meta.element_count.saturating_sub(total_removed);
    }

    serde_json::to_vec(&som).map_err(|_| ())
}

fn count_elements(elements: &[Element]) -> usize {
    let mut count = elements.len();
    for el in elements {
        if let Some(children) = &el.children {
            count += count_elements(children);
        }
    }
    count
}
