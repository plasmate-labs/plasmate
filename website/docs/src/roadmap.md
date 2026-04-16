# Roadmap

Plasmate's roadmap is public and standards-first. We ship compression and correctness before scale.

## Completed (v0.1.1)

- SOM compiler with 9.4x median compression across 38 sites
- V8 JavaScript execution with full DOM shim
- AWP WebSocket server
- CDP compatibility (Puppeteer connects out of the box)
- MCP server mode (stdio JSON-RPC)
- Cookie management
- Published on crates.io, npm, PyPI
- Docker image (GHCR multi-arch)

## Completed (v0.2)

- SOM Specification v1.0 with JSON Schema and conformance test suite
- Benchmark expansion to 100 URLs across 13 categories
- Node.js SDK with full TypeScript types (npm v0.3.0)
- Python SDK with Pydantic models (PyPI v0.3.0)
- Go SDK with structs, client, and query helpers
- Browser Use integration page and docs
- LangChain integration page and docs
- Interactive coverage scorecards (nightly HTML, weekly JS)
- CDP cookie jar (getCookies, setCookies, deleteCookies, clearBrowserCookies)

## Completed (v0.3)

- SPA Rendering Bridge: V8 mutations flow into real DOM tree, SOM recompiled after JS
- NodeRegistry with bidirectional V8-DOM bindings
- CSS selector engine for querySelector/querySelectorAll
- Screenshot support wired (CLI, CDP, AWP, MCP). Renderer not shipped yet, SOM fallback used.
- Parallel Session Manager (up to 50 concurrent sessions per instance)
- CDP multi-target support with independent page contexts
- Network request interception (block, modify, mock responses)
- TLS fingerprint configuration (cipher suites, version control)
- Wasm plugin system (8 plugin types, wasmtime runtime)
- Browser-realistic HTTP headers

## Completed (v0.4)

- Deep SPA hydration ops (insertBefore, replaceChild, classList, cloneNode)
- Timer queue drain (setTimeout, requestAnimationFrame)
- page.click() / page.type() via DOM bridge
- page.waitForSelector() (final DOM state)
- Chrome-delegated Page.captureScreenshot for pixel-perfect rendering
- CDP stubs wired: setDeviceMetricsOverride, addScriptToEvaluateOnNewDocument, getLayoutMetrics, getProperties

## v0.5: Scale & Adoption (Next)

- [ ] Parallel sessions at scale (500+ concurrent per 8GB)
- [x] Proxy support (HTTP, HTTPS, SOCKS5 with auth)
- [ ] Proxy rotation (pool management, sticky sessions)
- [ ] Iframe support
- [ ] Shadow DOM support
- [ ] Full ES module support
- [ ] Chrome extension on Web Store
