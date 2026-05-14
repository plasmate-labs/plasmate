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
   `attrs.accept`, `attrs.capture`, `attrs.multiple`, `attrs.autocomplete`,
   ARIA state, details attrs, iframe attrs, and `shadow` should be treated as
   public contract across all SDKs.

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

### 2026-05-13 Semantic Fidelity Polish Adjustment

Competitor docs keep turning browser state into reusable action contracts:
Playwright MCP snapshots expose accessibility roles and refs, Stagehand
`observe()` plus action caching rewards stable target descriptions, Firecrawl
Browser Sandbox and Browser Use Cloud package managed execution, Crawl4AI is
moving open-source crawling toward cloud extraction, and Cloudflare WebMCP is
testing typed website-provided tools. Plasmate should keep the local-first
wedge, but small semantics now determine whether an agent trusts SOM without
raw DOM recovery.

1. **Search is a landmark, not generic content**: ARIA `role="search"` should
   compile into a labelled region so agents can scope query tasks reliably.
2. **Menus carry actionable state**: ARIA `menuitemcheckbox` and
   `menuitemradio` should map to checkbox/radio action targets before
   framework adapters consume the page.
3. **Noise stripping must tolerate production CSS**: visibility parsing should
   ignore casing and arbitrary whitespace in stylesheet declarations, matching
   the inline-style hardening already shipped.

### 2026-05-13 Action-Semantics Fixture Adjustment

Current browser-agent comparisons keep confirming that reusable action state is
only sticky when downstream app code can trust it without engine-specific
knowledge. Browser Use and Stagehand make action menus developer-facing,
Playwright MCP makes structured refs the interaction unit, and hosted browser
tools sell traces and session reuse around the same contract. Plasmate should
promote semantic fixes into shared fixtures as soon as they land.

1. **Menu widgets belong in the manifest**: ARIA menu checkbox/radio targets
   should appear in the shared action-availability fixture before adapters
   treat them as reusable actions.
2. **Search and visibility need one fixture**: search landmarks and
   stylesheet-hidden whitespace are common SaaS cases that should be tested
   together with action targets.
3. **Docs fixtures need executable guards**: conformance fixtures should have
   focused Rust coverage first, then graduate into parser, SDK, and adapter
   release gates.

### 2026-05-13 ARIA Fallback and Visibility Adjustment

Official docs and current competitor positioning continue to reward compact,
browser-like action surfaces over raw DOM access. Playwright MCP snapshots use
fresh accessibility refs, Stagehand `observe()` returns actions that can be
cached locally or on Browserbase, Firecrawl Interact resumes scrape sessions
for prompt/code actions with profiles, Browser Use Cloud exposes CDP browser
sessions with profile state, and Crawl4AI is broadening LLM-friendly crawling
toward cloud extraction. Plasmate should keep the local-first wedge and close
the small production-markup gaps that force agents back to raw DOM recovery.

1. **ARIA roles need fallback-token tolerance**: landmark and widget roles
   should honor the first known role in a space-separated `role` list.
2. **Hidden state should match browser intent**: uppercase ARIA booleans and
   inline opacity/zero-size hiding should be stripped like equivalent
   stylesheet rules.
3. **Conformance fixtures should absorb semantic polish**: every production
   tolerance fix should be attached to `016-action-semantics` or another shared
   fixture before adapter release gates consume it.

### 2026-05-13 Control-State Action Menu Adjustment

The latest docs keep reinforcing that reusable actions are only sticky when
state is current. Playwright MCP refs are scoped to fresh accessibility
snapshots, Stagehand's `observe()` cache has to validate before acting, and
Firecrawl/Browser Use sell browser/session continuity around forms that change
between runs. Plasmate should keep the local-first action memory wedge, but
compact targets need enough live state to keep cached plans honest.

1. **Current values are planning context**: text inputs and selects should
   surface non-empty `value` fields in action plans and prompt renderers so
   agents know whether a form is already filled.
2. **Checked state must cross custom controls**: native `checked` attrs and
   ARIA `checked` state should normalize into one compact action-plan field for
   checkbox, radio, menuitemcheckbox, and menuitemradio targets.
3. **Cache keys stay target-focused**: live value/checked state should be
   visible to agents without changing deterministic target `cache_key` values,
   preserving local action memory while still exposing state drift.

### 2026-05-13 ARIA State-Cues Adjustment

Current competitor movement keeps raising the value of state-aware action
menus. Playwright MCP snapshots are valid only against the current page,
Stagehand v3 action caches need local or Browserbase validation before reuse,
Browser Run/WebMCP points toward typed page actions, and hosted browser
platforms sell traces and persistent sessions around the same drift problem.
Plasmate should keep the local-first wedge by making compact SOM targets carry
the ARIA state agents need before they choose a cached action.

1. **Expanded state prevents stale menu actions**: action plans should surface
   `aria-expanded` so agents know whether disclosure menus and comboboxes
   already expose the target content.
2. **Pressed state matters for toggle buttons**: `aria-pressed` should travel
   with compact targets just like `checked`, because repeated workflows often
   need to avoid toggling an already-correct state.
3. **Selected state is reusable context**: custom tabs/options using
   `aria-selected` should expose that state across parser packages, SDKs, and
   framework prompt renderers without changing target cache keys.

### 2026-05-13 Readonly and Selected-Value Adjustment

Current browser-agent products keep moving from one-off page observation to
validated action replay. Playwright MCP refs are snapshot-scoped, Stagehand
`observe()` caches need action validation, and Firecrawl/Browser Use sell
persistent sessions around forms whose state can drift between runs. Plasmate's
local-first roadmap should make compact action menus safer to reuse by carrying
the small blockers and current values agents otherwise recover from raw DOM.

1. **Read-only is an execution gate**: text inputs and textareas with
   `readonly` should be visible in SOM and action plans as unavailable targets
   with `blocked_reason="readonly"`.
2. **Current values include textarea/select state**: cached plans need current
   textarea text and selected option values, not only `value` attributes on
   inputs.
3. **Production ARIA is not always lowercase**: ARIA boolean preservation
   should trim and parse casing variants so compact state remains typed and
   comparable across SDKs.

### 2026-05-13 Validation-Constraint Action Menu Adjustment

The newest competitor docs keep making cached actions depend on validation
state, not just target identity. Playwright MCP refs are current-snapshot
handles, Stagehand can cache `observe()`/action results locally or on
Browserbase, and Browser Use/Firecrawl package session continuity around
repetitive forms. Plasmate should keep the local-first wedge by making compact
action plans carry the field constraints agents need before reusing a cached
type action.

1. **Autocomplete is planning context**: action targets should expose
   `autocomplete` tokens so agents can pick the right credential/profile data
   without re-walking SOM attrs.
2. **Validation constraints reduce bad retries**: `minlength`, `maxlength`,
   and `pattern` should travel through Rust, schema, SDKs, parser packages, and
   adapters so agents know what a field accepts before typing.
3. **Invalid state blocks blind replay**: `aria-invalid` should surface as
   compact `invalid` state without changing target `cache_key` values, letting
   cached plans stay stable while validation drift remains visible.

### 2026-05-13 Input-Affordance Action Menu Adjustment

Current browser-agent products keep making repeated actions depend on the
target's current browser affordances. Playwright MCP refs still belong to a
fresh accessibility snapshot, while Stagehand/Browserbase action caches only
pay off when field modality and autocomplete state are visible before replay.
Plasmate should keep the local-first wedge by making compact action targets
carry input hints that affect credential selection, keyboard flow, and
autocomplete suggestion state.

1. **Input modality is planning context**: `inputmode` should travel through
   Rust, schema, SDKs, parser packages, and adapters so agents know whether a
   field expects email, decimal, numeric, search, or URL-style values.
2. **Keyboard intent matters for form flow**: `enterkeyhint` should be exposed
   in compact menus so repeated workflows know whether Enter advances,
   searches, submits, or sends.
3. **Autocomplete widgets need live state**: `aria-autocomplete` and
   `aria-activedescendant` should surface as compact action-plan cues without
   changing target `cache_key` values, preserving local action memory while
   making suggestion drift visible.

### 2026-05-14 Text-Entry Affordance Adjustment

Current browser-agent docs and recent developer commentary keep validating the
same retention surface: compact action menus must expose the current field
state before a cached typing plan is replayed. Playwright MCP keeps interaction
tied to fresh accessibility snapshots, while Stagehand/Browserbase action
caching only remains trustworthy when a field's keyboard and prompt affordances
still match the cached target. Plasmate should keep the local-first wedge by
surfacing small text-entry cues across the shared manifest.

1. **Typing behavior is replay context**: `spellcheck` and
   `autocapitalize` should travel through Rust, schema, SDKs, parser packages,
   and adapters so agents understand language and virtual-keyboard behavior.
2. **Direction capture matters for global forms**: `dirname` should be exposed
   as compact target context for bidirectional text-entry workflows.
3. **Custom textboxes need prompt text**: `aria-placeholder` should surface as
   `aria_placeholder` without changing target `cache_key` values, preserving
   local action memory while making custom-field prompt drift visible.

### 2026-05-14 Upload Affordance Adjustment

Current browser-agent products keep making repeat workflows depend on compact,
validated action targets. File inputs are a sticky SaaS case because agents
cannot safely reuse an upload plan unless they know the field identity, allowed
file/media types, capture hint, and multi-file state before replay. Plasmate
should keep the local-first wedge by making upload controls as explicit as
text fields across the shared manifest.

1. **Field identity affects cacheability**: `name` should stay visible in
   compact targets so action caches distinguish similarly labelled upload
   controls inside enterprise forms.
2. **Upload constraints prevent wrong-file prompts**: `accept` and `capture`
   should travel through Rust, schema, SDKs, parser packages, and adapters so
   agents can ask for the right artifact before invoking an upload flow.
3. **Multi-select state changes the plan**: native `multiple` should surface in
   action plans for file and select controls without changing deterministic
   target ids.

### 2026-05-14 Form Submission Context Adjustment

Validated action menus are only sticky when they include the submission
contract around the target. Playwright MCP keeps refs scoped to a current
snapshot, while Stagehand/Browserbase action caches need the page state to
match before replay. Plasmate should preserve the form-level metadata agents
need to tell whether a cached type, upload, or submit step still belongs to the
same workflow.

1. **Destination changes risk**: `action`, `method`, and `target` should
   travel into compact targets as form context so agents can distinguish
   settings, checkout, and background-submit flows.
2. **Encoding changes artifacts**: `enctype` and `accept-charset` should be
   visible before file uploads or internationalized form submissions are
   replayed.
3. **Validation and autofill change readiness**: `novalidate` and form-level
   `autocomplete` should surface across SDKs and adapters so local action
   memory can be checked before a browser action is spent.

### 2026-05-14 Submitter Override Adjustment

Repeated SaaS forms often contain several submit buttons with different
endpoints or validation behavior. Browser-agent competitors keep teaching users
to validate cached actions against the current structured state before replay,
so Plasmate should preserve the button-level submission contract as compact
target context.

1. **Submit buttons need identity beyond label**: `button_type` should expose
   whether a button submits, resets, or acts as a plain command.
2. **Button overrides can change destination**: `formaction`, `formmethod`,
   `formenctype`, and `formtarget` should travel with the action target.
3. **Validation mode is replay context**: `formnovalidate` should be visible so
   cached submit actions do not assume browser validation will run.

### 2026-05-14 Form-Relation Action Menu Adjustment

Current browser-agent products keep turning page state into reusable action
menus. Playwright MCP refs remain valid only for the current snapshot,
Stagehand local/server caches need page-state validation before replay, and
Firecrawl plus Browser Use keep monetizing persistent sessions for repeated
form workflows. Plasmate should keep the local-first wedge by making compact
targets carry the relationships agents need before typing or submitting.

1. **Form ownership prevents wrong-submit actions**: `form` should travel
   through Rust, schema, SDKs, parser packages, and adapters so controls
   outside a `<form>` still show which submission scope owns them.
2. **Datalist references shape value choice**: `list` should surface in action
   plans so agents know when an input is backed by a suggestion source.
3. **Error-message relationships explain invalid state**:
   `aria-errormessage` should surface as compact `errormessage` state without
   changing target `cache_key` values.

### 2026-05-14 Live-Region Action Menu Adjustment

Current browser-agent docs keep making repeated actions depend on current page
state. Playwright MCP refs expire when the snapshot changes, Stagehand caching
validates page state before replay, and Browser Use/CDP sessions preserve
dynamic app state for long-running workflows. Plasmate should keep the
local-first wedge by surfacing live-region state in the same portable compact
target contract.

1. **Busy state gates replay**: `aria-busy` should surface as compact `busy`
   state so agents know whether results or controls are still updating.
2. **Live politeness shapes waiting**: `aria-live` should travel as `live` so
   agents can distinguish polite status updates from urgent alert feedback.
3. **Announcement scope explains drift**: `aria-atomic` and `aria-relevant`
   should surface as `atomic` and `relevant` without changing deterministic
   `cache_key` values.

### 2026-05-14 Popover and Command Relationship Adjustment

Browser-native action relationships are becoming more important for agents
that replay cached plans on modern app UIs. The Popover API gives buttons a
declarative target and action, and `commandfor`/`command` generalize that model
for popovers, dialogs, and custom commands. Plasmate should preserve those
relationships as compact target context instead of forcing agents to rediscover
them from raw DOM.

1. **Invoker targets are replay context**: `popovertarget` should surface so an
   agent knows which panel a button affects.
2. **Native action verbs reduce guesswork**: `popovertargetaction` and
   `command` should travel through action plans so cached clicks can be
   validated as show, hide, toggle, or custom commands.
3. **Command ownership complements ARIA controls**: `commandfor` should sit
   alongside `aria-controls` as a native relationship cue without changing
   deterministic `cache_key` values.

### 2026-05-14 ARIA Relationship Context Adjustment

Current browser-agent products reward action menus that explain why a target is
safe to reuse, not just that it is clickable. Playwright MCP keeps refs scoped
to the fresh accessibility snapshot, Stagehand caches observed actions only
when the page still validates, and hosted browser products make traces easy to
inspect. Plasmate should make local compact targets carry more relationship
context before users need those hosted traces.

1. **Custom ownership is action context**: `aria-owns` should surface as
   `owns` so agents understand menu, listbox, and composite-widget ownership.
2. **Guided flow order should be portable**: `aria-flowto` should surface as
   `flowto` for multi-step forms and custom onboarding flows.
3. **Detailed help can stay out of raw DOM**: `aria-details` should surface as
   `details` so agents can locate extended help or validation panels without
   changing deterministic `cache_key` values.

### 2026-05-14 Link Navigation Cue Adjustment

Current browser-agent products make replay safer by validating action targets
against current page state. Links remain common agent actions, but `href` alone
does not tell an agent whether the click opens a new browsing context, carries
relationship tokens, or triggers a download. Plasmate should preserve those
native link cues in compact targets while keeping cache keys focused on the
stable target identity.

1. **New contexts are replay context**: `target` should travel through Rust,
   schema, SDKs, parser packages, and adapters so agents know when a link is
   likely to open outside the current page.
2. **Relationship tokens reduce unsafe assumptions**: `rel` should surface for
   cues such as `noopener`, `noreferrer`, `sponsored`, or `nofollow`.
3. **Downloads are action side effects**: `download` should be exposed as
   boolean/string compact target state without changing deterministic
   `cache_key` values.

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
- ARIA `role="search"` now compiles into a labelled navigation region, keeping
  search landmarks available as scoped SOM regions.
- ARIA `menuitemcheckbox` and `menuitemradio` now compile as actionable
  checkbox/radio targets for custom menu widgets.
- Stylesheet hidden-rule parsing now ignores arbitrary declaration whitespace
  and casing, so `DISPLAY\t:\nnone` is treated like `display:none`.
- The SOM improvements test suite now asserts case-sensitive URL path
  preservation, matching the compiler's public deduplication contract.
- Shared action-availability expectations now include ARIA menu checkbox and
  radio targets, keeping parser, SDK, and framework action menus aligned on
  custom menu controls.
- Added conformance fixture `016-action-semantics` for labelled search
  landmarks, ARIA menuitem checkbox/radio targets, and stylesheet hidden-rule
  whitespace/casing.
- Rust compiler tests now load the `016-action-semantics` fixture directly so
  those semantics have executable coverage before adapter promotion.
- ARIA landmark role parsing now honors fallback-token role lists, keeping
  `role="utility search"` available as a labelled navigation/search region.
- ARIA widget role parsing now honors fallback-token role lists, keeping custom
  menu checkbox/radio targets actionable when unknown roles precede known
  roles.
- Inline and ARIA hidden stripping now covers uppercase `aria-hidden="TRUE"`
  and inline `opacity: 0`, matching common production visibility variants.
- The `016-action-semantics` fixture now guards role fallback tokens, uppercase
  ARIA-hidden state, and inline opacity hiding alongside menu/search semantics.
- Python/Node parser packages, Python/Node/Go SDKs, Browser Use, LangChain,
  and Vercel AI action-plan surfaces now expose non-empty control `value`
  fields for interactive targets.
- Compact action targets now normalize native `attrs.checked` and ARIA
  `aria.checked` into a shared `checked` field across parser, SDK, and
  framework adapter outputs.
- The shared action-availability manifest now asserts `value` and `checked`
  state while keeping existing deterministic action `cache_key` values stable.
- Python/Node parser packages, Python/Node/Go SDKs, Browser Use, LangChain,
  and Vercel AI action-plan surfaces now expose ARIA `expanded`, `pressed`,
  and `selected` cues for interactive targets.
- Browser Use, LangChain, and Vercel AI prompt renderers now include
  expanded/pressed/selected state alongside value and checked state.
- The shared action-availability manifest now asserts ARIA state cues without
  changing existing deterministic action `cache_key` values.
- Native read-only input and textarea controls now preserve `attrs.readonly`,
  and parser/SDK/framework action-plan surfaces mark those targets unavailable
  with `blocked_reason="readonly"` while preserving cache-key stability.
- Textarea content and selected `<select>` options now populate compact target
  `value` fields, extending current-control state beyond input `value`
  attributes.
- ARIA state preservation now trims and parses case-insensitive boolean values,
  so `aria-expanded=" FALSE "` and `aria-pressed="TRUE"` remain typed booleans.
- The shared action-availability manifest now asserts read-only blockers and
  selected-option values across Browser Use, LangChain, Vercel AI, parser
  packages, and SDKs.
- Rust SOM compilation and the JSON Schema now preserve form ownership,
  datalist, and error-message relationships with `form`, `list`, and
  `aria-errormessage`.
- Python/Node parser packages, Python/Node/Go SDKs, Browser Use, LangChain, and
  Vercel AI action-plan surfaces now expose `form`, `list`, and `errormessage`
  without changing deterministic action `cache_key` values.
- The shared action-availability manifest now asserts form-relation cues across
  parser, SDK, and framework adapter outputs.
- Rust SOM compilation and the JSON Schema now preserve ARIA live-region state
  with `aria-busy`, `aria-live`, `aria-atomic`, and `aria-relevant`.
- Python/Node parser packages, Python/Node/Go SDKs, Browser Use, LangChain, and
  Vercel AI action-plan surfaces now expose `busy`, `live`, `atomic`, and
  `relevant` without changing deterministic action `cache_key` values.
- The shared action-availability manifest now asserts live-region cues across
  parser, SDK, and framework adapter outputs.
- Rust SOM compilation and the JSON Schema now preserve native popover/command
  relationship state with `popovertarget`, `popovertargetaction`,
  `commandfor`, `command`, and `popover`.
- Python/Node parser packages, Python/Node/Go SDKs, Browser Use, LangChain, and
  Vercel AI action-plan surfaces now expose `popovertarget`,
  `popovertargetaction`, `commandfor`, and `command` without changing
  deterministic action `cache_key` values.
- The shared action-availability manifest now asserts popover/command cues
  across parser, SDK, and framework adapter outputs.
- Rust SOM compilation and the JSON Schema now preserve ARIA relationship
  context with `aria-owns`, `aria-flowto`, and `aria-details`.
- Python/Node parser packages, Python/Node/Go SDKs, Browser Use, LangChain,
  and Vercel AI action-plan surfaces now expose `owns`, `flowto`, and
  `details` without changing deterministic action `cache_key` values.
- The shared action-availability manifest now asserts ARIA owns/flowto/details
  cues across parser, SDK, and framework adapter outputs.
- Rust SOM compilation and the JSON Schema now preserve link navigation cues
  with `target`, `rel`, and `download`.
- Python/Node parser packages, Python/Node/Go SDKs, Browser Use, LangChain,
  and Vercel AI action-plan surfaces now expose `target`, `rel`, and
  `download` without changing deterministic action `cache_key` values.
- The shared action-availability manifest now asserts link target/rel/download
  cues across parser, SDK, and framework adapter outputs.
- Rust SOM compilation and the JSON Schema now preserve ARIA widget affordance
  state with `aria-readonly`, `aria-multiline`, and
  `aria-multiselectable`.
- Python/Node parser packages, Python/Node/Go SDKs, Browser Use, LangChain,
  and Vercel AI action-plan surfaces now expose `readonly`, `multiline`, and
  `multiselectable`; ARIA read-only targets are unavailable with
  `blocked_reason="readonly"` while deterministic action `cache_key` values
  stay stable.
- The shared action-availability manifest now asserts ARIA read-only gating,
  multiline text entry, and multiselectable widget cues across parser, SDK,
  and framework adapter outputs.
- Rust SOM compilation and the JSON Schema now preserve text-entry affordance
  cues with native `spellcheck`, `autocapitalize`, `dirname`, and ARIA
  `aria-placeholder`.
- Python/Node parser packages, Python/Node/Go SDKs, Browser Use, LangChain,
  and Vercel AI action-plan surfaces now expose `spellcheck`,
  `autocapitalize`, `dirname`, and `aria_placeholder` without changing
  deterministic action `cache_key` values.
- The shared action-availability manifest and `016-action-semantics`
  conformance fixture now assert text-entry affordance cues across Rust,
  parser, SDK, and framework adapter outputs.
- Rust SOM compilation and the JSON Schema now preserve upload action cues
  with native `accept`, `capture`, and input `multiple`, and the shared action
  manifest now asserts `name` as field identity for cacheable targets.
- Python/Node parser packages, Python/Node/Go SDKs, Browser Use, LangChain,
  and Vercel AI action-plan surfaces now expose `name`, `accept`, `capture`,
  and `multiple` for upload and multi-select workflows.
- The shared action-availability manifest now asserts upload constraints and
  native multiple-selection state across parser, SDK, and framework adapter
  outputs.
- Rust SOM compilation and the JSON Schema now preserve form submission
  context with form `target`, `enctype`, `novalidate`, `accept-charset`, and
  form-level `autocomplete` alongside existing `action` and `method`.
- Python/Node parser packages, Python/Node/Go SDKs, Browser Use, LangChain,
  and Vercel AI action-plan surfaces now expose `form_action`, `form_method`,
  `form_target`, `form_enctype`, `form_novalidate`, `form_accept_charset`,
  and `form_autocomplete` without changing deterministic cache keys.
- The shared action-availability manifest now asserts form submission context
  across parser, SDK, and framework adapter outputs.
- Rust SOM compilation and the JSON Schema now preserve submit-button override cues with
  `button_type`, `formaction`, `formmethod`, `formenctype`, `formtarget`, and
  `formnovalidate`, matching the existing schema/spec contract.
- Python/Node parser packages, Python/Node/Go SDKs, Browser Use, LangChain,
  and Vercel AI action-plan surfaces now expose those submit override cues so
  cached submit actions can validate endpoint, method, encoding, target, and
  validation mode before replay.
- The shared action-availability manifest now asserts submit-button override
  context across parser, SDK, and framework adapter outputs.
- Next conformance step: promote upload-affordance, form-submission context,
  and submit-button override cases into broader Rust/parser/SDK and adapter
  fixtures alongside text-entry, ARIA widget, range, and set-position cases.

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
