//! CDP (Chrome DevTools Protocol) compatibility layer.
//!
//! This module implements Plasmate's documented CDP subset for supported client
//! workflows. It is not full Puppeteer, Playwright, CDP, Chromium, or
//! web-platform compatibility. Under the hood, page processing goes through
//! the SOM pipeline.
//!
//! We also expose a custom `Plasmate` CDP domain with SOM-native commands,
//! similar to Lightpanda's `LP` domain but with full SOM support.
//!
//! Supported CDP domains:
//!   - Browser.*     (version, close)
//!   - Target.*      (getTargets, createTarget, attachToTarget, etc.)
//!   - Page.*        (navigate, enable, getFrameTree, lifecycleEvent)
//!   - Runtime.*     (evaluate, callFunctionOn, enable)
//!   - DOM.*         (getDocument, querySelector, querySelectorAll, resolveNode)
//!   - Input.*       (dispatchMouseEvent, dispatchKeyEvent)
//!   - Network.*     (enable, setCookies, getCookies, deleteCookies, clearBrowserCookies)
//!   - Fetch.*       (enable, disable, fulfillRequest, failRequest, continueRequest, continueResponse, getResponseBody)
//!   - Plasmate.*    (getSom, getStructuredData, getInteractiveElements, getMarkdown, getText, act)

pub mod cookies;
pub mod domains;
pub mod handler;
pub mod server;
pub mod session;
pub mod types;
