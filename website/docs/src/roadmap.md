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

### 2026-05-13 Action-State Conformance Adjustment

The latest Browserbase/Stagehand and Playwright MCP messaging makes action state a retention feature: agents need the current snapshot to tell them which controls are usable before they reuse a plan. Plasmate should treat inherited native disabled state as part of the same public contract as ARIA state.

- **Inherited disabled state matters**: controls inside disabled fieldsets should expose `attrs.disabled` directly, not only through a parent group.
- **Fixtures are adapter glue**: shared conformance cases should cover native inheritance and ARIA promotion so parser packages, SDKs, and integrations can test the same action surface.
- **Plan reuse beats raw DOM recovery**: compact action targets should carry enough state for agents to skip unavailable controls without asking for a full DOM traversal.

### 2026-05-13 Action-Plan Availability Adjustment

Current competitor docs make action menus the retention surface. Playwright MCP refs are only valid against the current snapshot, Stagehand `observe()` returns actions that teams cache and validate, and Firecrawl/Browser Use are broadening managed browser sessions around that workflow. Plasmate's wedge remains local SOM portability, so compact action plans should expose availability directly in every SDK.

- **Availability is a first-class plan field**: action targets should include `enabled` and `blocked_reason` so agents can gate execution without bespoke attrs checks.
- **Cross-language parity reduces churn**: Python, Node, and Go planners should return the same shape for disabled targets because teams mix these runtimes in real agent systems.
- **Framework adapters are next**: Browser Use, LangChain, and Vercel AI integrations should forward availability state instead of making downstream agents rediscover it.

### 2026-05-13 Framework Adapter Availability Adjustment

The current market keeps pushing action planning toward the framework edge: Playwright MCP snapshots expose current refs, Stagehand action caches reward stable target descriptions, Firecrawl Interact and Browser Use Cloud make hosted browsers easy, and Cloudflare Browser Run is adding MCP/CDP/WebMCP distribution around managed sessions. Plasmate's retention path remains local-first portability, so adapters should make disabled and required action state visible before an agent spends a tool call on a dead control.

- **Adapters are product surface**: Browser Use and LangChain context strings should render the same availability, description, group, and required fields exposed by parser action plans.
- **Prompt helpers reduce misuse**: Vercel AI users should get a small exported guidance string that tells models to honor SOM `enabled` and `blocked_reason` fields.
- **Next conformance step**: shared adapter fixtures should verify that framework output does not regress from the parser/SDK action-plan contract.

### 2026-05-13 Cross-Adapter Fixture Adjustment

Current competitor pressure makes adapter consistency a retention issue. Playwright MCP snapshots, Stagehand action caching, and hosted browser traces all teach users to expect the current action surface to be trustworthy. Plasmate's local-first answer should be a shared adapter fixture suite that keeps every framework aligned with the same compact SOM contract.

- **Fixtures beat prose**: Browser Use, LangChain, Vercel AI, parser packages, and SDKs should test availability, required, group, type, and description fields against the same SOM fixture.
- **Enabled is the default action state**: adapters should mark interactive targets as enabled unless SOM explicitly blocks them.
- **Helpers should filter action menus**: Vercel AI apps need a small runtime helper for cached action plans, not only prompt guidance.

### 2026-05-13 Vercel AI Action Menu Adjustment

Competitor docs keep moving reusable page state into app workflows: Playwright MCP keeps refs tied to fresh snapshots, Stagehand `observe()` plans cacheable actions, and Browserbase foregrounds cached selectors plus observability. Plasmate should keep the local-first wedge and make Vercel AI apps treat SOM action plans as a first-class menu before the model spends tokens.

- **Blocked means unavailable**: helper APIs should treat any `blocked_reason` as an execution gate, not just disabled controls.
- **Prepare menus before prompting**: apps should normalize, filter, and cap action targets before handing them to `generateText` or `streamText`.
- **Prompt formatting is product surface**: compact action-plan text should preserve ids, roles, labels, actions, availability, required state, groups, and descriptions.

### 2026-05-13 Vercel AI SOM Extraction Adjustment

Official docs keep validating action menus as the retention layer: Playwright MCP snapshots return fresh refs, Stagehand v3 `observe()` creates cacheable structured actions, Firecrawl Interact and Browser Use Cloud package managed sessions, and Cloudflare Browser Run/WebMCP is testing typed browser-native tools. Plasmate should keep the local-first wedge but make raw SOM responses directly useful in app code.

- **Raw SOM should become an action menu**: Vercel AI apps should derive compact targets from SOM without hand-walking nested regions.
- **Shadow roots count at the framework edge**: extraction helpers should traverse `children` and `shadow.elements`.
- **Runtime fixture coverage is a release gate**: Vercel AI should test extraction, filtering, and prompt formatting against the shared adapter fixture.

### 2026-05-13 Deterministic Action Cache-Key Adjustment

Reusable action memory is now part of the category expectation. Playwright MCP refs stay tied to fresh snapshots, while Stagehand/Browserbase action caching makes repeated workflows cheaper after first observation. Plasmate should keep local SOM ids as execution targets and add deterministic action keys so apps can cache, dedupe, and compare repeated actions without hosted selector memory.

- **Cache keys complement ids**: `cache_key` gives apps a stable value for local action-plan storage, prompt dedupe, and trace correlation.
- **Parser parity first**: Python and Node parser packages should emit the same cache-key contract as framework helpers.
- **Adapters inherit the contract**: Browser Use, LangChain, Vercel AI, and Go should converge on one compact action target shape.

### 2026-05-13 Action Cache-Key Parity Adjustment

Current browser-agent competitors are making action memory part of daily app code. Playwright MCP exposes fresh refs, Stagehand/Browserbase cache resolved actions, Firecrawl Interact and Browser Use Cloud make hosted browser sessions easy to reuse, and WebMCP experiments point toward typed browser-native tools. Plasmate should keep the local-first wedge by making cacheable action targets portable across all high-use SDK and framework surfaces.

- **Go is part of the action contract**: durable worker services should get the same `cache_key` field and helper as Python/Node orchestration code.
- **Prompt context should show cache identity**: Browser Use and LangChain text outputs should render cache keys beside availability so repeated workflows can dedupe targets without raw SOM recovery.
- **Shared fixtures are the next guardrail**: cache-key parity should move from focused adapter tests into a cross-adapter fixture runner.

### 2026-05-13 Shared Expectation Manifest Adjustment

The market now rewards tools that make reusable action surfaces boringly consistent. Playwright MCP refs, Stagehand cached actions, and Browserbase or Cloudflare traces all set user expectations that the current action contract can be trusted. Plasmate's broad repo surface should turn that into an advantage by keeping adapter tests wired to a single expected action manifest.

- **One fixture, one contract**: Browser Use, LangChain, and Vercel AI should consume the same expected ids, labels, availability, blocked reasons, cache keys, required flags, groups, and descriptions.
- **Drift should fail centrally**: when action-plan semantics change, the SOM fixture and expected manifest should change together instead of silently updating hard-coded assertions in each adapter.
- **Next release gate**: extend the manifest into parser packages and SDKs, then wrap all checks in one release command.

### 2026-05-13 SDK Manifest Conformance Adjustment

Competitors are making reusable action state inspectable and cacheable at the application edge. Plasmate should turn its local action surface into a cross-language contract before adding more workflow-memory features.

- **SDKs should plan actions too**: Python and Node client SDKs need compact action-plan helpers because many apps consume SOM directly from MCP calls.
- **The manifest must cover runtimes**: parser packages, Go SDK, Python SDK, and Node SDK should read the same expected action target manifest as framework adapters.
- **Release automation is now the bottleneck**: after manifest parity lands, the next sticky step is one command that runs adapter, parser, and SDK fixture checks together.

### 2026-05-13 Action Manifest Release-Gate Adjustment

Playwright MCP, Stagehand, and Firecrawl all reinforce that reusable action state must be trustworthy at the moment an agent acts. Plasmate should make local conformance a release feature: one command should prove Browser Use, LangChain, Vercel AI, parser packages, and SDKs still agree on the shared action manifest.

- **One command should prove the contract**: adapters and SDKs need a shared release gate for the action availability manifest.
- **Package tests must include fixture parity**: Node SDK action-plan tests should run from `npm test`.
- **CI is the next guardrail**: after dependency setup, the release command should become a required workflow job.

### 2026-05-13 CI Action-Manifest Adjustment

The latest competitor read keeps pointing to one durable retention hook: agents stay with browser tools when action state is safe to reuse. Playwright MCP refs, Stagehand local/server action caches, Firecrawl Interact sessions, Browser Use Cloud profiles, and Cloudflare WebMCP all make the action surface feel like product infrastructure. Plasmate's local-first answer should be to make cross-runtime conformance cheap enough to run continuously.

- **CI should catch contract drift early**: the shared action manifest now needs a required pull-request path, not only a maintainer release command.
- **Fast and full gates serve different jobs**: quick mode should prove the single manifest contract on every change, while full mode remains the local pre-release check for broader action-plan behavior.
- **Next leverage is caching**: once the quick gate is stable, tune dependency caches and promote more shared fixtures without making CI adoption painful.

### 2026-05-13 Semantic Fidelity Polish Adjustment

Competitor docs keep turning browser state into reusable action contracts: Playwright MCP snapshots expose accessibility roles and refs, Stagehand `observe()` plus action caching rewards stable target descriptions, Firecrawl Browser Sandbox and Browser Use Cloud package managed execution, Crawl4AI is moving open-source crawling toward cloud extraction, and Cloudflare WebMCP is testing typed website-provided tools. Plasmate should keep the local-first wedge, but small semantics now determine whether an agent trusts SOM without raw DOM recovery.

- **Search is a landmark, not generic content**: ARIA `role="search"` should compile into a labelled region so agents can scope query tasks reliably.
- **Menus carry actionable state**: ARIA `menuitemcheckbox` and `menuitemradio` should map to checkbox/radio action targets before framework adapters consume the page.
- **Noise stripping must tolerate production CSS**: visibility parsing should ignore casing and arbitrary whitespace in stylesheet declarations, matching the inline-style hardening already shipped.

### 2026-05-13 Action-Semantics Fixture Adjustment

Current browser-agent comparisons keep confirming that reusable action state is only sticky when downstream app code can trust it without engine-specific knowledge. Browser Use and Stagehand make action menus developer-facing, Playwright MCP makes structured refs the interaction unit, and hosted browser tools sell traces and session reuse around the same contract. Plasmate should promote semantic fixes into shared fixtures as soon as they land.

- **Menu widgets belong in the manifest**: ARIA menu checkbox/radio targets should appear in the shared action-availability fixture before adapters treat them as reusable actions.
- **Search and visibility need one fixture**: search landmarks and stylesheet-hidden whitespace are common SaaS cases that should be tested together with action targets.
- **Docs fixtures need executable guards**: conformance fixtures should have focused Rust coverage first, then graduate into parser, SDK, and adapter release gates.

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
- [x] Disabled native fieldset state propagates to descendant native controls
- [x] Shared conformance fixture added for disabled/required action state
- [x] Action-plan availability fields across Python parser, Node parser, and Go SDK
- [x] Browser Use action-plan helper and availability-aware page context
- [x] LangChain availability-aware SOM text output
- [x] Vercel AI action availability guidance helper
- [x] Shared adapter action-availability fixture for Browser Use and LangChain
- [x] LangChain enabled-state fallback for normal interactive targets
- [x] Vercel AI action-target availability helper
- [x] Vercel AI action-menu normalization, filtering, and formatting helpers
- [x] Vercel AI typecheck fixture for compact action-plan helper parity
- [x] Vercel AI SOM-to-action-target extraction helper
- [x] Vercel AI runtime fixture test for extraction, filtering, and formatting
- [x] Vercel AI deterministic action target cache keys
- [x] Python and Node parser deterministic action-plan cache keys
- [x] Go SDK deterministic action-plan cache keys
- [x] Browser Use and LangChain action cache-key prompt rendering
- [x] Shared action-availability expectation manifest for Browser Use, LangChain, and Vercel AI
- [x] Browser Use and LangChain package version exports match package metadata
- [x] Python SDK compact action-plan helpers with deterministic cache keys
- [x] Node SDK compact action-plan helpers with deterministic cache keys
- [x] Shared action-availability expectation manifest for Python parser, Node parser, Go SDK, Python SDK, and Node SDK
- [x] One release command for Browser Use, LangChain, Vercel AI, parser-package, and SDK fixture checks
- [x] Node SDK `npm test` runs action-plan fixture coverage
- [x] Root and fixture docs advertise the shared action-manifest release gate
- [x] Quick/full modes for the shared action-manifest release gate
- [x] GitHub Actions conformance job for the quick action-manifest gate
- [x] ARIA search landmarks compile into labelled SOM navigation regions
- [x] ARIA menuitem checkbox/radio roles compile into actionable controls
- [x] Stylesheet hidden-rule parsing tolerates arbitrary whitespace and casing
- [x] Case-sensitive URL path dedup contract covered by integration tests
- [x] Shared action-availability manifest covers ARIA menu checkbox/radio targets
- [x] Shared conformance fixture covers search landmarks, ARIA menu targets, and stylesheet hidden whitespace
- [x] Rust compiler test loads the action-semantics conformance fixture
- [ ] Selector-aware SOM cache entries for repeated agent prompts
- [ ] Session replay/trace export for debugging agent runs
- [ ] Wire `016-action-semantics` into parser/SDK and adapter conformance runners
- [ ] Promote shadow-DOM and web-component cases into shared cross-adapter fixtures
- [ ] Add cross-adapter fixtures for enriched compact action-plan metadata
- [ ] Add cross-adapter accessible-description fixtures
- [ ] Wire disabled/required action-state fixtures into cross-adapter parser/SDK conformance runners
- [x] Promote adapter availability checks into shared cross-adapter fixtures
- [x] Add runtime Vercel AI fixture tests once the package has a local test runner
- [x] Extend shared action-availability expectations into parser-package and SDK conformance tests
- [ ] Promote the action-manifest CI job from quick checks to full conformance after dependency-cache tuning
- [ ] WebMCP/watchlist research spike: track whether browser-native tool exposure changes SOM adapter strategy
