//! V8-backed JavaScript runtime for Plasmate.
//!
//! Each page gets its own V8 Isolate + persistent Context.
//! Scripts share state within a page (as in a real browser).
//! A minimal DOM shim lets common JS patterns work without a full DOM.

use std::sync::Once;
use tracing::{info, warn, debug};

static V8_INIT: Once = Once::new();

/// Initialize V8 platform (must be called once).
pub fn init_platform() {
    V8_INIT.call_once(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
        info!("V8 platform initialized");
    });
}

/// Configuration for a JS runtime instance.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Max JS execution time per script in milliseconds.
    pub max_execution_ms: u64,
    /// Max heap size in bytes (0 = unlimited).
    pub max_heap_bytes: usize,
    /// Whether to execute inline scripts found in HTML.
    pub execute_inline_scripts: bool,
    /// Whether to inject the DOM shim before page scripts.
    pub inject_dom_shim: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_execution_ms: 5000,
            max_heap_bytes: 64 * 1024 * 1024,
            execute_inline_scripts: true,
            inject_dom_shim: true,
        }
    }
}

/// Minimal DOM shim injected into V8 before page scripts.
///
/// This provides just enough `document` and `window` API surface for
/// common JS patterns to work: data fetching, DOM insertion, event listeners.
/// We capture mutations in `__plasmate_mutations` for the SOM compiler.
const DOM_SHIM: &str = r#"
// Plasmate DOM Shim - minimal surface for agent-relevant JS

var __plasmate_mutations = [];
var __plasmate_timers = [];

var window = globalThis;
window.location = { href: '', protocol: 'https:', host: '', pathname: '/', search: '', hash: '' };
window.navigator = { userAgent: 'Plasmate/0.1', language: 'en-US', languages: ['en-US', 'en'] };
window.innerWidth = 1920;
window.innerHeight = 1080;

// Minimal Element
function PlasElement(tag, attrs) {
    this.tagName = tag.toUpperCase();
    this.nodeName = this.tagName;
    this.nodeType = 1;
    this.attributes = attrs || {};
    this.children = [];
    this.childNodes = this.children;
    this.textContent = '';
    this.innerHTML = '';
    this.className = '';
    this.id = '';
    this.style = {};
    this.dataset = {};
    this._listeners = {};
}
PlasElement.prototype.setAttribute = function(k, v) {
    this.attributes[k] = v;
    if (k === 'class') this.className = v;
    if (k === 'id') this.id = v;
};
PlasElement.prototype.getAttribute = function(k) { return this.attributes[k] || null; };
PlasElement.prototype.addEventListener = function(ev, fn) {
    if (!this._listeners[ev]) this._listeners[ev] = [];
    this._listeners[ev].push(fn);
};
PlasElement.prototype.removeEventListener = function() {};
PlasElement.prototype.appendChild = function(child) {
    this.children.push(child);
    __plasmate_mutations.push({ type: 'appendChild', tag: child.tagName, text: child.textContent });
    return child;
};
PlasElement.prototype.removeChild = function(child) {
    var idx = this.children.indexOf(child);
    if (idx >= 0) this.children.splice(idx, 1);
    return child;
};
PlasElement.prototype.insertBefore = function(newNode, ref) {
    var idx = ref ? this.children.indexOf(ref) : this.children.length;
    if (idx < 0) idx = this.children.length;
    this.children.splice(idx, 0, newNode);
    __plasmate_mutations.push({ type: 'insertBefore', tag: newNode.tagName });
    return newNode;
};
PlasElement.prototype.querySelector = function() { return null; };
PlasElement.prototype.querySelectorAll = function() { return []; };
PlasElement.prototype.getElementsByTagName = function() { return []; };
PlasElement.prototype.getElementsByClassName = function() { return []; };
PlasElement.prototype.closest = function() { return null; };
PlasElement.prototype.matches = function() { return false; };
PlasElement.prototype.getBoundingClientRect = function() {
    return { top: 0, left: 0, bottom: 0, right: 0, width: 0, height: 0 };
};
PlasElement.prototype.cloneNode = function(deep) { return new PlasElement(this.tagName); };
PlasElement.prototype.dispatchEvent = function() { return true; };
PlasElement.prototype.focus = function() {};
PlasElement.prototype.blur = function() {};
PlasElement.prototype.click = function() {};
PlasElement.prototype.remove = function() {};

// Document
var _docBody = new PlasElement('body');
var _docHead = new PlasElement('head');
var _docEl = new PlasElement('html');
_docEl.children = [_docHead, _docBody];

var document = {
    documentElement: _docEl,
    head: _docHead,
    body: _docBody,
    title: '',
    readyState: 'loading',
    cookie: '',
    referrer: '',
    URL: '',
    domain: '',
    createElement: function(tag) { return new PlasElement(tag); },
    createTextNode: function(text) { return { nodeType: 3, textContent: text }; },
    createDocumentFragment: function() { return new PlasElement('fragment'); },
    createComment: function() { return { nodeType: 8 }; },
    getElementById: function(id) { return null; },
    querySelector: function(sel) { return null; },
    querySelectorAll: function(sel) { return []; },
    getElementsByTagName: function(tag) { return tag === 'head' ? [_docHead] : tag === 'body' ? [_docBody] : []; },
    getElementsByClassName: function() { return []; },
    addEventListener: function(ev, fn) {
        if (ev === 'DOMContentLoaded' || ev === 'readystatechange') {
            try { fn(); } catch(e) {}
        }
    },
    removeEventListener: function() {},
    createEvent: function() { return { initEvent: function(){} }; },
    write: function(html) {
        __plasmate_mutations.push({ type: 'document.write', html: html.substring(0, 500) });
    },
    writeln: function(html) { document.write(html + '\n'); },
    hasFocus: function() { return true; },
    getSelection: function() { return null; },
    execCommand: function() { return false; },
    implementation: { hasFeature: function() { return false; } }
};

window.document = document;

// Console (capture for debugging)
var __plasmate_console = [];
var console = {
    log: function() { __plasmate_console.push(['log', Array.prototype.slice.call(arguments)]); },
    warn: function() { __plasmate_console.push(['warn', Array.prototype.slice.call(arguments)]); },
    error: function() { __plasmate_console.push(['error', Array.prototype.slice.call(arguments)]); },
    info: function() { __plasmate_console.push(['info', Array.prototype.slice.call(arguments)]); },
    debug: function() {},
    trace: function() {},
    dir: function() {},
    table: function() {},
    group: function() {},
    groupEnd: function() {},
    time: function() {},
    timeEnd: function() {},
    assert: function() {}
};

// Timers (tracked, not actually async)
var _timerCounter = 0;
function setTimeout(fn, ms) {
    var id = ++_timerCounter;
    __plasmate_timers.push({ id: id, fn: fn, ms: ms || 0, type: 'timeout' });
    return id;
}
function clearTimeout(id) {}
function setInterval(fn, ms) {
    var id = ++_timerCounter;
    __plasmate_timers.push({ id: id, fn: fn, ms: ms || 0, type: 'interval' });
    return id;
}
function clearInterval(id) {}
function requestAnimationFrame(fn) { return setTimeout(fn, 16); }
function cancelAnimationFrame(id) {}

// Fetch stub (returns empty response)
function fetch(url, opts) {
    __plasmate_mutations.push({ type: 'fetch', url: String(url).substring(0, 200) });
    return Promise.resolve({
        ok: true, status: 200, statusText: 'OK',
        json: function() { return Promise.resolve({}); },
        text: function() { return Promise.resolve(''); },
        blob: function() { return Promise.resolve(new Blob()); },
        headers: { get: function() { return null; } }
    });
}

// XMLHttpRequest stub
function XMLHttpRequest() {
    this.readyState = 0;
    this.status = 0;
    this.responseText = '';
    this.response = '';
}
XMLHttpRequest.prototype.open = function(method, url) {
    __plasmate_mutations.push({ type: 'xhr', method: method, url: String(url).substring(0, 200) });
};
XMLHttpRequest.prototype.send = function() { this.readyState = 4; this.status = 200; };
XMLHttpRequest.prototype.setRequestHeader = function() {};
XMLHttpRequest.prototype.addEventListener = function() {};
XMLHttpRequest.prototype.removeEventListener = function() {};
XMLHttpRequest.prototype.abort = function() {};

// Storage stubs
var _store = {};
var localStorage = {
    getItem: function(k) { return _store[k] || null; },
    setItem: function(k, v) { _store[k] = String(v); },
    removeItem: function(k) { delete _store[k]; },
    clear: function() { _store = {}; },
    get length() { return Object.keys(_store).length; }
};
var sessionStorage = localStorage;

// Other browser globals
var Blob = function() {};
var URL = { createObjectURL: function() { return ''; }, revokeObjectURL: function() {} };
var CustomEvent = function(name, opts) { this.type = name; this.detail = opts ? opts.detail : null; };
var Event = function(name) { this.type = name; };
var MutationObserver = function() {};
MutationObserver.prototype.observe = function() {};
MutationObserver.prototype.disconnect = function() {};
var IntersectionObserver = function() {};
IntersectionObserver.prototype.observe = function() {};
IntersectionObserver.prototype.disconnect = function() {};
var ResizeObserver = function() {};
ResizeObserver.prototype.observe = function() {};
ResizeObserver.prototype.disconnect = function() {};
var matchMedia = function() { return { matches: false, addListener: function(){}, removeListener: function(){} }; };
window.matchMedia = matchMedia;
var getComputedStyle = function() { return {}; };
window.getComputedStyle = getComputedStyle;
var btoa = function(s) { return s; };
var atob = function(s) { return s; };
window.btoa = btoa;
window.atob = atob;
var performance = { now: function() { return Date.now(); }, mark: function(){}, measure: function(){} };
window.performance = performance;
var crypto = { getRandomValues: function(arr) { for(var i=0;i<arr.length;i++) arr[i]=Math.floor(Math.random()*256); return arr; } };
window.crypto = crypto;
var queueMicrotask = function(fn) { try { fn(); } catch(e){} };
window.queueMicrotask = queueMicrotask;
var Promise = globalThis.Promise;

// Signal ready
document.readyState = 'interactive';
"#;

/// A JavaScript runtime bound to a single page.
/// Context persists between script executions (state accumulates like a browser).
pub struct JsRuntime {
    isolate: v8::OwnedIsolate,
    config: RuntimeConfig,
    context: Option<v8::Global<v8::Context>>,
    scripts_executed: usize,
}

impl JsRuntime {
    /// Create a new isolated JS runtime.
    pub fn new(config: RuntimeConfig) -> Self {
        init_platform();

        let params = if config.max_heap_bytes > 0 {
            v8::CreateParams::default()
                .heap_limits(0, config.max_heap_bytes)
        } else {
            v8::CreateParams::default()
        };

        let mut isolate = v8::Isolate::new(params);

        // Create a persistent context
        let context = {
            let scope = &mut v8::HandleScope::new(&mut isolate);
            let ctx = v8::Context::new(scope, Default::default());
            v8::Global::new(scope, ctx)
        };

        let mut rt = Self {
            isolate,
            config: config.clone(),
            context: Some(context),
            scripts_executed: 0,
        };

        // Inject DOM shim
        if config.inject_dom_shim {
            if let Err(e) = rt.execute_in_context(DOM_SHIM, "<plasmate-shim>") {
                warn!("Failed to inject DOM shim: {}", e);
            }
        }

        rt
    }

    /// Set the page URL in the JS context (updates window.location).
    pub fn set_page_url(&mut self, url: &str) {
        let script = format!(
            "window.location.href = '{}'; document.URL = '{}'; document.domain = '{}';",
            url.replace('\'', "\\'"),
            url.replace('\'', "\\'"),
            url::Url::parse(url)
                .map(|u| u.host_str().unwrap_or("").to_string())
                .unwrap_or_default()
                .replace('\'', "\\'"),
        );
        let _ = self.execute_in_context(&script, "<set-url>");
    }

    /// Execute a script in the persistent page context.
    pub fn execute_in_context(&mut self, source: &str, filename: &str) -> Result<String, JsError> {
        let context = self.context.as_ref()
            .ok_or_else(|| JsError::Runtime("No context available".into()))?;

        let scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = v8::Local::new(scope, context);
        let scope = &mut v8::ContextScope::new(scope, context);

        let source_str = v8::String::new(scope, source)
            .ok_or_else(|| JsError::Runtime("Failed to create source string".into()))?;

        let name = v8::String::new(scope, filename).unwrap();
        let origin = v8::ScriptOrigin::new(
            scope, name.into(), 0, 0, false, 0, None, false, false, false, None,
        );

        let tc = &mut v8::TryCatch::new(scope);

        let script = match v8::Script::compile(tc, source_str, Some(&origin)) {
            Some(s) => s,
            None => {
                let msg = tc.exception()
                    .map(|e| e.to_rust_string_lossy(tc))
                    .unwrap_or_else(|| "Unknown compile error".into());
                return Err(JsError::Compile(msg));
            }
        };

        match script.run(tc) {
            Some(result) => {
                self.scripts_executed += 1;
                let result_str = result
                    .to_string(tc)
                    .map(|s| s.to_rust_string_lossy(tc))
                    .unwrap_or_default();
                Ok(result_str)
            }
            None => {
                let msg = tc.exception()
                    .map(|e| e.to_rust_string_lossy(tc))
                    .unwrap_or_else(|| "Unknown runtime error".into());
                // Don't fail - just log and continue (like a real browser)
                debug!(filename, error = %msg, "JS error (non-fatal)");
                Err(JsError::Runtime(msg))
            }
        }
    }

    /// Execute multiple script blocks in order (state accumulates).
    pub fn execute_page_scripts(&mut self, scripts: &[(String, String)]) -> JsExecutionReport {
        let mut report = JsExecutionReport {
            total: scripts.len(),
            succeeded: 0,
            failed: 0,
            errors: Vec::new(),
        };

        for (source, filename) in scripts {
            if source.trim().is_empty() {
                continue;
            }
            match self.execute_in_context(source, filename) {
                Ok(_) => report.succeeded += 1,
                Err(e) => {
                    report.failed += 1;
                    report.errors.push((filename.clone(), e.to_string()));
                }
            }
        }
        report
    }

    /// Get the DOM mutations captured by the shim.
    pub fn get_mutations(&mut self) -> Vec<String> {
        match self.execute_in_context("JSON.stringify(__plasmate_mutations)", "<get-mutations>") {
            Ok(json) => {
                serde_json::from_str::<Vec<serde_json::Value>>(&json)
                    .unwrap_or_default()
                    .iter()
                    .map(|v| v.to_string())
                    .collect()
            }
            Err(_) => Vec::new(),
        }
    }

    /// Get the document.title as set by JS.
    pub fn get_title(&mut self) -> Option<String> {
        self.execute_in_context("document.title", "<get-title>")
            .ok()
            .filter(|s| !s.is_empty())
    }

    /// Drain pending short timers (execute setTimeout callbacks with delay <= threshold_ms).
    pub fn drain_timers(&mut self, threshold_ms: u64) {
        let script = format!(
            r#"(function() {{
                var executed = 0;
                for (var i = 0; i < __plasmate_timers.length && executed < 50; i++) {{
                    var t = __plasmate_timers[i];
                    if (t.type === 'timeout' && t.ms <= {}) {{
                        try {{ t.fn(); }} catch(e) {{}}
                        executed++;
                    }}
                }}
                __plasmate_timers = [];
                return executed;
            }})()"#,
            threshold_ms
        );
        let _ = self.execute_in_context(&script, "<drain-timers>");
    }

    /// Quick eval for AWP page.extract / interactive use.
    pub fn eval(&mut self, expression: &str) -> Result<String, JsError> {
        self.execute_in_context(expression, "<eval>")
    }

    /// Get heap statistics.
    pub fn heap_stats(&mut self) -> HeapStats {
        let mut stats = v8::HeapStatistics::default();
        self.isolate.get_heap_statistics(&mut stats);
        HeapStats {
            used_bytes: stats.used_heap_size(),
            total_bytes: stats.total_heap_size(),
            limit_bytes: stats.heap_size_limit(),
        }
    }

    /// Number of scripts successfully executed.
    pub fn scripts_executed(&self) -> usize {
        self.scripts_executed
    }
}

/// Report from executing page scripts.
#[derive(Debug, Clone)]
pub struct JsExecutionReport {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub errors: Vec<(String, String)>,
}

/// Heap memory statistics.
#[derive(Debug, Clone)]
pub struct HeapStats {
    pub used_bytes: usize,
    pub total_bytes: usize,
    pub limit_bytes: usize,
}

/// Errors from JS execution.
#[derive(Debug, thiserror::Error)]
pub enum JsError {
    #[error("Compile error: {0}")]
    Compile(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
    #[error("Timeout: execution exceeded {0}ms")]
    Timeout(u64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_execution() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        let result = rt.execute_in_context("1 + 2", "test.js").unwrap();
        assert_eq!(result, "3");
    }

    #[test]
    fn test_persistent_context() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.execute_in_context("var x = 42;", "a.js").unwrap();
        let result = rt.execute_in_context("x + 8", "b.js").unwrap();
        assert_eq!(result, "50", "Variables should persist across scripts");
    }

    #[test]
    fn test_dom_shim_exists() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        let result = rt.execute_in_context("typeof document", "test.js").unwrap();
        assert_eq!(result, "object");
        let result = rt.execute_in_context("typeof window", "test.js").unwrap();
        assert_eq!(result, "object");
    }

    #[test]
    fn test_document_create_element() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        let result = rt.execute_in_context(
            "var el = document.createElement('div'); el.tagName",
            "test.js",
        ).unwrap();
        assert_eq!(result, "DIV");
    }

    #[test]
    fn test_dom_mutations_captured() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.execute_in_context(
            "var el = document.createElement('p'); el.textContent = 'hello'; document.body.appendChild(el);",
            "test.js",
        ).unwrap();
        let mutations = rt.get_mutations();
        assert!(!mutations.is_empty(), "DOM mutations should be captured");
    }

    #[test]
    fn test_set_timeout_captured() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.execute_in_context("setTimeout(function(){}, 100)", "test.js").unwrap();
        let timers = rt.execute_in_context("__plasmate_timers.length", "test.js").unwrap();
        assert_eq!(timers, "1");
    }

    #[test]
    fn test_console_captured() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.execute_in_context("console.log('hello', 'world')", "test.js").unwrap();
        let logs = rt.execute_in_context("__plasmate_console.length", "test.js").unwrap();
        assert_eq!(logs, "1");
    }

    #[test]
    fn test_js_error_nonfatal() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        // This should fail but not crash
        let result = rt.execute_in_context("undefinedVar.prop", "test.js");
        assert!(result.is_err());
        // Runtime should still work after error
        let ok = rt.execute_in_context("1 + 1", "test.js").unwrap();
        assert_eq!(ok, "2");
    }

    #[test]
    fn test_page_scripts_execution() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        let scripts = vec![
            ("var counter = 0;".to_string(), "init.js".to_string()),
            ("counter += 10;".to_string(), "add.js".to_string()),
            ("counter += 5;".to_string(), "add2.js".to_string()),
        ];
        let report = rt.execute_page_scripts(&scripts);
        assert_eq!(report.succeeded, 3);
        assert_eq!(report.failed, 0);
        let val = rt.execute_in_context("counter", "check.js").unwrap();
        assert_eq!(val, "15");
    }

    #[test]
    fn test_page_url_set() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.set_page_url("https://example.com/page");
        let href = rt.execute_in_context("window.location.href", "test.js").unwrap();
        assert!(href.contains("example.com"));
    }

    #[test]
    fn test_drain_timers() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.execute_in_context("var x = 0; setTimeout(function(){ x = 42; }, 0);", "test.js").unwrap();
        rt.drain_timers(100);
        let val = rt.execute_in_context("x", "check.js").unwrap();
        assert_eq!(val, "42");
    }
}
