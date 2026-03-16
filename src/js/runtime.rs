//! V8-backed JavaScript runtime for Plasmate.
//!
//! Each browser session gets its own V8 Isolate for full isolation.
//! The runtime exposes a minimal DOM bridge - just enough for pages
//! to render their dynamic content.

use std::sync::Once;
use tracing::{info, warn};

static V8_INIT: Once = Once::new();

/// Initialize V8 platform (must be called once before creating any runtime).
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
    /// Max JS execution time in milliseconds before termination.
    pub max_execution_ms: u64,
    /// Max heap size in bytes (0 = unlimited).
    pub max_heap_bytes: usize,
    /// Whether to execute inline scripts found in HTML.
    pub execute_inline_scripts: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_execution_ms: 5000,
            max_heap_bytes: 64 * 1024 * 1024, // 64MB per isolate
            execute_inline_scripts: true,
        }
    }
}

/// A JavaScript runtime bound to a single page/session.
pub struct JsRuntime {
    isolate: v8::OwnedIsolate,
    config: RuntimeConfig,
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

        let isolate = v8::Isolate::new(params);

        Self { isolate, config }
    }

    /// Execute a JavaScript string and return the result as a Rust string.
    pub fn execute(&mut self, source: &str, filename: &str) -> Result<String, JsError> {
        let scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = v8::Context::new(scope, Default::default());
        let scope = &mut v8::ContextScope::new(scope, context);

        let source_str = v8::String::new(scope, source)
            .ok_or_else(|| JsError::Runtime("Failed to create source string".into()))?;

        let origin = create_script_origin(scope, filename);

        let script = v8::Script::compile(scope, source_str, Some(&origin))
            .ok_or_else(|| JsError::Compile(format!("Failed to compile: {}", filename)))?;

        let result = script
            .run(scope)
            .ok_or_else(|| JsError::Runtime("Script execution returned null".into()))?;

        let result_str = result
            .to_string(scope)
            .map(|s| s.to_rust_string_lossy(scope))
            .unwrap_or_default();

        Ok(result_str)
    }

    /// Execute multiple script blocks (as found in HTML), collecting errors.
    pub fn execute_scripts(&mut self, scripts: &[(String, String)]) -> Vec<Result<String, JsError>> {
        scripts
            .iter()
            .map(|(source, filename)| self.execute(source, filename))
            .collect()
    }

    /// Quick eval for CDP Runtime.evaluate
    pub fn eval(&mut self, expression: &str) -> Result<String, JsError> {
        self.execute(expression, "<eval>")
    }

    /// Get heap statistics for memory tracking.
    pub fn heap_stats(&mut self) -> HeapStats {
        let mut stats = v8::HeapStatistics::default();
        self.isolate.get_heap_statistics(&mut stats);
        HeapStats {
            used_bytes: stats.used_heap_size(),
            total_bytes: stats.total_heap_size(),
            limit_bytes: stats.heap_size_limit(),
        }
    }
}

/// Heap memory statistics for a runtime.
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

fn create_script_origin<'s>(
    scope: &mut v8::HandleScope<'s>,
    filename: &str,
) -> v8::ScriptOrigin<'s> {
    let name = v8::String::new(scope, filename).unwrap();
    v8::ScriptOrigin::new(
        scope,
        name.into(),
        0,
        0,
        false,
        0,
        None,
        false,
        false,
        false,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_execution() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        let result = rt.execute("1 + 2", "test.js").unwrap();
        assert_eq!(result, "3");
    }

    #[test]
    fn test_string_execution() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        let result = rt.execute("'hello' + ' ' + 'world'", "test.js").unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_json_output() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        let result = rt.execute("JSON.stringify({a: 1, b: 'two'})", "test.js").unwrap();
        assert_eq!(result, r#"{"a":1,"b":"two"}"#);
    }

    #[test]
    fn test_compile_error() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        let result = rt.execute("this is not valid javascript }{", "bad.js");
        assert!(result.is_err());
    }

    #[test]
    fn test_heap_stats() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        let stats = rt.heap_stats();
        assert!(stats.total_bytes > 0);
        assert!(stats.limit_bytes > 0);
    }

    #[test]
    fn test_multiple_executions_same_isolate() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        let r1 = rt.execute("var x = 42; x", "a.js").unwrap();
        assert_eq!(r1, "42");
        // Note: each execute creates a new context, so x won't persist
        // This is intentional for isolation
    }
}
