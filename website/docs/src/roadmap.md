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

### 2026-05-06 Roadmap Adjustment

The market is moving from "browser access" toward agent-ready page state: Playwright MCP has made structured refs familiar, Stagehand's `observe()` and action caching promise deterministic repeated actions, Firecrawl's MCP surface now includes interaction/browser sessions, and Skyvern keeps differentiating on visual workflow completion. Plasmate should keep the local-first wedge and increase stickiness by making SOM output more action-complete:

- **Actionability metadata**: preserve `contenteditable`, `tabindex`, form names, autocomplete hints, and ARIA states so agents can plan custom SaaS controls without falling back to raw DOM.
- **Correct URL semantics**: cache and compiler deduplication must preserve case-sensitive paths while normalizing only the parts of URLs that are actually case-insensitive.
- **Robust MCP surfaces**: helper tools should never panic on multilingual content or partial token budgets; UTF-8-safe truncation is table stakes for global web pages.

### 2026-05-07 Roadmap Adjustment

Competitor pressure is expanding from structured snapshots into durable workflow memory and full browser surfaces. Playwright MCP keeps stable accessibility refs at the center of interaction, Stagehand v3 now makes `observe()` planning, action caching, and targeted iframe/shadow-root operation part of its core story, and Firecrawl/Browser Use are selling managed browser sessions and persistent cloud profiles. Plasmate should keep the local-first wedge by making SOM contracts complete and portable across adapters.

- **Schema parity before new adapters**: JSON Schema, parser packages, SDKs, and integrations must accept the same SOM shape the Rust compiler emits.
- **Web-component reachability**: shadow-root elements should be discoverable by id, role, text, link, and actionability helpers in every language.
- **Conformance as distribution**: the large repo surface is a growth asset only when downstream adapters stay thin, current, and release-tested.

### 2026-05-09 Roadmap Adjustment

The highest-retention competitor features now cluster around reusable action surfaces. Playwright MCP and Cloudflare Browser Run normalize structured snapshots with action refs, Stagehand uses `observe()` and action caching to turn repeated workflows into deterministic low-cost actions, Firecrawl now packages scrape/search/extract with agent and browser-session APIs, and Skyvern continues to bundle visual workflow completion with credential management. The roadmap should increase stickiness by making SOM the local action-planning layer:

- **Action-plan helpers everywhere**: SDKs should expose compact action targets so agents can choose from SOM ids, roles, labels, and actions without bespoke tree traversal.
- **Hint/action conformance**: `actions` and `hints` are now public contract, not incidental metadata. Shared fixtures should verify them across Rust, Python, Node, Go, and integrations.
- **Cloud-optional workflow memory**: keep local cache/diff as the wedge, then add optional trace exports and cache observability before considering hosted browser infrastructure.

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
- [x] Case-preserving SOM link deduplication
- [x] Case-tolerant input type and ARIA role parsing
- [x] Custom-control actionability attrs (`contenteditable`, `tabindex`, `name`, `autocomplete`)
- [x] UTF-8-safe MCP text truncation
- [x] SOM Schema parity for `shadow`, `iframe`, `details`, ARIA state, and actionability attrs
- [x] Shadow-root query coverage across Python/Node SDK and parser packages
- [x] Action and hint query helpers across Python/Node parser packages
- [x] Compact action-plan helpers across Python/Node parser packages
- [x] Node parser compression-ratio parity for zero-byte SOM edge cases
- [ ] Selector-aware SOM cache entries for repeated agent prompts
- [ ] Session replay/trace export for debugging agent runs
- [ ] Promote shadow-DOM and web-component cases into shared cross-adapter fixtures
- [ ] Promote action-plan helper parity into the Go SDK and framework integrations
- [ ] WebMCP/watchlist research spike: track whether browser-native tool exposure changes SOM adapter strategy
