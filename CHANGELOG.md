# Changelog

## v0.3.0 (2026-03-22)

### SPA Rendering Bridge
- Live DOM bridge: V8 JavaScript mutations now flow into the real rcdom tree
- NodeRegistry with bidirectional V8-DOM bindings (14 native callbacks)
- CSS selector engine for querySelector/querySelectorAll
- SOM recompiled from JS-modified DOM tree after script execution
- DOM shim expanded: createTreeWalker, createRange, Observer stubs, navigator, window APIs

### Screenshot Support
- `plasmate screenshot <url>` CLI command
- Page.captureScreenshot CDP method (returns SOM fallback until renderer lands)
- Screenshot support in AWP and MCP protocols

### Parallel Sessions
- SessionManager for concurrent page processing (up to 50 sessions)
- CDP multi-target support with independent page contexts
- Thread-safe session storage with idle timeout and memory tracking

### Network & Security
- Network request interception (block, modify, mock responses)
- TLS fingerprint configuration (cipher suites, version control)
- CDP cookie jar (Network.getCookies, setCookies, deleteCookies, clearBrowserCookies)

### Plugin System
- Wasm plugin runtime (wasmtime-based)
- 8 plugin types: page_transform, request_intercept, response_transform, dom_mutate, som_post_process, auth_provider, cache_strategy, analytics

### Coverage & Benchmarks
- 100-URL benchmark suite (98 sites tested)
- HTML coverage: 95.9% full rendering
- JS coverage: 95.9% full rendering
- Median SOM compression: 9.05x
- Nightly HTML + weekly JS coverage CI

### Other
- Browser-realistic HTTP headers for anti-bot compatibility
- URL/URLSearchParams polyfill improvements
- External module script handling
- Pre/post JS SOM comparison (keep best result)

## v0.1.1 (2026-03-18)

### Added
- Cookie-based auth profiles for authenticated browsing
- `--profile` flag on serve for authenticated sessions
- Extension CLI bridge server with AES-256-GCM encryption at rest
- Privacy policy page for Chrome Web Store submission
- `/api/wait` endpoint for agent-driven auth flow
- Top-100 coverage scorecard with configurable JS safety budgets
- Functional MutationObserver for SPA framework support

### Fixed
- Browser-realistic HTTP headers to avoid anti-bot blocking
- Pre/post JS SOM comparison to prevent JS content degradation
- Robust URL/URLSearchParams polyfills
- Strip macOS quarantine flag in install script

## v0.1.0 (2026-03-17)

- Initial release
- Headless browser engine with Semantic Object Model (SOM)
- CDP-compatible WebSocket server
- AWP and MCP protocol support
- JavaScript execution via V8
- HTML parsing and rendering pipeline
