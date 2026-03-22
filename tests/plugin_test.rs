//! Integration tests for the Wasm plugin system.

use plasmate::plugin::{Hook, PluginManager};

/// A minimal passthrough plugin in WAT. Registers for post_som (hook 4).
/// Returns the input unchanged.
const PASSTHROUGH_WAT: &str = r#"
(module
  (memory (export "memory") 1)

  ;; Static data: plugin name and version
  (data (i32.const 0) "passthrough")
  (data (i32.const 16) "0.1.0")

  ;; Bump allocator
  (global $heap (mut i32) (i32.const 1024))
  (func (export "malloc") (param $size i32) (result i32)
    (local $ptr i32)
    (local.set $ptr (global.get $heap))
    (global.set $heap (i32.add (global.get $heap) (local.get $size)))
    (local.get $ptr)
  )

  ;; Plugin metadata
  (func (export "plugin_name_ptr") (result i32) (i32.const 0))
  (func (export "plugin_name_len") (result i32) (i32.const 11))
  (func (export "plugin_version_ptr") (result i32) (i32.const 16))
  (func (export "plugin_version_len") (result i32) (i32.const 5))

  ;; Hook bitmask: post_som = 4
  (func (export "get_hooks") (result i32) (i32.const 4))

  ;; Result tracking
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  (func (export "get_result_ptr") (result i32) (global.get $result_ptr))
  (func (export "get_result_len") (result i32) (global.get $result_len))

  ;; on_hook: passthrough — returns the input unchanged
  (func (export "on_hook") (param $hook i32) (param $ptr i32) (param $len i32) (result i32)
    (global.set $result_ptr (local.get $ptr))
    (global.set $result_len (local.get $len))
    (i32.const 0)
  )
)
"#;

/// A plugin that returns zero-length result (no modification).
const NOOP_WAT: &str = r#"
(module
  (memory (export "memory") 1)
  (data (i32.const 0) "noop")
  (data (i32.const 8) "1.0.0")

  (global $heap (mut i32) (i32.const 1024))
  (func (export "malloc") (param $size i32) (result i32)
    (local $ptr i32)
    (local.set $ptr (global.get $heap))
    (global.set $heap (i32.add (global.get $heap) (local.get $size)))
    (local.get $ptr)
  )

  (func (export "plugin_name_ptr") (result i32) (i32.const 0))
  (func (export "plugin_name_len") (result i32) (i32.const 4))
  (func (export "plugin_version_ptr") (result i32) (i32.const 8))
  (func (export "plugin_version_len") (result i32) (i32.const 5))

  ;; Registers for all hooks
  (func (export "get_hooks") (result i32) (i32.const 15))

  (func (export "get_result_ptr") (result i32) (i32.const 0))
  (func (export "get_result_len") (result i32) (i32.const 0))

  ;; on_hook: returns success but result_len stays 0 (no modification)
  (func (export "on_hook") (param $hook i32) (param $ptr i32) (param $len i32) (result i32)
    (i32.const 0)
  )
)
"#;

/// A plugin that uppercases all bytes in the input (for post_parse HTML hook).
/// Registers for post_parse (hook 2). Copies input to new location, uppercases ASCII.
const UPPERCASE_WAT: &str = r#"
(module
  (memory (export "memory") 2)
  (data (i32.const 0) "uppercase")
  (data (i32.const 16) "0.1.0")

  (global $heap (mut i32) (i32.const 4096))
  (func (export "malloc") (param $size i32) (result i32)
    (local $ptr i32)
    (local.set $ptr (global.get $heap))
    (global.set $heap (i32.add (global.get $heap) (local.get $size)))
    (local.get $ptr)
  )

  (func (export "plugin_name_ptr") (result i32) (i32.const 0))
  (func (export "plugin_name_len") (result i32) (i32.const 9))
  (func (export "plugin_version_ptr") (result i32) (i32.const 16))
  (func (export "plugin_version_len") (result i32) (i32.const 5))

  ;; post_parse = 2
  (func (export "get_hooks") (result i32) (i32.const 2))

  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  (func (export "get_result_ptr") (result i32) (global.get $result_ptr))
  (func (export "get_result_len") (result i32) (global.get $result_len))

  ;; on_hook: copy input and uppercase ASCII a-z -> A-Z
  (func (export "on_hook") (param $hook i32) (param $ptr i32) (param $len i32) (result i32)
    (local $i i32)
    (local $out i32)
    (local $byte i32)

    ;; Allocate output at a fixed offset (65536 = second page)
    (local.set $out (i32.const 65536))
    (global.set $result_ptr (local.get $out))
    (global.set $result_len (local.get $len))

    (local.set $i (i32.const 0))
    (block $done
      (loop $loop
        (br_if $done (i32.ge_u (local.get $i) (local.get $len)))
        (local.set $byte (i32.load8_u (i32.add (local.get $ptr) (local.get $i))))
        ;; If 'a' <= byte <= 'z', subtract 32
        (if (i32.and
              (i32.ge_u (local.get $byte) (i32.const 97))
              (i32.le_u (local.get $byte) (i32.const 122)))
          (then
            (local.set $byte (i32.sub (local.get $byte) (i32.const 32)))
          )
        )
        (i32.store8
          (i32.add (local.get $out) (local.get $i))
          (local.get $byte)
        )
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $loop)
      )
    )
    (i32.const 0)
  )
)
"#;

/// A plugin that logs via host_log.
const LOGGING_WAT: &str = r#"
(module
  (import "env" "host_log" (func $host_log (param i32 i32)))
  (memory (export "memory") 1)
  (data (i32.const 0) "logger")
  (data (i32.const 8) "0.1.0")
  (data (i32.const 32) "hello from plugin")

  (global $heap (mut i32) (i32.const 1024))
  (func (export "malloc") (param $size i32) (result i32)
    (local $ptr i32)
    (local.set $ptr (global.get $heap))
    (global.set $heap (i32.add (global.get $heap) (local.get $size)))
    (local.get $ptr)
  )

  (func (export "plugin_name_ptr") (result i32) (i32.const 0))
  (func (export "plugin_name_len") (result i32) (i32.const 6))
  (func (export "plugin_version_ptr") (result i32) (i32.const 8))
  (func (export "plugin_version_len") (result i32) (i32.const 5))
  (func (export "get_hooks") (result i32) (i32.const 4))

  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  (func (export "get_result_ptr") (result i32) (global.get $result_ptr))
  (func (export "get_result_len") (result i32) (global.get $result_len))

  (func (export "on_hook") (param $hook i32) (param $ptr i32) (param $len i32) (result i32)
    ;; Log a message via host
    (call $host_log (i32.const 32) (i32.const 17))
    ;; Passthrough
    (global.set $result_ptr (local.get $ptr))
    (global.set $result_len (local.get $len))
    (i32.const 0)
  )
)
"#;

/// A plugin that returns error code 1.
const ERROR_WAT: &str = r#"
(module
  (memory (export "memory") 1)
  (data (i32.const 0) "error-plugin")
  (data (i32.const 16) "0.0.1")

  (global $heap (mut i32) (i32.const 1024))
  (func (export "malloc") (param $size i32) (result i32)
    (local $ptr i32)
    (local.set $ptr (global.get $heap))
    (global.set $heap (i32.add (global.get $heap) (local.get $size)))
    (local.get $ptr)
  )

  (func (export "plugin_name_ptr") (result i32) (i32.const 0))
  (func (export "plugin_name_len") (result i32) (i32.const 12))
  (func (export "plugin_version_ptr") (result i32) (i32.const 16))
  (func (export "plugin_version_len") (result i32) (i32.const 5))
  (func (export "get_hooks") (result i32) (i32.const 4))

  (func (export "get_result_ptr") (result i32) (i32.const 0))
  (func (export "get_result_len") (result i32) (i32.const 0))

  ;; Always returns error code 1
  (func (export "on_hook") (param $hook i32) (param $ptr i32) (param $len i32) (result i32)
    (i32.const 1)
  )
)
"#;

fn compile_wat(wat_source: &str) -> Vec<u8> {
    wat::parse_str(wat_source).expect("failed to compile WAT")
}

// ---- Tests ----

#[test]
fn test_plugin_load_and_manifest() {
    let wasm = compile_wat(PASSTHROUGH_WAT);
    let mut pm = PluginManager::new().unwrap();
    let manifest = pm.load_bytes(&wasm).unwrap();

    assert_eq!(manifest.name, "passthrough");
    assert_eq!(manifest.version, "0.1.0");
    assert_eq!(manifest.hooks, vec!["post_som"]);
    assert_eq!(pm.plugin_count(), 1);
}

#[test]
fn test_plugin_noop_manifest() {
    let wasm = compile_wat(NOOP_WAT);
    let mut pm = PluginManager::new().unwrap();
    let manifest = pm.load_bytes(&wasm).unwrap();

    assert_eq!(manifest.name, "noop");
    assert_eq!(manifest.version, "1.0.0");
    assert_eq!(
        manifest.hooks,
        vec!["pre_navigate", "post_parse", "post_som", "on_extract"]
    );
}

#[test]
fn test_plugin_passthrough_preserves_data() {
    let wasm = compile_wat(PASSTHROUGH_WAT);
    let mut pm = PluginManager::new().unwrap();
    pm.load_bytes(&wasm).unwrap();

    let input = br#"{"som_version":"0.1","url":"https://example.com","title":"Test","lang":"en","regions":[],"meta":{"html_bytes":100,"som_bytes":50,"element_count":0,"interactive_count":0}}"#;

    let output = pm.run_hook(Hook::PostSom, input).unwrap();
    assert_eq!(output, input.to_vec());
}

#[test]
fn test_plugin_noop_returns_original() {
    let wasm = compile_wat(NOOP_WAT);
    let mut pm = PluginManager::new().unwrap();
    pm.load_bytes(&wasm).unwrap();

    let input = b"hello world";
    let output = pm.run_hook(Hook::PostParse, input).unwrap();
    // Noop returns len=0, so run_hook should return the original input
    assert_eq!(output, input.to_vec());
}

#[test]
fn test_plugin_uppercase_modifies_html() {
    let wasm = compile_wat(UPPERCASE_WAT);
    let mut pm = PluginManager::new().unwrap();
    pm.load_bytes(&wasm).unwrap();

    let html = "<html><body>hello</body></html>";
    let result = pm.run_post_parse(html).unwrap();
    assert_eq!(result, "<HTML><BODY>HELLO</BODY></HTML>");
}

#[test]
fn test_plugin_hook_not_registered() {
    let wasm = compile_wat(PASSTHROUGH_WAT);
    let mut pm = PluginManager::new().unwrap();
    pm.load_bytes(&wasm).unwrap();

    // Passthrough only registers for post_som (4), not pre_navigate (1)
    assert!(!pm.has_hook(Hook::PreNavigate));
    assert!(pm.has_hook(Hook::PostSom));

    // Running a hook that no plugin handles returns the input unchanged
    let url = "https://example.com";
    let result = pm.run_pre_navigate(url).unwrap();
    assert_eq!(result, url);
}

#[test]
fn test_plugin_error_propagates() {
    let wasm = compile_wat(ERROR_WAT);
    let mut pm = PluginManager::new().unwrap();
    pm.load_bytes(&wasm).unwrap();

    let input = b"test data";
    let result = pm.run_hook(Hook::PostSom, input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("error code 1"));
}

#[test]
fn test_plugin_logging() {
    let wasm = compile_wat(LOGGING_WAT);
    let mut pm = PluginManager::new().unwrap();
    pm.load_bytes(&wasm).unwrap();

    let input = b"test";
    let _output = pm.run_hook(Hook::PostSom, input).unwrap();
    // The host_log call was made — we verify the plugin ran without crashing.
    // (Log messages are consumed internally by the runtime.)
}

#[test]
fn test_plugin_chain_multiple_plugins() {
    let passthrough_wasm = compile_wat(PASSTHROUGH_WAT);
    let noop_wasm = compile_wat(NOOP_WAT);

    let mut pm = PluginManager::new().unwrap();
    pm.load_bytes(&passthrough_wasm).unwrap();
    pm.load_bytes(&noop_wasm).unwrap();

    assert_eq!(pm.plugin_count(), 2);

    let manifests = pm.manifests();
    assert_eq!(manifests[0].name, "passthrough");
    assert_eq!(manifests[1].name, "noop");

    // Both handle post_som — passthrough returns data, noop returns nothing.
    // Final output should be the passthrough's output (unchanged).
    let input = b"chain test";
    let output = pm.run_hook(Hook::PostSom, input).unwrap();
    assert_eq!(output, input.to_vec());
}

#[test]
fn test_plugin_post_som_roundtrip() {
    let wasm = compile_wat(PASSTHROUGH_WAT);
    let mut pm = PluginManager::new().unwrap();
    pm.load_bytes(&wasm).unwrap();

    // Create a real SOM and round-trip it through the plugin
    let som = plasmate::som::types::Som {
        som_version: "0.1".to_string(),
        url: "https://example.com".to_string(),
        title: "Plugin Test".to_string(),
        lang: "en".to_string(),
        regions: vec![plasmate::som::types::Region {
            id: "r_main_0".to_string(),
            role: plasmate::som::types::RegionRole::Main,
            label: None,
            action: None,
            method: None,
            elements: vec![plasmate::som::types::Element {
                id: "e_heading_1".to_string(),
                role: plasmate::som::types::ElementRole::Heading,
                text: Some("Hello World".to_string()),
                label: None,
                actions: None,
                attrs: None,
                children: None,
                hints: None,
            }],
        }],
        meta: plasmate::som::types::SomMeta {
            html_bytes: 100,
            som_bytes: 50,
            element_count: 1,
            interactive_count: 0,
        },
        structured_data: None,
    };

    let result_som = pm.run_post_som(som).unwrap();
    assert_eq!(result_som.title, "Plugin Test");
    assert_eq!(result_som.regions.len(), 1);
    assert_eq!(result_som.regions[0].elements[0].text.as_deref(), Some("Hello World"));
}

#[test]
fn test_plugin_missing_export_error() {
    // A module that's missing required exports should fail to load
    let bad_wat = r#"
    (module
      (memory (export "memory") 1)
      (func (export "malloc") (param i32) (result i32) (i32.const 0))
    )
    "#;
    let wasm = compile_wat(bad_wat);
    let mut pm = PluginManager::new().unwrap();
    let result = pm.load_bytes(&wasm);
    assert!(result.is_err());
}

#[test]
fn test_plugin_large_data() {
    let wasm = compile_wat(PASSTHROUGH_WAT);
    let mut pm = PluginManager::new().unwrap();
    pm.load_bytes(&wasm).unwrap();

    // Test with a larger payload (32KB of data — fits in one Wasm page with the allocator)
    let large_input: Vec<u8> = (0..32768).map(|i| b"abcdefghij"[i % 10]).collect();
    let output = pm.run_hook(Hook::PostSom, &large_input).unwrap();
    assert_eq!(output.len(), large_input.len());
    assert_eq!(output, large_input);
}

#[test]
fn test_hook_from_bits() {
    assert_eq!(Hook::from_bits(0), vec![]);
    assert_eq!(Hook::from_bits(1), vec![Hook::PreNavigate]);
    assert_eq!(Hook::from_bits(4), vec![Hook::PostSom]);
    assert_eq!(
        Hook::from_bits(15),
        vec![
            Hook::PreNavigate,
            Hook::PostParse,
            Hook::PostSom,
            Hook::OnExtract
        ]
    );
    assert_eq!(
        Hook::from_bits(5),
        vec![Hook::PreNavigate, Hook::PostSom]
    );
}

#[test]
fn test_plugin_with_pipeline() {
    // Test that plugins integrate correctly with the pipeline
    let wasm = compile_wat(PASSTHROUGH_WAT);
    let mut pm = PluginManager::new().unwrap();
    pm.load_bytes(&wasm).unwrap();

    let config = plasmate::js::pipeline::PipelineConfig {
        execute_js: false,
        ..Default::default()
    };
    let html = r#"<html><body><h1>Test</h1><p>Content</p></body></html>"#;
    let result =
        plasmate::js::pipeline::process_page_with_plugins(html, "https://example.com", &config, &mut pm)
            .unwrap();

    assert!(result.som.meta.element_count > 0);
}
