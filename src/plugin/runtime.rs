//! Single Wasm plugin instance.
//!
//! Each `.wasm` file becomes one `WasmPlugin`. The host provides a minimal
//! set of imports (`env.host_log`) and expects a fixed set of exports from
//! the guest (see the plugin ABI below).
//!
//! ## Plugin ABI (guest exports)
//!
//! | Export              | Signature           | Purpose                              |
//! |---------------------|---------------------|--------------------------------------|
//! | `memory`            | Memory              | Shared linear memory                 |
//! | `malloc`            | `(i32) -> i32`      | Allocate `n` bytes, return pointer    |
//! | `plugin_name_ptr`   | `() -> i32`         | Pointer to plugin name (UTF-8)       |
//! | `plugin_name_len`   | `() -> i32`         | Length of plugin name                |
//! | `plugin_version_ptr`| `() -> i32`         | Pointer to plugin version (UTF-8)    |
//! | `plugin_version_len`| `() -> i32`         | Length of plugin version             |
//! | `get_hooks`         | `() -> i32`         | Bitmask of hooks this plugin wants   |
//! | `on_hook`           | `(i32,i32,i32)->i32`| Process hook: (hook_id, ptr, len)    |
//! | `get_result_ptr`    | `() -> i32`         | Pointer to last result               |
//! | `get_result_len`    | `() -> i32`         | Length of last result (0 = no change)|

use std::path::Path;

use tracing::debug;
use wasmtime::*;

use super::types::*;

/// Host state passed into the Wasm store.
struct PluginState {
    log_messages: Vec<String>,
}

/// A loaded Wasm plugin, ready to process hooks.
pub struct WasmPlugin {
    store: Store<PluginState>,
    instance: Instance,
    manifest: PluginManifest,
    hook_bits: u32,
}

impl WasmPlugin {
    /// Load and instantiate a plugin from a `.wasm` file.
    pub fn load(path: &Path, engine: &Engine) -> Result<Self, PluginError> {
        let module = Module::from_file(engine, path)
            .map_err(|e| PluginError::Load(format!("{}: {}", path.display(), e)))?;

        Self::from_module(&module, engine, path)
    }

    /// Load a plugin from raw Wasm bytes (used by tests).
    pub fn from_bytes(bytes: &[u8], engine: &Engine) -> Result<Self, PluginError> {
        let module = Module::new(engine, bytes)
            .map_err(|e| PluginError::Load(format!("<bytes>: {}", e)))?;

        Self::from_module(&module, engine, Path::new("<bytes>"))
    }

    fn from_module(module: &Module, engine: &Engine, path: &Path) -> Result<Self, PluginError> {
        let mut store = Store::new(
            engine,
            PluginState {
                log_messages: Vec::new(),
            },
        );

        let mut linker = Linker::new(engine);

        // Host import: host_log(ptr, len) — write a log message from the plugin.
        linker
            .func_wrap(
                "env",
                "host_log",
                |mut caller: Caller<'_, PluginState>, ptr: i32, len: i32| {
                    let mem = caller
                        .get_export("memory")
                        .and_then(|e| e.into_memory())
                        .expect("plugin must export memory");
                    let data = mem.data(&caller);
                    let msg = data
                        .get(ptr as usize..ptr as usize + len as usize)
                        .and_then(|slice| std::str::from_utf8(slice).ok())
                        .map(|s| s.to_string());
                    if let Some(msg) = msg {
                        debug!(plugin_msg = %msg, "plugin log");
                        caller.data_mut().log_messages.push(msg);
                    }
                },
            )
            .map_err(|e| PluginError::Load(e.to_string()))?;

        let instance = linker
            .instantiate(&mut store, module)
            .map_err(|e| PluginError::Load(e.to_string()))?;

        // Read plugin metadata from guest memory.
        let name = read_guest_string(&mut store, &instance, "plugin_name_ptr", "plugin_name_len")?;
        let version = read_guest_string(
            &mut store,
            &instance,
            "plugin_version_ptr",
            "plugin_version_len",
        )?;

        let get_hooks = instance
            .get_typed_func::<(), i32>(&mut store, "get_hooks")
            .map_err(|_| PluginError::MissingExport("get_hooks".into()))?;
        let hook_bits = get_hooks
            .call(&mut store, ())
            .map_err(|e| PluginError::Execution(e.to_string()))? as u32;

        let hooks = Hook::from_bits(hook_bits)
            .iter()
            .map(|h| h.as_str().to_string())
            .collect();

        let manifest = PluginManifest {
            name,
            version,
            hooks,
            path: path.to_path_buf(),
        };

        Ok(WasmPlugin {
            store,
            instance,
            manifest,
            hook_bits,
        })
    }

    pub fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    pub fn supports_hook(&self, hook: Hook) -> bool {
        self.hook_bits & (hook as u32) != 0
    }

    /// Invoke a hook on this plugin.
    ///
    /// Returns `Ok(Some(bytes))` if the plugin produced output,
    /// `Ok(None)` if the plugin chose not to modify the data,
    /// or `Err` on failure.
    pub fn call_hook(&mut self, hook: Hook, input: &[u8]) -> Result<Option<Vec<u8>>, PluginError> {
        if !self.supports_hook(hook) {
            return Ok(None);
        }

        let memory = self
            .instance
            .get_memory(&mut self.store, "memory")
            .ok_or_else(|| PluginError::MissingExport("memory".into()))?;

        // Allocate space in guest for the input.
        let malloc = self
            .instance
            .get_typed_func::<i32, i32>(&mut self.store, "malloc")
            .map_err(|_| PluginError::MissingExport("malloc".into()))?;

        let input_ptr = malloc
            .call(&mut self.store, input.len() as i32)
            .map_err(|e| PluginError::Execution(e.to_string()))?;

        // Write input into guest memory.
        memory
            .write(&mut self.store, input_ptr as usize, input)
            .map_err(|e| PluginError::Execution(format!("memory write: {}", e)))?;

        // Call the hook.
        let on_hook = self
            .instance
            .get_typed_func::<(i32, i32, i32), i32>(&mut self.store, "on_hook")
            .map_err(|_| PluginError::MissingExport("on_hook".into()))?;

        let rc = on_hook
            .call(
                &mut self.store,
                (hook as i32, input_ptr, input.len() as i32),
            )
            .map_err(|e| PluginError::Execution(e.to_string()))?;

        if rc != 0 {
            return Err(PluginError::Execution(format!(
                "plugin '{}' returned error code {} for {:?}",
                self.manifest.name, rc, hook
            )));
        }

        // Read result pointer and length.
        let get_result_ptr = self
            .instance
            .get_typed_func::<(), i32>(&mut self.store, "get_result_ptr")
            .map_err(|_| PluginError::MissingExport("get_result_ptr".into()))?;
        let get_result_len = self
            .instance
            .get_typed_func::<(), i32>(&mut self.store, "get_result_len")
            .map_err(|_| PluginError::MissingExport("get_result_len".into()))?;

        let result_ptr = get_result_ptr
            .call(&mut self.store, ())
            .map_err(|e| PluginError::Execution(e.to_string()))? as usize;
        let result_len = get_result_len
            .call(&mut self.store, ())
            .map_err(|e| PluginError::Execution(e.to_string()))? as usize;

        if result_len == 0 {
            return Ok(None);
        }

        let data = memory.data(&self.store);
        let output = data
            .get(result_ptr..result_ptr + result_len)
            .ok_or_else(|| {
                PluginError::Execution(format!(
                    "result out of bounds: ptr={}, len={}, mem={}",
                    result_ptr,
                    result_len,
                    data.len()
                ))
            })?
            .to_vec();

        Ok(Some(output))
    }

    /// Drain log messages accumulated during the last hook call.
    pub fn take_logs(&mut self) -> Vec<String> {
        std::mem::take(&mut self.store.data_mut().log_messages)
    }
}

/// Read a UTF-8 string from guest memory using a ptr/len function pair.
fn read_guest_string(
    store: &mut Store<PluginState>,
    instance: &Instance,
    ptr_fn: &str,
    len_fn: &str,
) -> Result<String, PluginError> {
    let get_ptr = instance
        .get_typed_func::<(), i32>(&mut *store, ptr_fn)
        .map_err(|_| PluginError::MissingExport(ptr_fn.into()))?;
    let get_len = instance
        .get_typed_func::<(), i32>(&mut *store, len_fn)
        .map_err(|_| PluginError::MissingExport(len_fn.into()))?;

    let ptr = get_ptr
        .call(&mut *store, ())
        .map_err(|e| PluginError::Execution(e.to_string()))? as usize;
    let len = get_len
        .call(&mut *store, ())
        .map_err(|e| PluginError::Execution(e.to_string()))? as usize;

    let memory = instance
        .get_memory(&mut *store, "memory")
        .ok_or_else(|| PluginError::MissingExport("memory".into()))?;

    let data = memory.data(&*store);
    let bytes = data.get(ptr..ptr + len).ok_or_else(|| {
        PluginError::Execution(format!(
            "string out of bounds: ptr={}, len={}, mem={}",
            ptr,
            len,
            data.len()
        ))
    })?;

    String::from_utf8(bytes.to_vec())
        .map_err(|e| PluginError::Execution(format!("invalid UTF-8: {}", e)))
}
