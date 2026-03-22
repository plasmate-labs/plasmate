//! Wasm plugin system for extending the Plasmate pipeline.
//!
//! Plugins are WebAssembly modules that hook into the page-processing pipeline
//! at well-defined points: before navigation, after HTML parsing, after SOM
//! compilation, and during data extraction.
//!
//! Plugins run sandboxed — they have no filesystem or network access. The only
//! host functionality available is `host_log` for debug output.
//!
//! # Plugin ABI
//!
//! A valid plugin `.wasm` must export:
//!
//! | Export                | Signature             | Purpose                           |
//! |-----------------------|-----------------------|-----------------------------------|
//! | `memory`              | Memory                | Shared linear memory              |
//! | `malloc`              | `(i32) -> i32`        | Allocate bytes, return pointer    |
//! | `plugin_name_ptr`     | `() -> i32`           | Pointer to name string            |
//! | `plugin_name_len`     | `() -> i32`           | Length of name string             |
//! | `plugin_version_ptr`  | `() -> i32`           | Pointer to version string         |
//! | `plugin_version_len`  | `() -> i32`           | Length of version string          |
//! | `get_hooks`           | `() -> i32`           | Bitmask: 1=pre_navigate, 2=post_parse, 4=post_som, 8=on_extract |
//! | `on_hook`             | `(i32,i32,i32) -> i32`| `(hook_id, data_ptr, data_len) -> 0` on success |
//! | `get_result_ptr`      | `() -> i32`           | Pointer to result buffer          |
//! | `get_result_len`      | `() -> i32`           | Length of result (0 = no change)  |
//!
//! The host writes input data into guest memory via `malloc`, then calls
//! `on_hook`. The guest processes the data and stores its result in its own
//! memory, returning a pointer/length via `get_result_ptr`/`get_result_len`.
//!
//! # Hook types
//!
//! - **pre_navigate** (1): receives URL bytes, returns (optionally rewritten) URL
//! - **post_parse** (2): receives HTML bytes after JS execution, returns modified HTML
//! - **post_som** (4): receives SOM JSON, returns modified SOM JSON
//! - **on_extract** (8): receives SOM JSON, returns extracted/annotated JSON
//!
//! # Feature gating
//!
//! The Wasm runtime (wasmtime) is behind the `plugins` Cargo feature.
//! When disabled, a stub `PluginManager` is provided that returns errors
//! on any attempt to load plugins. The types module is always available.

pub mod types;

#[cfg(feature = "plugins")]
pub mod manager;
#[cfg(feature = "plugins")]
pub mod runtime;

#[cfg(feature = "plugins")]
pub use manager::PluginManager;

pub use types::{Hook, PluginError, PluginManifest};

// ---------------------------------------------------------------------------
// Stub PluginManager when the `plugins` feature is disabled.
// ---------------------------------------------------------------------------

#[cfg(not(feature = "plugins"))]
pub struct PluginManager;

#[cfg(not(feature = "plugins"))]
impl PluginManager {
    pub fn new() -> Result<Self, PluginError> {
        Err(PluginError::Load(
            "Wasm plugins require the 'plugins' feature (build with --features plugins)".into(),
        ))
    }

    pub fn load(&mut self, _path: &std::path::Path) -> Result<PluginManifest, PluginError> {
        Err(PluginError::Load(
            "Wasm plugins require the 'plugins' feature".into(),
        ))
    }

    pub fn load_bytes(&mut self, _bytes: &[u8]) -> Result<PluginManifest, PluginError> {
        Err(PluginError::Load(
            "Wasm plugins require the 'plugins' feature".into(),
        ))
    }

    pub fn plugin_count(&self) -> usize {
        0
    }

    pub fn manifests(&self) -> Vec<&PluginManifest> {
        Vec::new()
    }

    pub fn has_hook(&self, _hook: Hook) -> bool {
        false
    }

    pub fn run_hook(&mut self, _hook: Hook, input: &[u8]) -> Result<Vec<u8>, PluginError> {
        Ok(input.to_vec())
    }

    pub fn run_pre_navigate(&mut self, url: &str) -> Result<String, PluginError> {
        Ok(url.to_string())
    }

    pub fn run_post_parse(&mut self, html: &str) -> Result<String, PluginError> {
        Ok(html.to_string())
    }

    pub fn run_post_som(
        &mut self,
        som: crate::som::types::Som,
    ) -> Result<crate::som::types::Som, PluginError> {
        Ok(som)
    }

    pub fn run_on_extract(
        &mut self,
        _som: &crate::som::types::Som,
    ) -> Result<Option<serde_json::Value>, PluginError> {
        Ok(None)
    }
}
