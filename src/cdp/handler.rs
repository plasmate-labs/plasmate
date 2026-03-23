//! CDP request dispatcher.
//!
//! Routes incoming CDP method calls to the appropriate domain handler.
//! Methods we don't implement are silently acknowledged (return empty success)
//! to avoid breaking Puppeteer/Playwright which send many setup calls.

use tracing::{debug, error, warn};

use super::domains;
use super::session::CdpTarget;
use super::types::*;

/// Process a CDP request, returning a response and any events to emit.
pub async fn handle_cdp_request(
    req: &CdpRequest,
    target: &mut CdpTarget,
) -> (CdpResponse, Vec<CdpEvent>) {
    match handle_cdp_request_inner(req, target).await {
        Ok(result) => result,
        Err(msg) => {
            error!(method = %req.method, id = req.id, error = %msg, "CDP handler panic");
            let response = CdpResponse::error(req.id, CDP_ERR_SERVER, &msg)
                .with_session(req.session_id.clone());
            (response, vec![])
        }
    }
}

async fn handle_cdp_request_inner(
    req: &CdpRequest,
    target: &mut CdpTarget,
) -> Result<(CdpResponse, Vec<CdpEvent>), String> {
    let id = req.id;
    let method = req.method.as_str();
    let params = &req.params;

    debug!(method, id, session_id = ?req.session_id, "CDP request");

    let (response, events) = match method {
        // ---- Browser ----
        "Browser.getVersion" => (domains::browser_get_version(id), vec![]),
        "Browser.close" => (domains::browser_close(id), vec![]),

        // ---- Target ----
        "Target.getTargets" => (domains::target_get_targets(id, target), vec![]),
        "Target.createBrowserContext" => (
            CdpResponse::success(
                id,
                serde_json::json!({
                    "browserContextId": "default",
                }),
            ),
            vec![],
        ),
        "Target.createTarget" => {
            // Generate fresh target + session IDs
            let new_target_id = target.create_child_target();
            let new_session_id = target.session_id.clone();
            // Emit targetCreated + attachedToTarget (browser-level events,
            // not wrapped with sessionId because they start with "Target.")
            let events = vec![
                CdpEvent::new(
                    "Target.targetCreated",
                    serde_json::json!({
                        "targetInfo": {
                            "targetId": new_target_id,
                            "type": "page",
                            "title": "",
                            "url": "about:blank",
                            "attached": true,
                            "browserContextId": "default",
                        }
                    }),
                ),
                CdpEvent::new(
                    "Target.attachedToTarget",
                    serde_json::json!({
                        "sessionId": new_session_id,
                        "targetInfo": {
                            "targetId": new_target_id,
                            "type": "page",
                            "title": "",
                            "url": "about:blank",
                            "attached": true,
                            "browserContextId": "default",
                        },
                        "waitingForDebugger": false,
                    }),
                ),
            ];
            (
                CdpResponse::success(id, serde_json::json!({"targetId": new_target_id})),
                events,
            )
        }
        "Target.attachToTarget" => {
            let events = vec![CdpEvent::new(
                "Target.attachedToTarget",
                serde_json::json!({
                    "sessionId": target.session_id,
                    "targetInfo": {
                        "targetId": target.target_id,
                        "type": "page",
                        "title": "",
                        "url": target.current_url.as_deref().unwrap_or("about:blank"),
                        "attached": true,
                        "browserContextId": "default",
                    },
                    "waitingForDebugger": false,
                }),
            )];
            (domains::target_attach_to_target(id, target), events)
        }
        "Target.setDiscoverTargets" => (domains::target_set_discover_targets(id), vec![]),
        "Target.getBrowserContexts" => (
            CdpResponse::success(
                id,
                serde_json::json!({
                    "browserContextIds": []
                }),
            ),
            vec![],
        ),
        "Target.setAutoAttach" => {
            let auto_attach = params
                .get("autoAttach")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let flatten = params
                .get("flatten")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let mut events = vec![];

            // Only emit attach events for browser-level calls (no sessionId) AND
            // only if we haven't already configured auto-attach (prevents infinite loop).
            // Session-scoped setAutoAttach (from child pages) returns empty -
            // there are no child targets to discover (we don't have iframes/workers).
            if auto_attach && flatten && req.session_id.is_none() && !target.auto_attach_configured
            {
                // Mark as configured to prevent re-emitting on subsequent calls
                target.auto_attach_configured = true;

                // First call: attach the default target
                events.push(CdpEvent::new(
                    "Target.targetCreated",
                    serde_json::json!({
                        "targetInfo": {
                            "targetId": target.target_id,
                            "type": "page",
                            "title": "",
                            "url": target.current_url.as_deref().unwrap_or("about:blank"),
                            "attached": true,
                            "browserContextId": "default",
                        }
                    }),
                ));
                events.push(CdpEvent::new(
                    "Target.attachedToTarget",
                    serde_json::json!({
                        "sessionId": target.session_id,
                        "targetInfo": {
                            "targetId": target.target_id,
                            "type": "page",
                            "title": "",
                            "url": target.current_url.as_deref().unwrap_or("about:blank"),
                            "attached": true,
                            "browserContextId": "default",
                        },
                        "waitingForDebugger": false,
                    }),
                ));
            }
            (CdpResponse::success(id, serde_json::json!({})), events)
        }
        "Target.disposeBrowserContext" => (CdpResponse::success(id, serde_json::json!({})), vec![]),

        // ---- Page ----
        "Page.navigate" => domains::page_navigate(id, params, target).await,
        "Page.setDocumentContent" => domains::page_set_content(id, params, target).await,
        "Page.getResourceContent" => (domains::page_get_content(id, target), vec![]),
        "Page.enable" => (domains::page_enable(id), vec![]),
        "Page.getFrameTree" => (domains::page_get_frame_tree(id, target), vec![]),
        "Page.setLifecycleEventsEnabled" => {
            (domains::page_set_lifecycle_events_enabled(id), vec![])
        }
        "Page.createIsolatedWorld" => {
            let world_name = params
                .get("worldName")
                .and_then(|v| v.as_str())
                .unwrap_or("__puppeteer_utility_world__")
                .to_string();
            let ctx_id = 3; // isolated world context
            let events = vec![CdpEvent::new(
                "Runtime.executionContextCreated",
                serde_json::json!({
                    "context": {
                        "id": ctx_id,
                        "origin": target.current_url.as_deref().unwrap_or("about:blank"),
                        "name": world_name,
                        "uniqueId": format!("iso-{}", ctx_id),
                        "auxData": {
                            "isDefault": false,
                            "type": "isolated",
                            "frameId": target.frame_id,
                        }
                    }
                }),
            )];
            (
                CdpResponse::success(id, serde_json::json!({"executionContextId": ctx_id})),
                events,
            )
        }
        "Page.captureScreenshot" => domains::page_capture_screenshot(id, params, target),
        "Page.setInterceptFileChooserDialog" => {
            (CdpResponse::success(id, serde_json::json!({})), vec![])
        }
        "Page.addScriptToEvaluateOnNewDocument" => {
            let source = params.get("source").and_then(|v| v.as_str()).unwrap_or("");
            target.next_script_id += 1;
            let identifier = format!("{}", target.next_script_id);
            target
                .scripts_on_new_document
                .push((identifier.clone(), source.to_string()));
            (
                CdpResponse::success(id, serde_json::json!({"identifier": identifier})),
                vec![],
            )
        }
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
            let function = params
                .get("functionDeclaration")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let return_by_value = params
                .get("returnByValue")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let arguments: Vec<serde_json::Value> = params
                .get("arguments")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .map(|a| {
                            // CDP passes arguments as {value: ...} objects
                            a.get("value").cloned().unwrap_or(a.clone())
                        })
                        .collect()
                })
                .unwrap_or_default();

            debug!("callFunctionOn: {}", &function[..function.len().min(500)]);

            // Use real V8 execution if we have effective HTML
            let result = if target.effective_html.is_some() {
                match target.call_function_on(function, &arguments) {
                    Ok(value) => domains::value_to_cdp_result(&value, return_by_value),
                    Err(e) => {
                        debug!("callFunctionOn error: {}", e);
                        serde_json::json!({"type": "undefined"})
                    }
                }
            } else {
                // Fallback for when no page is loaded
                if function.contains("document.title") {
                    let title = target
                        .current_som
                        .as_ref()
                        .map(|s| s.title.clone())
                        .unwrap_or_default();
                    serde_json::json!({"type": "string", "value": title})
                } else if function.contains("outerHTML")
                    || function.contains("document.documentElement")
                {
                    let html = target.current_html.as_deref().unwrap_or("<html></html>");
                    serde_json::json!({"type": "string", "value": html})
                } else if return_by_value {
                    serde_json::json!({"type": "undefined"})
                } else {
                    serde_json::json!({"type": "object", "objectId": "eval-result"})
                }
            };

            (
                CdpResponse::success(id, serde_json::json!({"result": result})),
                vec![],
            )
        }
        "Runtime.getProperties" => (
            CdpResponse::success(
                id,
                serde_json::json!({
                    "result": [],
                    "internalProperties": [],
                    "privateProperties": []
                }),
            ),
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
        "DOM.getBoxModel" => (domains::dom_get_box_model(id, params, target), vec![]),
        "DOM.enable" => (CdpResponse::success(id, serde_json::json!({})), vec![]),
        "DOM.disable" => (CdpResponse::success(id, serde_json::json!({})), vec![]),
        "DOM.requestChildNodes" => (CdpResponse::success(id, serde_json::json!({})), vec![]),
        "DOM.setAttributeValue" => (CdpResponse::success(id, serde_json::json!({})), vec![]),
        "DOM.removeAttribute" => (CdpResponse::success(id, serde_json::json!({})), vec![]),

        // ---- Input ----
        "Input.dispatchMouseEvent" => domains::input_dispatch_mouse_event(id, params, target).await,
        "Input.dispatchKeyEvent" => (domains::input_dispatch_key_event(id, params), vec![]),
        "Input.insertText" => (domains::input_insert_text(id, params, target), vec![]),

        // ---- Network ----
        "Network.enable" => (domains::network_enable(id), vec![]),
        "Network.disable" => (CdpResponse::success(id, serde_json::json!({})), vec![]),
        "Network.setExtraHTTPHeaders" => (
            domains::network_set_extra_http_headers(id, params, target),
            vec![],
        ),
        "Network.getCookies" => (domains::network_get_cookies(id, params, target), vec![]),
        "Network.getAllCookies" => (domains::network_get_all_cookies(id, target), vec![]),
        "Network.setCookies" => (domains::network_set_cookies(id, params, target), vec![]),
        "Network.setCookie" => (domains::network_set_cookie(id, params, target), vec![]),
        "Network.deleteCookies" => (domains::network_delete_cookies(id, params, target), vec![]),
        "Network.clearBrowserCookies" => {
            (domains::network_clear_browser_cookies(id, target), vec![])
        }
        "Network.setRequestInterception" => {
            // Legacy API — forward to Fetch.enable for compatibility
            (domains::fetch_enable(id, params, target), vec![])
        }

        // ---- Fetch (network interception) ----
        "Fetch.enable" => (domains::fetch_enable(id, params, target), vec![]),
        "Fetch.disable" => (domains::fetch_disable(id, target), vec![]),
        "Fetch.fulfillRequest" => (domains::fetch_fulfill_request(id, params, target), vec![]),
        "Fetch.failRequest" => (domains::fetch_fail_request(id, params, target), vec![]),
        "Fetch.continueRequest" => (domains::fetch_continue_request(id, params, target), vec![]),
        "Fetch.continueResponse" => (domains::fetch_continue_response(id, params, target), vec![]),
        "Fetch.getResponseBody" => (domains::fetch_get_response_body(id, params, target), vec![]),

        // ---- Emulation ----
        "Emulation.setDeviceMetricsOverride" => (
            domains::emulation_set_device_metrics_override(id, params, target),
            vec![],
        ),
        "Emulation.setTouchEmulationEnabled" => {
            (domains::emulation_set_touch_emulation_enabled(id), vec![])
        }
        "Emulation.setUserAgentOverride" => {
            if let Some(ua) = params.get("userAgent").and_then(|v| v.as_str()) {
                target.user_agent = ua.to_string();
                // Also set as extra header so it's used in subsequent fetches
                target
                    .extra_headers
                    .insert("User-Agent".to_string(), ua.to_string());
            }
            (CdpResponse::success(id, serde_json::json!({})), vec![])
        }
        "Emulation.setScrollbarsHidden" => {
            (CdpResponse::success(id, serde_json::json!({})), vec![])
        }

        // ---- Extra Target methods ----
        "Target.activateTarget" | "Target.detachFromTarget" => {
            (CdpResponse::success(id, serde_json::json!({})), vec![])
        }
        "Target.closeTarget" => {
            // Extract target ID from params, or use current target
            let close_target_id = params
                .get("targetId")
                .and_then(|v| v.as_str())
                .unwrap_or(&target.target_id)
                .to_string();

            // Get the session ID for this target
            let session_for_target = target.session_id.clone();

            // Emit both detachedFromTarget and targetDestroyed events
            // Puppeteer waits for detachedFromTarget when closing pages
            let events = vec![
                CdpEvent::new(
                    "Target.detachedFromTarget",
                    serde_json::json!({
                        "sessionId": session_for_target,
                        "targetId": close_target_id,
                    }),
                ),
                CdpEvent::new(
                    "Target.targetDestroyed",
                    serde_json::json!({
                        "targetId": close_target_id,
                    }),
                ),
            ];
            (
                CdpResponse::success(id, serde_json::json!({"success": true})),
                events,
            )
        }
        "Target.getTargetInfo" => (
            CdpResponse::success(
                id,
                serde_json::json!({
                    "targetInfo": {
                        "targetId": target.target_id,
                        "type": "page",
                        "title": target.current_som.as_ref().map(|s| s.title.as_str()).unwrap_or(""),
                        "url": target.current_url.as_deref().unwrap_or("about:blank"),
                        "attached": true,
                        "browserContextId": "default",
                    }
                }),
            ),
            vec![],
        ),

        // ---- Extra Page methods ----
        "Page.getLayoutMetrics" => {
            let w = target.viewport_width;
            let h = target.viewport_height;
            (
                CdpResponse::success(
                    id,
                    serde_json::json!({
                        "layoutViewport": {"pageX": 0, "pageY": 0, "clientWidth": w, "clientHeight": h},
                        "visualViewport": {"offsetX": 0, "offsetY": 0, "pageX": 0, "pageY": 0, "clientWidth": w, "clientHeight": h, "scale": target.device_scale_factor},
                        "contentSize": {"x": 0, "y": 0, "width": w, "height": h},
                        "cssLayoutViewport": {"clientWidth": w, "clientHeight": h, "pageX": 0, "pageY": 0},
                        "cssVisualViewport": {"clientWidth": w, "clientHeight": h, "offsetX": 0, "offsetY": 0, "pageX": 0, "pageY": 0, "zoom": target.device_scale_factor},
                        "cssContentSize": {"x": 0, "y": 0, "width": w, "height": h}
                    }),
                ),
                vec![],
            )
        }
        "Page.removeScriptToEvaluateOnNewDocument" => {
            let identifier = params
                .get("identifier")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            target
                .scripts_on_new_document
                .retain(|(id, _)| id != identifier);
            (CdpResponse::success(id, serde_json::json!({})), vec![])
        }
        "Page.bringToFront" | "Page.stopLoading" | "Page.close" | "Page.setBypassCSP" => {
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
        | "ServiceWorker.enable"
        | "ServiceWorker.disable"
        | "CSS.enable"
        | "CSS.disable"
        | "Overlay.enable"
        | "Overlay.disable"
        | "Accessibility.enable"
        | "Accessibility.disable" => (CdpResponse::success(id, serde_json::json!({})), vec![]),
        "Accessibility.getFullAXTree" => {
            (domains::accessibility_get_full_ax_tree(id, target), vec![])
        }
        "Inspector.enable"
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
        "Plasmate.listPlugins" => (domains::plasmate_list_plugins(id, target).await, vec![]),

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
            // Target-level events are always browser-scoped (no sessionId wrapper).
            // All other events inherit the request's sessionId.
            if !e.method.starts_with("Target.") {
                e.session_id = req.session_id.clone();
            }
            e
        })
        .collect();

    Ok((response, events))
}
