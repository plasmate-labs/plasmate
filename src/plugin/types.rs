use std::path::PathBuf;

use serde::Serialize;

/// Hook points in the Plasmate pipeline where plugins can intercept.
///
/// Represented as a bitmask so plugins can register for multiple hooks
/// with a single `get_hooks()` export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Hook {
    /// Before navigation. Input: URL string. Output: optionally rewritten URL.
    PreNavigate = 1,
    /// After JS execution, before SOM compilation. Input: HTML. Output: modified HTML.
    PostParse = 2,
    /// After SOM compilation. Input: SOM JSON. Output: modified SOM JSON.
    PostSom = 4,
    /// During data extraction. Input: SOM JSON. Output: extracted/annotated data.
    OnExtract = 8,
}

impl Hook {
    /// Decode a bitmask into a list of hooks.
    pub fn from_bits(bits: u32) -> Vec<Hook> {
        let mut hooks = Vec::new();
        if bits & 1 != 0 {
            hooks.push(Hook::PreNavigate);
        }
        if bits & 2 != 0 {
            hooks.push(Hook::PostParse);
        }
        if bits & 4 != 0 {
            hooks.push(Hook::PostSom);
        }
        if bits & 8 != 0 {
            hooks.push(Hook::OnExtract);
        }
        hooks
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Hook::PreNavigate => "pre_navigate",
            Hook::PostParse => "post_parse",
            Hook::PostSom => "post_som",
            Hook::OnExtract => "on_extract",
        }
    }
}

/// Metadata about a loaded plugin, returned by list/info APIs.
#[derive(Debug, Clone, Serialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub hooks: Vec<String>,
    pub path: PathBuf,
}

/// Errors from the plugin system.
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("failed to load plugin: {0}")]
    Load(String),
    #[error("plugin missing required export '{0}'")]
    MissingExport(String),
    #[error("plugin execution failed: {0}")]
    Execution(String),
    #[error("plugin returned invalid data: {0}")]
    InvalidOutput(String),
}
