# Changelog

All notable changes to Plasmate, tracked by version and date.

## v0.4.0 - 2026-03-23

Chrome-delegated screenshots, completed SPA hydration, and full CDP stub coverage for Puppeteer compatibility.

- **feat** - **Chrome-delegated screenshots** for pixel-perfect rendering via `Page.captureScreenshot`.
- **feat** - **Completed SPA hydration ops:** `insertBefore`, `replaceChild`, `classList`, `cloneNode`.
- **feat** - **Timer queue drain** for `setTimeout` and `requestAnimationFrame`.
- **feat** - **Wired `page.click()` and `page.type()`** via DOM bridge.
- **feat** - **Implemented `page.waitForSelector()`** for final DOM state.
- **fix** - **Fixed all remaining CDP stubs**, including `setDeviceMetricsOverride`, `addScriptToEvaluateOnNewDocument`, `getLayoutMetrics`, and `getProperties` for improved Puppeteer compatibility.

---

## v0.3.0 - 2026-03-22

SPA rendering, interaction APIs, plugin system, and multi-session support. ~25K lines Rust, 200+ tests passing.

- **feat** - **Network request interception** - block, modify, or mock responses.
- **feat** - **TLS configuration** - cipher suite and fingerprint tuning.
- **feat** - **Wasm plugin system** - optional feature flag, wasmtime runtime.
- **feat** - **SPA rendering bridge** - bidirectional V8/rcdom DOM sync, SOM recompiled after JS.
- **feat** - **Timer queue drain** for React/Vue deferred rendering.
- **feat** - **page.click(), page.type(), page.waitForSelector()** via DOM bridge.
- **feat** - **Deep SPA hydration** - insertBefore, replaceChild, cloneNode, classList.
- **feat** - **Multi-page session manager** - 50 concurrent sessions per instance.
- **feat** - **Auth profiles** with encrypted cookie storage.
- **feat** - **Screenshots protocol surface** (SOM fallback until renderer lands).

### Stats

- **Binary:** 45.8 MB default / 54 MB with plugins
- **Memory:** 28 MB RSS per process
- **Tests:** 200+ passing
- **Codebase:** ~25K lines Rust

---

## v0.2.0 - 2026-03-21

V8 integration, CDP server, SDK expansion, and coverage tooling.

- **feat** - **V8 JavaScript integration** with full DOM shim.
- **feat** - **CDP server** for Puppeteer/Playwright compatibility.
- **feat** - **MCP server mode** (stdio JSON-RPC).
- **feat** - **SOM Specification v1.0** with JSON Schema and conformance test suite.
- **feat** - **Node.js SDK** with full TypeScript types.
- **feat** - **Python SDK** with Pydantic models.
- **feat** - **Go SDK** with structs, client, and query helpers.
- **feat** - **Throughput CI** benchmark expansion to 100 URLs across 13 categories.
- **feat** - **CDP cookie jar** (getCookies, setCookies, deleteCookies, clearBrowserCookies).
- **docs** - Browser Use and LangChain integration pages.
- **docs** - Interactive coverage scorecards (nightly HTML, weekly JS).

---

## v0.1.1 - 2026-03-20

Major JS runtime improvements. Coverage jumps from 71% to 82%.

- **feat** - Functional **MutationObserver** for SPA framework support (React, Vue, Next.js). Observer callbacks now fire on DOM mutations, enabling client-side rendering in the headless engine.
- **feat** - **URL/URLSearchParams polyfills** with try-catch constructor probing. Detects partial V8 built-ins and only overrides when necessary.
- **feat** - **JS coverage scorecard** tracking 100 real-world sites with per-site element counts, JS failure rates, and error details.
- **fix** - **Browser-realistic HTTP headers** to avoid anti-bot blocking (zero speed/memory cost).
- **fix** - **Pre/post JS SOM comparison** keeps whichever SOM has more elements, preventing JS from degrading content.
- **fix** - Skip `type="module"` external scripts (prevents `SyntaxError: Cannot use import statement outside a module`).
- **fix** - `var self = globalThis` for SPA frameworks that reference `self`.
- **perf** - Configurable JS safety budgets: `--max-external-scripts`, `--max-external-script-kb`, `--external-script-timeout-ms`.

### Coverage Results

- **HTML scorecard:** 80/100 Full (80%)
- **JS scorecard:** 79/98 Full (80.6%)
- **New Full sites:** amazon.com, ebay.com (via MutationObserver)
- **Remaining thin:** 14 sites needing deeper DOM mutation support
- **Failed:** 4 (anti-bot: etsy, tripadvisor, wsj; non-HTML: httpbin)

---

## v0.1.0 - 2026-03-01

Initial public release. The browser engine for agents.

- **feat** - **Semantic Object Model (SOM)** compiler: HTML to structured, token-efficient representation.
- **feat** - **Agent Web Protocol (AWP)**: 7-method protocol for agent-native browsing (navigate, snapshot, click, type, scroll, select, extract).
- **feat** - **V8 JavaScript runtime** with DOM shim for JS-rendered pages.
- **feat** - **CDP compatibility layer** for Puppeteer/Playwright drop-in usage.
- **feat** - **MCP server mode** (`plasmate mcp`) with stateful tools: open_page, evaluate, click, close_page.
- **feat** - **Cookie-based auth profiles** with AES-256-GCM encryption at rest.
- **feat** - **Chrome extension bridge** for importing auth sessions from a real browser.
- **feat** - **Docker image** on GHCR (multi-arch).
- **feat** - **Node.js and Python SDKs**.
- **perf** - 4-5ms per page (50x faster than Chrome, 5x faster than Lightpanda).
- **perf** - ~30MB memory for 100 concurrent pages.
- **perf** - 10.4x token compression vs raw DOM.

### Architecture

- **Language:** Rust (16,740 lines at launch, 171 tests)
- **Binary:** 43 MB single file, no dependencies
- **License:** Apache 2.0
