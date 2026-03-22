//! CDP (Chrome DevTools Protocol) compatibility layer.
//!
//! This module implements enough of CDP that Puppeteer and Playwright can connect
//! to Plasmate and work. Under the hood, all page processing goes through
//! our SOM pipeline - agents get the same speed and token efficiency benefits
//! whether they connect via CDP or AWP.
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
//!   - Plasmate.*    (getSom, getStructuredData, getInteractiveElements, act)

pub mod cookies;
pub mod domains;
pub mod handler;
pub mod server;
pub mod session;
pub mod types;
