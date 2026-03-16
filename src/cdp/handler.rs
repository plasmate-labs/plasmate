//! CDP request dispatcher.
//!
//! Routes incoming CDP method calls to the appropriate domain handler.
//! Methods we don't implement are silently acknowledged (return empty success)
//! to avoid breaking Puppeteer/Playwright which send many setup calls.

use tracing::{debug, warn};

use super::domains;
use super::session::CdpTarget;
use super::types::*;

/// Process a CDP request, returning a response and any events to emit.
pub async fn handle_cdp_request(
    req: &CdpRequest,
    target: &mut CdpTarget,
) -> (CdpResponse, Vec<CdpEvent>) {
    let id = req.id;
    let method = req.method.as_str();
    let params = &req.params;

    debug!(method, id, "CDP request");

    let (response, events) = match method {
        // ---- Browser ----
        "Browser.getVersion" => (domains::browser_get_version(id), vec![]),
        "Browser.close" => (domains::browser_close(id), vec![]),

        // ---- Target ----
        "Target.getTargets" => (domains::target_get_targets(id, target), vec![]),
        "Target.createTarget" => (domains::target_create_target(id, target), vec![]),
        "Target.attachToTarget" => (domains::target_attach_to_target(id, target), vec![]),
        "Target.setDiscoverTargets" => (domains::target_set_discover_targets(id), vec![]),
        "Target.setAutoAttach" => (CdpResponse::success(id, serde_json::json!({})), vec![]),

        // ---- Page ----
        "Page.navigate" => domains::page_navigate(id, params, target).await,
        "Page.enable" => (domains::page_enable(id), vec![]),
        "Page.getFrameTree" => (domains::page_get_frame_tree(id, target), vec![]),
        "Page.setLifecycleEventsEnabled" => {
            (domains::page_set_lifecycle_events_enabled(id), vec![])
        }
        "Page.createIsolatedWorld" => (domains::page_create_isolated_world(id), vec![]),
        "Page.setInterceptFileChooserDialog" => {
            (CdpResponse::success(id, serde_json::json!({})), vec![])
        }
        "Page.addScriptToEvaluateOnNewDocument" => (
            CdpResponse::success(id, serde_json::json!({"identifier": "1"})),
            vec![],
        ),
        "Page.getNavigationHistory" => {
            let url = target.current_url.as_deref().unwrap_or("about:blank");
            (
                CdpResponse::success(
                    id,
                    serde_json::json!({
                        "currentIndex": 0,
                        "entries": [{"id": 0, "url": url, "title": "", "userTypedURL": url, "transitionType": "typed"}]
                    }),
                ),
                vec![],
            )
        }

        // ---- Runtime ----
        "Runtime.enable" => domains::runtime_enable(id, target),
        "Runtime.evaluate" => (domains::runtime_evaluate(id, params, target), vec![]),
        "Runtime.callFunctionOn" => {
            // Puppeteer uses this heavily for DOM manipulation
            let function = params
                .get("functionDeclaration")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            debug!("callFunctionOn: {}", &function[..function.len().min(80)]);
            (
                CdpResponse::success(
                    id,
                    serde_json::json!({
                        "result": {"type": "undefined"}
                    }),
                ),
                vec![],
            )
        }
        "Runtime.getProperties" => (
            CdpResponse::success(id, serde_json::json!({"result": []})),
            vec![],
        ),
        "Runtime.releaseObject" | "Runtime.releaseObjectGroup" => {
            (CdpResponse::success(id, serde_json::json!({})), vec![])
        }
        "Runtime.runIfWaitingForDebugger" => {
            (CdpResponse::success(id, serde_json::json!({})), vec![])
        }

        // ---- DOM ----
        "DOM.getDocument" => (domains::dom_get_document(id, target), vec![]),
        "DOM.querySelector" => (domains::dom_query_selector(id, params, target), vec![]),
        "DOM.querySelectorAll" => (domains::dom_query_selector_all(id, params, target), vec![]),
        "DOM.describeNode" => (domains::dom_describe_node(id, params, target), vec![]),
        "DOM.resolveNode" => (domains::dom_resolve_node(id, params, target), vec![]),
        "DOM.enable" => (CdpResponse::success(id, serde_json::json!({})), vec![]),
        "DOM.disable" => (CdpResponse::success(id, serde_json::json!({})), vec![]),
        "DOM.requestChildNodes" => (CdpResponse::success(id, serde_json::json!({})), vec![]),
        "DOM.setAttributeValue" => (CdpResponse::success(id, serde_json::json!({})), vec![]),
        "DOM.removeAttribute" => (CdpResponse::success(id, serde_json::json!({})), vec![]),

        // ---- Input ----
        "Input.dispatchMouseEvent" => domains::input_dispatch_mouse_event(id, params, target).await,
        "Input.dispatchKeyEvent" => (domains::input_dispatch_key_event(id, params), vec![]),

        // ---- Network ----
        "Network.enable" => (domains::network_enable(id), vec![]),
        "Network.disable" => (CdpResponse::success(id, serde_json::json!({})), vec![]),
        "Network.setExtraHTTPHeaders" => (
            domains::network_set_extra_http_headers(id, params, target),
            vec![],
        ),
        "Network.getCookies" => (domains::network_get_cookies(id), vec![]),
        "Network.setCookies" => (domains::network_set_cookies(id), vec![]),
        "Network.setRequestInterception" => {
            (CdpResponse::success(id, serde_json::json!({})), vec![])
        }

        // ---- Emulation ----
        "Emulation.setDeviceMetricsOverride" => {
            (domains::emulation_set_device_metrics_override(id), vec![])
        }
        "Emulation.setTouchEmulationEnabled" => {
            (domains::emulation_set_touch_emulation_enabled(id), vec![])
        }
        "Emulation.setUserAgentOverride" => {
            (CdpResponse::success(id, serde_json::json!({})), vec![])
        }
        "Emulation.setScrollbarsHidden" => {
            (CdpResponse::success(id, serde_json::json!({})), vec![])
        }

        // ---- Log / Performance / Security (ack and ignore) ----
        "Log.enable"
        | "Log.disable"
        | "Performance.enable"
        | "Performance.disable"
        | "Performance.getMetrics"
        | "Security.enable"
        | "Security.disable"
        | "Security.setIgnoreCertificateErrors"
        | "Fetch.enable"
        | "Fetch.disable"
        | "ServiceWorker.enable"
        | "ServiceWorker.disable"
        | "CSS.enable"
        | "CSS.disable"
        | "Overlay.enable"
        | "Overlay.disable"
        | "Accessibility.enable"
        | "Accessibility.disable"
        | "Inspector.enable"
        | "Inspector.disable"
        | "Debugger.enable"
        | "Debugger.disable"
        | "HeadlessExperimental.enable" => {
            (CdpResponse::success(id, serde_json::json!({})), vec![])
        }

        // ---- Plasmate custom domain ----
        "Plasmate.getSom" => (domains::plasmate_get_som(id, target), vec![]),
        "Plasmate.getStructuredData" => (domains::plasmate_get_structured_data(id, target), vec![]),
        "Plasmate.getInteractiveElements" => (
            domains::plasmate_get_interactive_elements(id, target),
            vec![],
        ),
        "Plasmate.getMarkdown" => (domains::plasmate_get_markdown(id, target), vec![]),

        // ---- Unknown: acknowledge to avoid breaking clients ----
        _ => {
            warn!("Unhandled CDP method: {}", method);
            (CdpResponse::success(id, serde_json::json!({})), vec![])
        }
    };

    // Attach session ID if the request had one
    let response = response.with_session(req.session_id.clone());
    let events: Vec<CdpEvent> = events
        .into_iter()
        .map(|mut e| {
            e.session_id = req.session_id.clone();
            e
        })
        .collect();

    (response, events)
}
