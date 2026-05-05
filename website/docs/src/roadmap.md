# Roadmap

Plasmate's roadmap is public and standards-first. We ship compression and correctness before scale.

## 2026 Market Adjustment

Browser-agent infrastructure is converging on structured context instead of raw page dumps. Playwright MCP has normalized accessibility snapshots, Firecrawl is packaging search/scrape/browser sessions behind MCP, and Browserbase/Stagehand is pushing cached actions to reduce repeated LLM calls.

Plasmate should keep its local-first position, but the roadmap now emphasizes three sticky advantages:

- **Actionable SOM snapshots**: selectors, ARIA widget parity, and stable ids are product features agents depend on.
- **Cheaper repeated workflows**: SOM cache and diff should become the local, page-level answer to cloud selector/action caching.
- **Ecosystem distribution**: MCP, Browser Use, SDKs, and comparison pages should remain conformance-tested so partner repos do not drift.

Near-term target: make Plasmate the fastest local way to turn authenticated or repetitive web workflows into compact, inspectable, reusable state.

### 2026-05-05 Roadmap Adjustment

Current competitor pressure reinforces the same direction but raises the bar on completeness. Playwright MCP snapshots train agents to expect every actionable surface to appear in structured output, Browserbase/Stagehand caching trains operators to expect repeated flows to get cheaper, and Firecrawl's MCP/browser sessions make broad hosted extraction easy to adopt. Plasmate should answer with local-first depth:

- **Full-tree SOM fidelity**: nested content, shadow DOM, ARIA widgets, and web-component links/text must flow through every extraction path, not only the compiler.
- **Reusable local memory**: cache keys and prefetch discovery need to preserve real URL semantics, dedupe work, and feed selector-aware cache views.
- **Ecosystem conformance**: the repo now spans Rust core, MCP/CDP/AWP, Python/Node/Go SDKs, Browser Use, LangChain, Vercel AI, SOM parser packages, generated docs, comparison pages, and marketing assets. This breadth should be treated as a synchronized product surface with shared fixtures.

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
- [x] Proxy rotation (pool management, sticky sessions)
- [x] Iframe support
- [x] Shadow DOM support (declarative shadow DOM)
- [x] Full ES module support
- [x] Chrome extension on Web Store
- [x] Selector whitespace and `#region-id` support
- [x] Common ARIA widget roles mapped to actionable SOM elements
- [x] Robust hidden inline-style stripping
- [x] Full-tree cache prefetch extraction across nested and shadow DOM links
- [x] Shadow-root text/link extraction in MCP helpers
- [ ] Selector-aware SOM cache entries for repeated agent prompts
- [ ] Session replay/trace export for debugging agent runs
- [ ] Shared shadow-DOM and web-component fixtures across SDK/parser packages
- [ ] WebMCP/watchlist research spike: track whether browser-native tool exposure changes SOM adapter strategy
