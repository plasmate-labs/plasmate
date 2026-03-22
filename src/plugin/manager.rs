//! Plugin manager — coordinates multiple Wasm plugins.
//!
//! Plugins are loaded once at startup and invoked at each hook point.
//! When multiple plugins register for the same hook, they are chained:
//! the output of one becomes the input of the next (pipeline order).

use std::path::Path;

use tracing::info;
use wasmtime::Engine;

use super::runtime::WasmPlugin;
use super::types::*;
use crate::som::types::Som;

/// Manages the lifecycle of all loaded Wasm plugins.
pub struct PluginManager {
    engine: Engine,
    plugins: Vec<WasmPlugin>,
}

impl PluginManager {
    pub fn new() -> Result<Self, PluginError> {
        let engine = Engine::default();
        Ok(PluginManager {
            engine,
            plugins: Vec::new(),
        })
    }

    /// Load a `.wasm` plugin from disk.
    pub fn load(&mut self, path: &Path) -> Result<PluginManifest, PluginError> {
        let plugin = WasmPlugin::load(path, &self.engine)?;
        let manifest = plugin.manifest().clone();
        info!(
            name = %manifest.name,
            version = %manifest.version,
            hooks = ?manifest.hooks,
            path = %path.display(),
            "Plugin loaded"
        );
        self.plugins.push(plugin);
        Ok(manifest)
    }

    /// Load a plugin from raw bytes (used by tests).
    pub fn load_bytes(&mut self, bytes: &[u8]) -> Result<PluginManifest, PluginError> {
        let plugin = WasmPlugin::from_bytes(bytes, &self.engine)?;
        let manifest = plugin.manifest().clone();
        self.plugins.push(plugin);
        Ok(manifest)
    }

    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    pub fn manifests(&self) -> Vec<&PluginManifest> {
        self.plugins.iter().map(|p| p.manifest()).collect()
    }

    /// Whether any loaded plugin handles `hook`.
    pub fn has_hook(&self, hook: Hook) -> bool {
        self.plugins.iter().any(|p| p.supports_hook(hook))
    }

    /// Run all plugins that handle `hook`, chaining their output.
    ///
    /// If no plugin modifies the data the original input is returned.
    pub fn run_hook(&mut self, hook: Hook, input: &[u8]) -> Result<Vec<u8>, PluginError> {
        let mut current = input.to_vec();
        for plugin in &mut self.plugins {
            if plugin.supports_hook(hook) {
                if let Some(output) = plugin.call_hook(hook, &current)? {
                    current = output;
                }
            }
        }
        Ok(current)
    }

    // ---- Convenience methods for each hook type ----

    /// Run `pre_navigate` plugins on a URL string.
    pub fn run_pre_navigate(&mut self, url: &str) -> Result<String, PluginError> {
        if !self.has_hook(Hook::PreNavigate) {
            return Ok(url.to_string());
        }
        let output = self.run_hook(Hook::PreNavigate, url.as_bytes())?;
        String::from_utf8(output).map_err(|e| {
            PluginError::InvalidOutput(format!("pre_navigate produced invalid UTF-8: {}", e))
        })
    }

    /// Run `post_parse` plugins on the effective HTML (after JS, before SOM).
    pub fn run_post_parse(&mut self, html: &str) -> Result<String, PluginError> {
        if !self.has_hook(Hook::PostParse) {
            return Ok(html.to_string());
        }
        let output = self.run_hook(Hook::PostParse, html.as_bytes())?;
        String::from_utf8(output).map_err(|e| {
            PluginError::InvalidOutput(format!("post_parse produced invalid UTF-8: {}", e))
        })
    }

    /// Run `post_som` plugins on a compiled SOM.
    pub fn run_post_som(&mut self, som: Som) -> Result<Som, PluginError> {
        if !self.has_hook(Hook::PostSom) {
            return Ok(som);
        }
        let json = serde_json::to_vec(&som)
            .map_err(|e| PluginError::Execution(format!("serialize SOM: {}", e)))?;
        let output = self.run_hook(Hook::PostSom, &json)?;
        serde_json::from_slice(&output).map_err(|e| {
            PluginError::InvalidOutput(format!("post_som produced invalid SOM JSON: {}", e))
        })
    }

    /// Run `on_extract` plugins. Returns the plugin-produced JSON (if any).
    pub fn run_on_extract(
        &mut self,
        som: &Som,
    ) -> Result<Option<serde_json::Value>, PluginError> {
        if !self.has_hook(Hook::OnExtract) {
            return Ok(None);
        }
        let json = serde_json::to_vec(som)
            .map_err(|e| PluginError::Execution(format!("serialize SOM: {}", e)))?;
        let output = self.run_hook(Hook::OnExtract, &json)?;
        let value = serde_json::from_slice(&output).map_err(|e| {
            PluginError::InvalidOutput(format!("on_extract produced invalid JSON: {}", e))
        })?;
        Ok(Some(value))
    }
}
