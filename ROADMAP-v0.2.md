# Plasmate v0.2 Roadmap - The Full Engine

## Vision

Plasmate v0.1 proved SOM works: 9.4x median compression across 38 real sites.
v0.2 makes it a drop-in replacement for Lightpanda and Chrome headless.

Three pillars: **Speed**, **Memory**, **Parallelism** - all powered by SOM-native architecture.

## 2026 Market Adjustment

The browser-agent market is moving from raw browser control toward structured,
repeatable context. Playwright MCP exposes accessibility snapshots without
vision models, Firecrawl is pushing hosted MCP scraping and browser sessions,
and Browserbase/Stagehand is making action caching a headline feature for
latency and cost reduction. Plasmate should stay local-first and open, but the
roadmap needs to make three sticky advantages obvious:

1. **Structured snapshots agents can act on**: SOM selectors, ARIA widget parity,
   and stable element ids should be treated as product surface, not internals.
2. **Repeatable workflows get cheaper over time**: SOM cache and diff should be
   positioned against Stagehand-style action caching, with page-level and
   element-level reuse rather than cloud-hosted selector memory.
3. **Distribution through agent ecosystems**: MCP, Browser Use, SDKs, and
   comparison pages are growth channels. Keep adapters small, documented, and
   conformance-tested so downstream repos do not drift.

Near-term stickiness target: developers should keep Plasmate installed because
it becomes the fastest local way to turn authenticated or repetitive web
workflows into compact, inspectable, reusable state.

### 2026-05-05 Roadmap Adjustment

Current competitor pressure reinforces the same direction but raises the bar on
completeness. Playwright MCP snapshots train agents to expect every actionable
surface to appear in structured output, Browserbase/Stagehand caching trains
operators to expect repeated flows to get cheaper, and Firecrawl's MCP/browser
sessions make broad hosted extraction easy to adopt. Plasmate should answer with
local-first depth:

1. **Full-tree SOM fidelity**: nested content, shadow DOM, ARIA widgets, and
   web-component links/text must flow through every extraction path, not only
   the compiler.
2. **Reusable local memory**: cache keys and prefetch discovery need to preserve
   real URL semantics, dedupe work, and feed selector-aware cache views.
3. **Ecosystem conformance**: the repo now spans Rust core, MCP/CDP/AWP,
   Python/Node/Go SDKs, Browser Use, LangChain, Vercel AI, SOM parser packages,
   generated docs, comparison pages, and marketing assets. This breadth should
   be treated as a synchronized product surface with shared fixtures.

### 2026-05-06 Roadmap Adjustment

The market is moving from "browser access" toward agent-ready page state:
Playwright MCP has made structured refs familiar, Stagehand's `observe()` and
action caching promise deterministic repeated actions, Firecrawl's MCP surface
now includes interaction/browser sessions, and Skyvern keeps differentiating on
visual workflow completion. Plasmate should keep the local-first wedge and
increase stickiness by making SOM output more action-complete:

1. **Actionability metadata**: preserve `contenteditable`, `tabindex`, form
   names, autocomplete hints, and ARIA states so agents can plan custom SaaS
   controls without falling back to raw DOM.
2. **Correct URL semantics**: cache and compiler deduplication must preserve
   case-sensitive paths while normalizing only the parts of URLs that are
   actually case-insensitive.
3. **Robust MCP surfaces**: helper tools should never panic on multilingual
   content or partial token budgets; UTF-8-safe truncation is table stakes for
   global web pages.

### 2026-05-07 Roadmap Adjustment

Competitor pressure is expanding from structured snapshots into durable
workflow memory and full browser surfaces. Playwright MCP keeps stable
accessibility refs at the center of interaction, Stagehand v3 now makes
`observe()` planning, action caching, and targeted iframe/shadow-root operation
part of its core story, and Firecrawl/Browser Use are selling managed browser
sessions and persistent cloud profiles. Plasmate should not pivot into hosted
browser infrastructure as the main wedge. The higher-retention move is to make
local SOM contracts complete and portable:

1. **Schema parity before new adapters**: JSON Schema, parser packages, SDKs,
   and integrations must accept the same SOM shape the Rust compiler emits.
2. **Web-component reachability**: shadow-root elements should be discoverable
   by id, role, text, link, and actionability helpers in every language.
3. **Conformance as distribution**: the large repo surface is a growth asset
   only when downstream adapters stay thin, current, and release-tested.

### 2026-05-09 Roadmap Adjustment

The highest-retention competitor features now cluster around reusable action
surfaces. Playwright MCP and Cloudflare Browser Run normalize structured
snapshots with action refs, Stagehand uses `observe()` and action caching to
turn repeated workflows into deterministic low-cost actions, Firecrawl now
packages scrape/search/extract with agent and browser-session APIs, and Skyvern
continues to bundle visual workflow completion with credential management. The
roadmap should increase stickiness by making SOM the local action-planning
layer:

1. **Action-plan helpers everywhere**: SDKs should expose compact action
   targets so agents can choose from SOM ids, roles, labels, and actions without
   bespoke tree traversal.
2. **Hint/action conformance**: `actions` and `hints` are now public contract,
   not incidental metadata. Shared fixtures should verify them across Rust,
   Python, Node, Go, and integrations.
3. **Cloud-optional workflow memory**: keep local cache/diff as the wedge, then
   add optional trace exports and cache observability before considering hosted
   browser infrastructure.

### 2026-05-10 Roadmap Adjustment

The market continues to reward structured browser state that can be reused
without another model call. Playwright MCP and Cloudflare Browser Run have made
accessibility snapshots the default low-cost interaction surface, Stagehand's
current docs center `observe()` plus local and managed action caching,
Firecrawl keeps adding hosted browser-session breadth, and Skyvern owns a
visual workflow lane. Plasmate should not pivot into hosted browser
infrastructure; the roadmap should make local SOM output more accurate and more
portable across the existing repo surface.

1. **Accessible-name parity**: controls must carry the names agents already see
   in browser accessibility snapshots, including `aria-labelledby` and external
   labels.
2. **Parser tolerance as adoption polish**: SDK/parser helpers should accept the
   real CLI/MCP payload shapes users paste into agents, including wrapped SOM
   objects and progress lines.
3. **Conformance before breadth**: every small core improvement should land
   with Rust, Python, Node, and docs coverage before another integration is
   added.

### 2026-05-11 Roadmap Adjustment

Current official docs reinforce that browser-agent products are competing on
usable page state, not raw transport. Playwright MCP centers accessibility
snapshots and stable refs, Stagehand centers `observe()` actions that can be
validated and cached, and Firecrawl/Browser Use make cloud sessions and
persistent profiles convenient for teams buying infrastructure. Plasmate should
not chase the hosted browser-cloud lane first. The roadmap should make local SOM
output more complete, deterministic, and verifiable:

1. **Accessible descriptions and names**: labels and descriptions are part of
   the action contract because agents choose controls by human-facing text.
2. **Full-tree accounting**: metadata, cache prefetch, MCP helpers, parser
   packages, and SDK helpers must all agree on shadow-root and nested content.
3. **Fixture-driven trust**: ARIA-heavy SaaS forms, web components, and repeated
   workflow pages should become shared conformance fixtures before adding more
   adapters.

### 2026-05-11 Go SDK Parity Adjustment

The repo's broad library surface is now a product promise. Python and Node
already expose action/hint lookup and compact action-plan helpers, while Go was
still missing current SOM fields and shadow-root traversal. That gap matters
because multi-service teams often adopt Go for durable workers and Python/Node
for agent orchestration; if the same SOM cannot be queried consistently across
those services, Plasmate becomes less sticky.

1. **Cross-language action plans**: Go should expose the same compact action
   targets as the parser packages so agents can plan from ids, roles, labels,
   actions, hrefs, names, and input types in any supported runtime.
2. **Shadow roots are not optional**: web-component controls must be reachable
   by id, role, text, interactivity, and flattened traversal in Go as well as
   Python and Node.
3. **Schema fields need SDK homes**: `attrs.description`, `attrs.name`,
   `attrs.autocomplete`, ARIA state, details attrs, iframe attrs, and `shadow`
   should be treated as public contract across all SDKs.

### 2026-05-11 Browser Run and Naming Adjustment

Cloudflare's Browser Run launch strengthens the trend toward browser platforms
that pair hosted sessions with Live View, recordings, human-in-loop, MCP/CDP,
and structured extraction. Plasmate should not chase that cloud lane as its
first-order wedge. The stickier roadmap move is to make SOM the most trustworthy
portable local action snapshot:

1. **Browser-like names for every target**: wrapped labels, region
   `aria-labelledby`, and input-button values should compile into the same
   human-facing names agents see in accessibility snapshots.
2. **Trace and cache over hosted scale**: repeated local workflows need
   selector-aware cache views and trace exports before a managed browser cloud
   would add durable retention.
3. **Conformance for SaaS forms**: shared fixtures should cover labels,
   descriptions, regions, fieldsets, and button values because form automation
   is where repeat users feel reliability or churn.

### 2026-05-12 Form Semantics Adjustment

Current competitor docs keep pushing the same retention lesson: agents stick
with browser tools that expose reusable action state, not just pixels or raw
HTML. Playwright MCP's accessibility snapshots train agents to rely on named
controls, Stagehand's `observe()` and caching make repeated form flows cheaper,
and Cloudflare Browser Run plus Browser Use Cloud make hosted scale easy to
buy. Plasmate's local-first answer should be stronger SaaS form semantics:

1. **Field groups are action context**: native `<fieldset>`/`<legend>` and
   ARIA `group`/`radiogroup` should survive in SOM so agents understand which
   radio buttons and controls belong together.
2. **Contract changes must cross adapters**: new roles and attrs should land in
   schema, spec, parser packages, SDKs, CDP mappings, and tests together.
3. **Conformance becomes sales collateral**: shared fixtures for grouped forms,
   descriptions, regions, and button values should prove Plasmate handles the
   repetitive SaaS workflows teams actually automate.

### 2026-05-12 Action Plan and WebMCP Adjustment

The current competitive direction is a validated action menu, not just a
browser session. Playwright MCP snapshots make current refs the interaction
unit, Stagehand `observe()` turns page understanding into cacheable executable
actions, Firecrawl's MCP surface covers scraping, extraction, research, and
browser interaction, and Cloudflare Browser Run is layering CDP/MCP/WebMCP onto
hosted sessions. Plasmate should keep the local-first wedge and make SOM action
plans more complete before pursuing hosted scale:

1. **Compact targets need context**: action plans should include placeholders,
   descriptions, disabled/required state, and group names so agents do not need
   to re-walk the full SOM for routine forms.
2. **Web components are first-class surfaces**: shadow-root extraction must
   recurse through wrapper containers because real design systems rarely put
   buttons and inputs directly under the template root.
3. **Browser tolerance beats ideal markup**: ARIA roles and landmarks should be
   parsed with the casing tolerance agents encounter on production SaaS pages.

### 2026-05-13 Action-Plan Availability Adjustment

Current competitor docs make action menus the retention surface. Playwright MCP
refs are only valid against the current snapshot, Stagehand `observe()` returns
actions that teams cache and validate, and Firecrawl/Browser Use are broadening
managed browser sessions around that workflow. Plasmate's wedge remains local
SOM portability, so compact action plans should expose availability directly in
every SDK.

1. **Availability is a first-class plan field**: action targets should include
   `enabled` and `blocked_reason` so agents can gate execution without bespoke
   attrs checks.
2. **Cross-language parity reduces churn**: Python, Node, and Go planners should
   return the same shape for disabled targets because teams mix these runtimes in
   real agent systems.
3. **Framework adapters are next**: Browser Use, LangChain, and Vercel AI
   integrations should forward availability state instead of making downstream
   agents rediscover it.

### 2026-05-13 Framework Adapter Availability Adjustment

The current market keeps pushing action planning toward the framework edge:
Playwright MCP snapshots expose current refs, Stagehand action caches reward
stable target descriptions, Firecrawl Interact and Browser Use Cloud make
hosted browsers easy, and Cloudflare Browser Run is adding MCP/CDP/WebMCP
distribution around managed sessions. Plasmate's retention path remains
local-first portability, so adapters should make disabled and required action
state visible before an agent spends a tool call on a dead control.

1. **Adapters are product surface**: Browser Use and LangChain context strings
   should render the same availability, description, group, and required fields
   exposed by parser action plans.
2. **Prompt helpers reduce misuse**: Vercel AI users should get a small
   exported guidance string that tells models to honor SOM `enabled` and
   `blocked_reason` fields.
3. **Next conformance step**: shared adapter fixtures should verify that
   framework output does not regress from the parser/SDK action-plan contract.

### 2026-05-13 Cross-Adapter Fixture Adjustment

Current competitor pressure makes adapter consistency a retention issue.
Playwright MCP snapshots, Stagehand action caching, and hosted browser traces
all teach users to expect the current action surface to be trustworthy.
Plasmate's local-first answer should be a shared adapter fixture suite that
keeps every framework aligned with the same compact SOM contract.

1. **Fixtures beat prose**: Browser Use, LangChain, Vercel AI, parser packages,
   and SDKs should test availability, required, group, type, and description
   fields against the same SOM fixture.
2. **Enabled is the default action state**: adapters should mark interactive
   targets as enabled unless SOM explicitly blocks them.
3. **Helpers should filter action menus**: Vercel AI apps need a small runtime
   helper for cached action plans, not only prompt guidance.

### 2026-05-13 Vercel AI Action Menu Adjustment

Current competitor docs keep moving reusable page state into application-level
workflows. Playwright MCP keeps action refs bound to fresh snapshots, Stagehand
`observe()` plans actions that can be cached and validated, Browserbase
foregrounds cached selectors plus observability, and Firecrawl/Browser Use keep
making managed sessions convenient. Plasmate should keep the local-first wedge
and make Vercel AI apps treat SOM action plans as a first-class menu before the
model spends tokens.

1. **Blocked means unavailable**: helper APIs should treat any
   `blocked_reason` as an execution gate, not just disabled controls.
2. **Prepare menus before prompting**: apps should normalize, filter, and cap
   action targets before handing them to `generateText` or `streamText`.
3. **Prompt formatting is product surface**: compact action-plan text should
   preserve ids, roles, labels, actions, availability, required state, groups,
   and descriptions so cached workflows do not need custom glue.

### 2026-05-13 Vercel AI SOM Extraction Adjustment

Current competitor docs keep validating action menus as the layer that creates
retention. Playwright MCP snapshots return fresh refs after actions, Stagehand
v3 `observe()` creates cacheable structured actions, Firecrawl Interact and
Browser Use Cloud package managed sessions, and Cloudflare Browser Run/WebMCP
is testing typed browser-native tools. Plasmate should not pivot into hosted
browser infrastructure, but framework integrations must make raw SOM responses
directly useful in app code.

1. **Raw SOM should become an action menu**: Vercel AI apps should be able to
   derive compact targets from a SOM response without importing parser packages
   or hand-walking nested regions.
2. **Shadow roots count at the framework edge**: extraction helpers should
   traverse `children` and `shadow.elements` because modern SaaS controls often
   live inside web components.
3. **Runtime fixture coverage is a release gate**: Vercel AI needs an
   executable fixture test for extraction, filtering, and prompt formatting so
   app-level helpers do not drift from Browser Use, LangChain, and parser
   contracts.

### 2026-05-13 Deterministic Action Cache-Key Adjustment

Reusable action memory is now part of the category expectation. Playwright MCP
keeps refs bound to fresh snapshots, while Stagehand/Browserbase action caching
and hosted trace tooling make repeated workflows cheaper after the first
observation. Plasmate should preserve its local-first execution model and add
deterministic action keys to compact SOM targets so apps can cache, dedupe, and
compare repeated actions without adopting hosted selector memory.

1. **Cache keys complement ids**: SOM ids remain the execution target, while
   `cache_key` gives agent apps a stable value for local action-plan storage,
   prompt dedupe, and trace correlation.
2. **Parser parity first**: Python and Node parser packages should emit the
   same cache-key contract as framework helpers before new hosted workflow
   features are considered.
3. **Adapters should inherit the contract**: Browser Use, LangChain, Vercel AI,
   and Go should converge on the same compact action target shape so cached
   workflows do not depend on one runtime.

### 2026-05-13 Action Cache-Key Parity Adjustment

Current browser-agent competitors are making action memory part of daily app
code. Playwright MCP exposes fresh refs, Stagehand/Browserbase cache resolved
actions, Firecrawl Interact and Browser Use Cloud make hosted browser sessions
easy to reuse, and WebMCP experiments point toward typed browser-native tools.
Plasmate should keep the local-first wedge by making cacheable action targets
portable across all high-use SDK and framework surfaces.

1. **Go is part of the action contract**: durable worker services should get
   the same `cache_key` field and helper as Python/Node orchestration code.
2. **Prompt context should show cache identity**: Browser Use and LangChain
   text outputs should render cache keys beside availability so repeated
   workflows can dedupe targets without raw SOM recovery.
3. **Shared fixtures are the next guardrail**: cache-key parity should move
   from focused adapter tests into a cross-adapter fixture runner.

### 2026-05-13 Shared Expectation Manifest Adjustment

The market now rewards tools that make reusable action surfaces boringly
consistent. Playwright MCP refs, Stagehand cached actions, and Browserbase or
Cloudflare traces all set user expectations that the current action contract
can be trusted. Plasmate's broad repo surface should turn that into an
advantage by keeping adapter tests wired to a single expected action manifest.

1. **One fixture, one contract**: Browser Use, LangChain, and Vercel AI should
   consume the same expected ids, labels, availability, blocked reasons, cache
   keys, required flags, groups, and descriptions.
2. **Drift should fail centrally**: when action-plan semantics change, the SOM
   fixture and expected manifest should change together instead of silently
   updating hard-coded assertions in each adapter.
3. **Next release gate**: extend the manifest into parser packages and SDKs,
   then wrap all checks in one release command.

### 2026-05-13 SDK Manifest Conformance Adjustment

Competitors are making reusable action state inspectable and cacheable at the
application edge. Playwright MCP refs stay tied to the current snapshot,
Stagehand/Browserbase depend on validated action caches, and Cloudflare Browser
Run recordings make run drift visible after the fact. Plasmate should turn its
local action surface into a cross-language contract before adding more
workflow-memory features.

1. **SDKs should plan actions too**: Python and Node client SDKs need compact
   action-plan helpers, not only parser packages, because many apps consume SOM
   directly from MCP calls.
2. **The manifest must cover runtimes**: parser packages, Go SDK, Python SDK,
   and Node SDK should read the same expected action target manifest as
   framework adapters.
3. **Release automation is now the bottleneck**: after manifest parity lands,
   the next sticky step is one command that runs adapter, parser, and SDK
   fixture checks together.

### 2026-05-13 Action Manifest Release-Gate Adjustment

Official docs keep putting reusable page state on the critical path.
Playwright MCP snapshots provide snapshot-scoped refs, Stagehand v3 `observe()`
returns structured actions that can be cached and validated, and Firecrawl
Interact keeps browser state alive after scraping. Plasmate should answer with
a local conformance gate that proves broad SDK and adapter support behaves like
one product surface before a release goes out.

1. **One command should prove the contract**: Browser Use, LangChain, Vercel
   AI, parser packages, and SDKs need a shared release command for the action
   availability manifest.
2. **Package tests must include fixture parity**: Node SDK action-plan tests
   should run from `npm test`, not only ad hoc TypeScript build commands.
3. **CI is the next adoption guardrail**: once dependencies are installed in
   Actions, the release command should become a required conformance job.

### 2026-05-13 CI Action-Manifest Adjustment

The latest competitor read keeps pointing to one durable retention hook:
agents stay with browser tools when action state is safe to reuse. Playwright
MCP refs, Stagehand local/server action caches, Firecrawl Interact sessions,
Browser Use Cloud profiles, and Cloudflare WebMCP all make the action surface
feel like product infrastructure. Plasmate's local-first answer should be to
make cross-runtime conformance cheap enough to run continuously.

1. **CI should catch contract drift early**: the shared action manifest now
   needs a required pull-request path, not only a maintainer release command.
2. **Fast and full gates serve different jobs**: quick mode should prove the
   single manifest contract on every change, while full mode remains the local
   pre-release check for broader action-plan behavior.
3. **Next leverage is caching**: once the quick gate is stable, tune dependency
   caches and promote more shared fixtures without making CI adoption painful.

## Architecture

```
                    +-----------------------+
                    |     CDP Gateway       |  <-- Puppeteer/Playwright compatible
                    |  (Chrome DevTools     |
                    |   Protocol subset)    |
                    +----------+------------+
                               |
                    +----------+------------+
                    |     AWP Server        |  <-- Native protocol (SOM-aware)
                    |  (WebSocket, v0.1)    |
                    +----------+------------+
                               |
              +----------------+----------------+
              |                |                |
    +---------v------+ +------v--------+ +-----v--------+
    |  Page Runtime  | | SOM Compiler  | | SOM Cache    |
    |  (V8 via       | | (v0.1, proven)| | (new)        |
    |   rusty_v8)    | |               | |              |
    +--------+-------+ +------+--------+ +-----+--------+
             |                |                |
    +--------v-------+ +-----v--------+       |
    | DOM Mutations  | | Heuristics   |       |
    | Observer       | | Engine       |       |
    +--------+-------+ +--------------+       |
             |                                |
    +--------v---------------------------------v-----+
    |              Session Manager                    |
    |  (per-tab isolation, parallel execution,        |
    |   shared cache, resource budgets)               |
    +-------------------------------------------------+
```

## Module Plan

### 1. JavaScript Execution (`src/js/`)
- **Crate**: `rusty_v8` (V8 bindings for Rust)
- **Scope**: Create V8 isolate per session, execute `<script>` tags
- **DOM bridge**: Minimal - expose `document.querySelector`, `document.getElementById`,
  `element.textContent`, `element.getAttribute`, `element.click()`,
  `window.location`, `setTimeout/setInterval`
- **Not needed for v0.2**: Full Web API (Canvas, WebRTC, WebGL, Workers)
- **Key insight**: We only need enough JS to make pages render their content.
  90% of agent-relevant JS is "fetch data, insert into DOM." We skip layout/paint.

### 2. CDP Compatibility Layer (`src/cdp/`)
- **Goal**: Puppeteer `puppeteer.connect()` works out of the box
- **Domains to implement**:
  - `Browser` (getVersion, close)
  - `Target` (createTarget, attachToTarget, getTargets)
  - `Page` (navigate, enable, getFrameTree, lifecycleEvent)
  - `Runtime` (evaluate, callFunctionOn, getProperties)
  - `DOM` (getDocument, querySelector, getOuterHTML, setAttributeValue)
  - `Network` (enable, requestWillBeSent, responseReceived - for interception)
  - `Input` (dispatchMouseEvent, dispatchKeyEvent)
  - `LP` (getMarkdown, getSemanticTree, getInteractiveElements) - Lightpanda compat!
- **Bonus**: Also expose `Plasmate` CDP domain with native SOM access

### 3. SOM Cache (`src/cache/`)
- **The paradigm shift**: Why re-parse a page you already understand?
- **Architecture**:
  ```
  URL + content_hash -> cached SOM snapshot
  ```
- **Three tiers**:
  1. **Hot cache** (in-memory): Last N pages, instant retrieval, ~0ms
  2. **Warm cache** (RocksDB on disk): Thousands of pages, <1ms retrieval
  3. **Cold cache** (shared/networked): Cross-session, cross-agent SOM sharing
- **Differential SOM**: When revisiting a URL:
  1. Fetch HTML, compute content hash
  2. If hash matches cache: return cached SOM instantly (zero parse time)
  3. If hash differs: compile new SOM, diff against cached version,
     return only changed elements + full SOM
- **Cache-aware navigation**: Agent says "go to HN" - if cached SOM is <60s old,
  return it WITHOUT fetching. Agent gets instant page understanding.
- **Prewarming**: Background thread fetches + caches URLs the agent is likely
  to visit (based on links in current SOM)
- **Selector-aware reuse**: Cache filtered SOM views (`main`, `form`, `#id`) so
  repeated agent prompts can skip both full-page serialization and downstream
  LLM token spend.

### 4. Parallel Session Manager (`src/sessions/`)
- **Rust advantage**: tokio green threads + zero-cost async
- **Per-session isolation**: Each session gets its own V8 isolate + cookie jar
- **Shared resources**: SOM cache, DNS cache, connection pool
- **Budget enforcement**: Max memory per session, max JS execution time
- **Benchmarks target**:
  - 500+ concurrent sessions per 8GB RAM (vs Lightpanda's 140, Chrome's 15)
  - <50ms cold start per session (vs Lightpanda's 100ms, Chrome's 3-5s)
  - <10ms warm start (cached SOM, no fetch needed)

### 5. Network Layer Upgrades (`src/network/`)
- **Connection pooling**: Reuse TCP/TLS connections across sessions
- **HTTP/2 multiplexing**: Multiple requests per connection
- **DNS caching**: Shared across sessions
- **Request interception**: Block ads, tracking, unnecessary resources
- **Resource budgets**: Skip images, fonts, media by default (agent doesn't need them)

## Performance Targets (vs Lightpanda)

| Metric | Chrome | Lightpanda | Plasmate Target |
|--------|--------|------------|-----------------|
| 100-page benchmark | 25.2s | 2.3s | <1.0s |
| Memory per instance | 207MB | 24MB | <8MB |
| Concurrent (8GB) | 15 | 140 | 500+ |
| Cold start | 3-5s | <100ms | <50ms |
| Token output | Raw DOM | Markdown/Tree (bolt-on) | SOM (native) |
| Warm page load | N/A | N/A | <10ms (cached SOM) |

## Why SOM Cache Changes Everything

Lightpanda and Chrome re-render every page from scratch every time.
Plasmate with SOM Cache creates a fundamentally different paradigm:

1. **First visit**: Fetch -> Parse -> JS -> Compile SOM -> Cache (normal speed)
2. **Revisit (same content)**: Cache hit -> Return SOM (~0ms)
3. **Revisit (changed content)**: Fetch -> Diff -> Update cache -> Return delta
4. **Predicted navigation**: Cache prewarms links from current page

For an AI agent that navigates 50 pages in a workflow, pages 2-50 are often
revisits or predictable next-pages. SOM Cache makes those effectively free.

## Build Order

1. **V8 integration** (unblocks JS execution, biggest gap vs Lightpanda)
2. **CDP compatibility** (unblocks Puppeteer/Playwright, enables benchmarking vs LP)
3. **SOM Cache** (the differentiator, makes us categorically faster)
4. **Parallel session manager** (proves the concurrency story)
5. **Network upgrades** (polish, production readiness)

## Current Minor Improvements Logged

- Cache prefetch extraction walks nested children and shadow-root elements,
  dedupes discovered HTTP(S) URLs, and filters out non-navigation schemes.
- Cache URL normalization now uses structured URL parsing so host casing is
  normalized without breaking case-sensitive paths.
- MCP `extract_text` and `extract_links` include shadow-root content, improving
  parity for web components and declarative shadow DOM.
- SOM link deduplication now preserves case-sensitive paths instead of
  lowercasing complete hrefs.
- Input type and ARIA role parsing tolerates real-world casing, preserving
  actionability for uppercase `type`/`role` values.
- Custom controls retain `contenteditable`, `tabindex`, `name`, and
  `autocomplete` attrs in SOM output.
- MCP `extract_text` truncation is UTF-8 safe for multilingual pages.
- SOM Schema now includes `shadow`, `iframe`, `details`, ARIA state, and
  actionability attrs emitted by the Rust compiler.
- Python and Node parser packages traverse shadow-root elements in query
  helpers and validate the newer SOM surface.
- Python and Node SDK helpers now find shadow-root elements by id, role, text,
  and actionability.
- Python and Node parser packages now expose `find_by_action`/`findByAction`,
  `find_by_hint`/`findByHint`, and compact action-plan helpers for agent
  planning over SOM ids and action metadata.
- Node parser compression-ratio handling now matches Python by returning
  infinity when `som_bytes` is zero.
- Rust SOM compilation resolves `aria-labelledby` and external `<label for>`
  accessible names for controls.
- Rust SOM compilation resolves accessible descriptions from
  `aria-describedby` and `aria-description`.
- Rust SOM compilation gives `aria-labelledby` precedence over `aria-label`.
- SOM metadata counts shadow-root elements and controls in `element_count` and
  `interactive_count`.
- SOM schema and Python/Node types accept `attrs.description`.
- Python `from_plasmate()` handles progress/log lines around SOM JSON.
- Node `fromPlasmate()` accepts wrapped `{ som: ... }` payloads, including in
  mixed CLI output.
- Go SDK types parse `shadow`, accessible descriptions, ARIA state, details
  attrs, and iframe attrs emitted by the Rust compiler.
- Go SDK query helpers traverse shadow roots for id, role, text, interactivity,
  and flattened element queries.
- Go SDK exposes action/hint lookup and compact action-plan helpers.
- Rust SOM compilation resolves wrapped `<label>` controls without leaking
  nested select option text into labels.
- Landmark and form regions resolve `aria-labelledby` labels.
- Input buttons expose value-derived labels and normalized `attrs.input_type`
  for `submit`, `button`, and `reset`.
- Native `<fieldset>` controls and ARIA `group`/`radiogroup` widgets compile as
  labelled SOM `group` elements.
- Fieldset groups expose `attrs.legend` and preserve disabled group state.
- SOM schema/spec, Python/Node SDK types, Python/Node parser types, Go SDK
  attrs, and CDP mappings accept the `group` role and `attrs.legend`.
- Shared conformance fixture added for fieldset/legend and ARIA radiogroup
  semantics.
- ARIA landmark role parsing is case-insensitive for uppercase production
  markup.
- Declarative shadow DOM extraction recurses through non-semantic wrappers so
  nested web-component controls survive.
- Python/Node parser and Go SDK compact action plans now include placeholder,
  description, required, disabled, and group metadata.
- Disabled native `<textarea>` and `<select>` controls preserve
  `attrs.disabled`, and ARIA `aria-required`/`aria-disabled` widgets promote
  top-level action-state attrs while retaining nested ARIA state.
- Disabled native `<fieldset>` state now propagates to descendant controls, so
  locked radio, textarea, select, and button targets expose `attrs.disabled`
  directly.
- Shared conformance fixture `015-action-state` now covers disabled fieldset
  inheritance and ARIA required/disabled state promotion.
- Python parser, Node parser, and Go SDK compact action plans now expose
  `enabled` plus disabled `blocked_reason` fields so agents can skip known
  unavailable targets before acting.
- Browser Use integration exposes sync/async action-plan helpers and renders
  availability, required, type, group, and description context in page prompts.
- LangChain SOM text output marks disabled, enabled, required, group, and
  description state on interactive targets.
- Vercel AI SDK integration exports `plasmateActionGuidance` so agents are
  explicitly told to honor SOM availability fields.
- Shared adapter action-availability fixture now keeps Browser Use and
  LangChain context tests aligned on enabled, disabled, required, group, type,
  and description cues.
- LangChain now treats omitted `attrs.disabled` as enabled for interactive
  targets and prints disabled `blocked_reason` state.
- Vercel AI SDK integration exports `PlasmateActionTarget` and
  `isPlasmateActionTargetAvailable()` for filtering cached action menus before
  prompting.
- Vercel AI availability checks now treat any `blocked_reason` as unavailable,
  not only disabled controls.
- Vercel AI SDK integration exports `normalizePlasmateActionTarget()`,
  `preparePlasmateActionPlan()`, and `formatPlasmateActionPlan()` for
  application-level action menu preparation.
- Vercel AI now has fixture-style TypeScript compile coverage for availability,
  required, group, and description action-plan metadata.
- Vercel AI SDK integration exports `extractPlasmateActionTargets()` to derive
  compact action targets from raw SOM responses, including nested children and
  shadow-root elements.
- Vercel AI action-plan formatting now preserves blocked reasons, input type,
  and placeholder metadata for model prompts and trace logs.
- Vercel AI now has an executable fixture test that builds the package and
  validates SOM extraction, availability filtering, and formatting against the
  shared adapter fixture.
- Vercel AI compact action targets now include deterministic `cache_key` values
  and export `getPlasmateActionTargetCacheKey()` for cached menus and trace
  correlation.
- Python and Node parser compact action plans now include deterministic
  `cache_key` fields plus helper functions for app-level cached workflows.
- Go SDK compact action plans now include deterministic `CacheKey` values and
  export `GetActionPlanCacheKey()` for worker-side action memory.
- Browser Use and LangChain context renderers now include action `cache_key`
  flags, keeping prompt text aligned with parser action plans.
- Shared adapter action-availability expectations are now centralized in
  `integrations/fixtures/action-availability.expected.json`.
- Browser Use, LangChain, and Vercel AI fixture tests now consume the shared
  expectation manifest for availability, blocked reasons, required state,
  groups, descriptions, and action cache keys.
- Integration fixture documentation now explains how to update SOM fixtures and
  expected action contracts together.
- Python SDK query helpers now expose compact action-plan helpers plus
  deterministic cache-key generation for direct client consumers.
- Node SDK query helpers now expose compact action-plan helpers plus
  deterministic cache-key generation for TypeScript app code.
- Python parser, Node parser, Go SDK, Python SDK, and Node SDK tests now consume
  the shared action-availability manifest.
- Added `scripts/action-manifest-conformance.sh` to run the shared
  action-availability manifest checks across Browser Use, LangChain, Vercel AI,
  parser packages, and SDKs from one release command.
- `scripts/action-manifest-conformance.sh` now supports `--quick` for focused
  shared-manifest checks and `--full` for the complete local release gate.
- GitHub Actions now runs a dedicated action-manifest conformance job that
  installs Python, Node, and Go dependencies before executing the quick gate.
- Node SDK `npm test` now builds the package and runs the action-plan fixture
  tests against the shared manifest.
- Root and fixture documentation now advertise the shared action-manifest
  release gate, including quick/full guidance, for maintainers changing
  action-plan semantics.
- Browser Use and LangChain package `__version__` exports now match package
  metadata.
- Selector handling now trims whitespace and supports documented region ids
  (`#region-id`) while preserving HTML id selection for agent actions.
- SOM compilation recognizes common ARIA widgets (`textbox`, `searchbox`,
  `combobox`, `listbox`, `switch`, `menuitem`, `tab`) as actionable elements,
  improving parity with accessibility-tree competitors.
- Hidden inline styles now tolerate casing and whitespace variants such as
  `DISPLAY : none`, reducing extraction noise from real-world CMS output.

## Dependencies to Add

```toml
# V8 JavaScript engine
rusty_v8 = "0.106"

# On-disk cache
rocksdb = { version = "0.22", default-features = false }

# Faster hashing for cache keys
xxhash-rust = { version = "0.8", features = ["xxh3"] }

# Memory tracking
jemalloc-ctl = "0.5"
jemallocator = "0.5"

# HTTP/2
hyper = { version = "1", features = ["http2", "server"] }
```
