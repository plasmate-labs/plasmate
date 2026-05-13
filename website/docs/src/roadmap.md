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

### 2026-05-10 Roadmap Adjustment

The browser-agent market keeps rewarding structured state that can be reused without another model call. Playwright MCP and Cloudflare Browser Run emphasize accessibility snapshots over screenshots, Stagehand centers `observe()` plus local/managed action caching, Firecrawl keeps broad hosted browser-session breadth, and Skyvern owns visual workflows. Plasmate should keep the local-first wedge by making SOM output more accurate and portable across the current repo surface.

- **Accessible-name parity**: controls must carry names from `aria-labelledby` and external labels so agents can reuse plans reliably.
- **Parser tolerance as adoption polish**: SDK/parser helpers should accept real CLI/MCP payload shapes, including wrapped SOM objects and progress lines.
- **Conformance before breadth**: small core improvements should land with Rust, Python, Node, and docs coverage before adding more integrations.

### 2026-05-11 Roadmap Adjustment

Current official docs reinforce that browser-agent products are competing on usable page state, not raw transport. Playwright MCP centers accessibility snapshots and stable refs, Stagehand centers `observe()` actions that can be validated and cached, and Firecrawl/Browser Use make cloud sessions and persistent profiles convenient for teams buying infrastructure. Plasmate should keep the local-first lane and make SOM output more complete, deterministic, and verifiable.

- **Accessible descriptions and names**: labels and descriptions are part of the action contract because agents choose controls by human-facing text.
- **Full-tree accounting**: metadata, cache prefetch, MCP helpers, parser packages, and SDK helpers must all agree on shadow-root and nested content.
- **Fixture-driven trust**: ARIA-heavy SaaS forms, web components, and repeated workflow pages should become shared conformance fixtures before adding more adapters.

### 2026-05-11 Go SDK Parity Adjustment

The repo's broad library surface is now a product promise. Python and Node already expose action/hint lookup and compact action-plan helpers, while Go was still missing current SOM fields and shadow-root traversal. That gap matters because multi-service teams often adopt Go for durable workers and Python/Node for agent orchestration; if the same SOM cannot be queried consistently across those services, Plasmate becomes less sticky.

- **Cross-language action plans**: Go should expose the same compact action targets as the parser packages so agents can plan from ids, roles, labels, actions, hrefs, names, and input types in any supported runtime.
- **Shadow roots are not optional**: web-component controls must be reachable by id, role, text, interactivity, and flattened traversal in Go as well as Python and Node.
- **Schema fields need SDK homes**: `attrs.description`, `attrs.name`, `attrs.autocomplete`, ARIA state, details attrs, iframe attrs, and `shadow` should be treated as public contract across all SDKs.

### 2026-05-11 Browser Run and Naming Adjustment

Cloudflare's Browser Run launch strengthens the trend toward browser platforms that pair hosted sessions with Live View, recordings, human-in-loop, MCP/CDP, and structured extraction. Plasmate should keep the local-first lane by making SOM the most trustworthy portable action snapshot.

- **Browser-like names for every target**: wrapped labels, region `aria-labelledby`, and input-button values should compile into the same human-facing names agents see in accessibility snapshots.
- **Trace and cache over hosted scale**: repeated local workflows need selector-aware cache views and trace exports before a managed browser cloud would add durable retention.
- **Conformance for SaaS forms**: shared fixtures should cover labels, descriptions, regions, fieldsets, and button values because form automation is where repeat users feel reliability or churn.

### 2026-05-12 Form Semantics Adjustment

Current competitor docs keep pushing the same retention lesson: agents stick with browser tools that expose reusable action state, not just pixels or raw HTML. Playwright MCP's accessibility snapshots train agents to rely on named controls, Stagehand's `observe()` and caching make repeated form flows cheaper, and Cloudflare Browser Run plus Browser Use Cloud make hosted scale easy to buy. Plasmate's local-first answer should be stronger SaaS form semantics:

- **Field groups are action context**: native `<fieldset>`/`<legend>` and ARIA `group`/`radiogroup` should survive in SOM so agents understand which radio buttons and controls belong together.
- **Contract changes must cross adapters**: new roles and attrs should land in schema, spec, parser packages, SDKs, CDP mappings, and tests together.
- **Conformance becomes sales collateral**: shared fixtures for grouped forms, descriptions, regions, and button values should prove Plasmate handles the repetitive SaaS workflows teams actually automate.

### 2026-05-12 Action Plan and WebMCP Adjustment

The browser-agent category is turning structured page state into validated action menus. Playwright MCP snapshots make current refs the interaction unit, Stagehand `observe()` turns page understanding into cacheable executable actions, Firecrawl's MCP surface spans scrape/search/extract plus browser interaction, and Cloudflare Browser Run is layering CDP/MCP/WebMCP onto hosted sessions. Plasmate should keep the local-first wedge and make SOM action plans more complete before pursuing hosted scale.

- **Compact targets need context**: action plans should include placeholders, descriptions, disabled/required state, and group names.
- **Web components are first-class surfaces**: shadow-root extraction must recurse through wrapper containers.
- **Browser tolerance beats ideal markup**: ARIA roles and landmarks should be parsed with production casing tolerance.

### 2026-05-13 State Fidelity Adjustment

Current trend research reinforces a conservative wedge: production teams want deterministic browser execution with selective AI planning, structured snapshots, persistent state, and traceability. Playwright/Playwright MCP, Stagehand, Browserbase, Browser Use, Skyvern, Firecrawl, and emerging WebMCP work all validate the same direction for Plasmate: richer local SOM/action state before hosted scale.

- **State flags are action contracts**: disabled and required state must land in the same top-level attrs no matter whether markup is native HTML or ARIA-heavy SaaS UI.
- **Action menus should avoid dead controls**: compact targets are stickier when unavailable fields and dropdowns are obvious without raw DOM recovery.
- **Conformance should chase SaaS edge cases**: disabled selects/textareas, ARIA required widgets, ARIA disabled widgets, field groups, and descriptions should become shared fixtures across SDKs and integrations.

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
- [x] Accessible-name parity for `aria-labelledby` and external `<label for>` controls
- [x] Accessible descriptions from `aria-describedby` and `aria-description`
- [x] `aria-labelledby` precedence over `aria-label`
- [x] Shadow-root elements included in SOM metadata counts
- [x] `attrs.description` schema and Python/Node type parity
- [x] Python parser support for mixed CLI output around SOM JSON
- [x] Node parser support for wrapped `{ som: ... }` payloads
- [x] Go SDK parsing for `shadow`, accessible descriptions, ARIA state, details attrs, and iframe attrs
- [x] Go SDK shadow-root traversal for id, role, text, interactivity, and flattened queries
- [x] Go SDK action/hint lookup and compact action-plan helpers
- [x] Wrapped `<label>` accessible-name support without nested option text leakage
- [x] `aria-labelledby` labels for landmark and form regions
- [x] Input button value-derived labels and normalized `attrs.input_type`
- [x] Native `<fieldset>` controls and ARIA `group`/`radiogroup` widgets compile as labelled SOM `group` elements
- [x] Fieldset groups expose `attrs.legend` and preserve disabled group state
- [x] SOM schema/spec, Python/Node SDK types, Python/Node parser types, Go SDK attrs, and CDP mappings accept the `group` role and `attrs.legend`
- [x] Shared conformance fixture added for fieldset/legend and ARIA radiogroup semantics
- [x] Case-insensitive ARIA landmark role parsing for SOM regions
- [x] Nested declarative shadow-root extraction through non-semantic wrappers
- [x] Enriched compact action plans with placeholder, description, required, disabled, and group metadata across Python/Node parser packages and Go SDK
- [x] Disabled native textarea controls preserve `attrs.disabled`
- [x] Disabled native select controls preserve `attrs.disabled`
- [x] `aria-required="true"` promotes `attrs.required` for custom controls
- [x] `aria-disabled="true"` promotes `attrs.disabled` for custom controls while retaining ARIA state
- [ ] Selector-aware SOM cache entries for repeated agent prompts
- [ ] Session replay/trace export for debugging agent runs
- [ ] Promote shadow-DOM and web-component cases into shared cross-adapter fixtures
- [ ] Add cross-adapter fixtures for enriched compact action-plan metadata
- [ ] Add cross-adapter accessible-description fixtures
- [ ] Add cross-adapter fixtures for disabled/required ARIA and native form state
- [ ] Promote action-plan helper parity into framework integrations
- [ ] WebMCP/watchlist research spike: track whether browser-native tool exposure changes SOM adapter strategy
