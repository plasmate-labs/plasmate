# Plasmate PRD: Agent Stickiness and Roadmap Direction

Last updated: 2026-05-13

## Product Thesis

Plasmate should be the local-first browser engine agents keep installed because
it turns web pages into compact, stable, actionable state. The product is not a
general scraper and should not compete feature-for-feature with hosted browser
clouds. Its advantage is fast SOM output, predictable element ids, MCP-native
tooling, and reusable page understanding without sending browsing data to a
cloud API.

## Target Audience

- AI agent developers building MCP tools, coding agents, and research agents.
- Framework maintainers who need cheaper page context for Browser Use,
  LangChain, LlamaIndex, CrewAI, and adjacent stacks.
- Growth, sales, and ops teams that monitor authenticated or repetitive web
  workflows and need structured change detection.
- Privacy-sensitive teams that want local web extraction instead of hosted
  scraping APIs.

## Competitive Context

- Playwright MCP has made structured accessibility snapshots a baseline for
  browser-agent tools.
- Firecrawl is broadening from scraping into MCP search, extraction, browser
  sessions, and hosted deep research.
- Browserbase/Stagehand is emphasizing action caching and observability for
  repeated automation flows.
- Crawl4AI remains strong for open-source Python crawling and extraction, but
  carries Chromium/Playwright operational weight.

2026-05-05 market read: the strongest retention hooks are reusable structured
state, cached repeated actions, and ecosystem-native distribution. Playwright
MCP returns accessibility snapshots with stable refs for interaction, Stagehand
now markets action caching as an LLM-cost and latency reduction path, and
Firecrawl's MCP surface combines scraping, search, browser sessions, and deep
research. Plasmate should not chase hosted anti-bot infrastructure as the main
wedge; it should make local SOM snapshots more complete, reusable, and easy to
verify across its many adapters.

2026-05-06 market read: competitors are converging on "agent-ready page state"
as a retention mechanism. Playwright MCP's structured snapshots set the
expectation that interactive elements carry stable refs, Stagehand's
`observe()` and action caching make repeated workflows feel deterministic after
the first run, Firecrawl's current MCP docs include interactive browser
sessions alongside scrape/search/extract, and Skyvern continues to package
screenshots plus DOM context for multi-step visual workflows. The clearest
Plasmate answer is not a pivot into hosted browser clouds; it is tighter local
SOM actionability, conformance fixtures, and deterministic cache/diff behavior
across the many SDK and integration repos.

2026-05-07 market read: competitors are widening the browser-agent state
contract. Playwright MCP documents accessibility snapshots as the default
interaction layer with stable refs, Stagehand v3 emphasizes `observe()`
planning, server/local action caching, and targeted operation across iframes
and shadow roots, while Firecrawl and Browser Use package managed browser
sessions, persistent profiles, and cloud automation for teams that do not want
local browser operations. Plasmate should keep its local-first wedge, but
SDK/parser parity is now product stickiness: if Rust emits shadow roots,
iframe/details roles, or ARIA/action attrs, every adapter must be able to parse
and query them.

2026-05-09 market read: the trend has moved from "give the model page text" to
"give the agent a reusable decision surface." Playwright MCP and Cloudflare's
fork continue to validate structured accessibility snapshots without vision
models, Stagehand is positioning `observe()` plus action caching as the path
from natural-language intent to deterministic repeated actions, Firecrawl is
bundling scrape/search/extract with agent and browser-session APIs, and Skyvern
is selling visual workflow reliability with credential and enterprise controls.
Plasmate should keep avoiding a hosted browser-cloud pivot. The stickier move is
to make local SOM output easy to query as an action plan across every SDK and
integration.

2026-05-10 market read: the same category shift is now explicit in competitor
messaging. Playwright MCP and Cloudflare's Browser Run fork both position
structured accessibility snapshots as the lightweight alternative to vision
models, Stagehand's current docs and marketing center `observe()` plus local or
managed action caching, Firecrawl continues to bundle scrape/search/extract
with browser sessions, and Skyvern targets end-to-end visual workflows. The
stickiest Plasmate path is still local-first SOM, but "correct labels
everywhere" is now part of actionability: agents cannot reliably reuse plans if
form controls lose `aria-labelledby`, `<label for>`, or parser output hidden
behind CLI progress lines.

2026-05-11 market read: official docs still point to the same sticky layer:
Playwright MCP centers accessibility snapshots with refs, Stagehand uses
`observe()` to plan, validate, and cache executable actions, and Firecrawl plus
Browser Use make managed sessions and persistent profiles easy to buy. Plasmate
should keep the local-first SOM wedge, but names, descriptions, and full-tree
metadata must match how agents choose controls. This is a refinement, not a
pivot: improve deterministic SOM contracts before hosted-browser features.

2026-05-11 Go parity read: Stagehand's action caching and Playwright MCP's
snapshot refs both teach agent developers to expect a compact, reusable action
surface in every language they use. Plasmate's repo already spans Rust, Python,
Node, Go, MCP, CDP, AWP, parser packages, and framework integrations, so
stickiness now depends on contract parity as much as core extraction quality.
Go should not lag Python and Node on shadow-root traversal, action/hint lookup,
or accessible description fields because teams adopting Plasmate across
services will judge the product by the weakest SDK surface.

2026-05-11 Browser Run read: Cloudflare has rebranded Browser Rendering as
Browser Run and is now positioning global headless browser sessions, Live View,
recordings, human-in-loop, MCP/CDP support, and structured extraction as an AI
agent browser platform. That makes the hosted infrastructure lane more crowded,
not more attractive for Plasmate's near-term wedge. Plasmate should deepen the
portable local snapshot contract: controls and regions need browser-like names,
repeat runs need cacheable state, and every adapter should consume the same SOM
shape without bespoke DOM recovery.

2026-05-12 market read: official docs continue to validate Plasmate's local
SOM-first direction. Playwright MCP's snapshot model makes structured
accessibility refs table stakes, Stagehand now packages `observe()` planning
with local/managed action caching, Cloudflare Browser Run is selling hosted
global browser sessions plus structured extraction, and Browser Use Cloud is
packaging agents, direct CDP sessions, profiles, skills, proxies, and managed
scale. The sticky counter-position is not another hosted browser fleet. It is a
portable local action snapshot that accurately represents the form semantics
agents need to reuse plans on SaaS pages.

2026-05-12 action-plan read: the frontier has moved one level up from
"structured page text" to "validated action menus." Playwright MCP snapshots
teach agents to select refs from the current accessibility tree, Stagehand
`observe()` returns executable actions that can be cached locally or on
Browserbase, Firecrawl's MCP distribution now spans scrape/search/extract plus
browser interaction, and Cloudflare Browser Run is adding MCP/CDP/WebMCP
surfaces around hosted sessions. Plasmate should keep treating hosted scale as
optional and make local SOM action plans richer: every compact target should
carry enough state, labels, descriptions, and group context for an agent to
reuse a plan without traversing raw DOM again.

2026-05-13 state-fidelity read: 2026 browser-agent commentary keeps converging
on the same split. Playwright and Playwright MCP remain the deterministic
execution layer, Stagehand-style `observe()` APIs turn ambiguous page state into
cacheable actions, Browserbase/Browser Use/Skyvern compete on managed sessions,
profiles, CAPTCHA/proxy support, and traces, while WebMCP-style structured tool
exposure is emerging as a standardization watch item. Plasmate should not pivot
into managed browser infrastructure. The sticky wedge is still a local,
portable SOM/action state contract, and the next reliability gains are in small
state fidelity fixes: disabled, required, grouped, described, and shadow-root
controls must mean the same thing to Rust, MCP, SDKs, parsers, and integrations.

2026-05-13 conformance read: current competitor positioning raises the bar from
"structured output exists" to "the action surface is safe to reuse." Playwright
MCP snapshots expose refs after each action, Stagehand 3.3 adds strict
structured outputs and clearer upload/action state, and managed browser
platforms sell traces for post-run debugging. Plasmate's local-first response
should be to make disabled and required state portable enough that agents do
not need raw DOM recovery before reusing a cached plan.

2026-05-13 availability read: official Playwright MCP docs still make fresh
structured snapshots with refs the interaction unit, Stagehand v3 documents
`observe()` as a cacheable action menu, and Firecrawl/Browser Use keep
expanding managed browser sessions. Plasmate should not pivot into hosted
session infrastructure; the higher-stickiness move is to make local action
plans safer by surfacing an explicit availability gate across SDKs.

2026-05-13 adapter read: current docs keep validating the framework edge as a
retention surface. Playwright MCP tells agents to act from structured snapshots
with fresh refs, Stagehand `observe()` turns page state into cacheable actions,
Firecrawl Interact and Browser Use Cloud package managed browser sessions,
profiles, and CDP access, and Cloudflare Browser Run is widening hosted
MCP/CDP/WebMCP distribution. Plasmate should still avoid a hosted-infra pivot;
the stickier move is to make Browser Use, LangChain, and Vercel AI adapters
surface the same local action availability cues already present in parser/SDK
helpers.

2026-05-13 fixture read: the near-term competitive gap is no longer whether an
adapter can show a page; it is whether every framework shows the same reusable
action contract. Playwright MCP snapshots and Stagehand cached actions both
make state drift costly. Plasmate should turn shared adapter fixtures into a
release gate so Browser Use, LangChain, Vercel AI, parser packages, and SDKs
all preserve availability, required, group, type, and description cues.

## Ecosystem Surface

The project already spans a large number of package and integration surfaces:
Rust CLI/daemon/MCP/CDP/AWP core, Python SDK, Node SDK, Go SDK, LangChain,
Browser Use, Vercel AI, SOM parser packages for Python and Node, plugin
examples, smoke tests, generated docs, comparison pages, and marketing assets.
This breadth is a distribution advantage only if contracts stay synchronized.
Short-term roadmap work should favor conformance fixtures, shared schema tests,
and adapter docs over one-off integration logic.

## Requirements

1. Preserve actionable structure: SOM must capture common accessibility roles,
   stable ids, labels, forms, links, state, and selectors that agents can reuse.
2. Reduce repeated-work cost: SOM cache, SOM diff, and selector-aware cache
   entries should make repeat visits cheaper than first visits.
3. Improve inspectability: expose traces, coverage scorecards, and reproducible
   fixtures so teams can trust extraction behavior.
4. Keep ecosystem adapters thin: SDKs and integrations should share conformance
   expectations instead of forking extraction logic.
5. Stay local-first by default: hosted competitors can own scale infrastructure;
   Plasmate should own local speed, privacy, and open protocol fit.

## Current Run Changes

- 2026-05-13:
  - Added a shared adapter fixture for action availability, required fields,
    groups, input type, and descriptions.
  - Browser Use and LangChain adapter tests now consume the same fixture,
    reducing drift between framework context output and parser action plans.
  - LangChain now marks normal interactive targets as `[enabled]` even when
    SOM omits `attrs.disabled`, and includes `[blocked_reason=disabled]` for
    disabled targets.
  - Vercel AI SDK integration now exports `PlasmateActionTarget` and
    `isPlasmateActionTargetAvailable()` so apps can filter cached action menus
    before prompting.
  - Browser Use and LangChain package `__version__` exports now match their
    `pyproject.toml` versions.
  - Browser Use integration page contexts now render compact action-plan
    targets with `enabled`, disabled `blocked_reason`, required, type, group,
    and description context.
  - Browser Use integration now exposes sync and async `extract_action_plan`
    helpers so agents can ask directly for reusable SOM action targets.
  - LangChain SOM text output now marks disabled, enabled, required, group, and
    description state on interactive elements before click/type planning.
  - Vercel AI SDK integration now exports `plasmateActionGuidance`, a concise
    system-prompt helper that tells agents to honor SOM availability fields.
  - Added focused adapter tests for Browser Use and LangChain availability
    rendering.
  - Python SOM parser action plans now include `enabled` and
    `blocked_reason`, so agents can skip disabled targets without re-walking
    attrs.
  - Node SOM parser action plans now expose the same availability contract in
    `ActionPlanItem`.
  - Go SDK action plans now expose `Enabled` and `BlockedReason`, keeping
    durable worker services aligned with Python and Node planners.
  - Parser and Go tests now cover disabled action-plan targets.
  - Disabled native `<fieldset>` state now propagates to descendant native
    controls, so radios, textareas, selects, and buttons inside locked groups
    expose `attrs.disabled` directly.
  - Added shared conformance fixture `015-action-state` covering disabled
    fieldset inheritance plus ARIA required/disabled promotion.
  - Updated the conformance index so adapter maintainers can promote
    action-state checks into SDK/parser release tests.
  - Native `<textarea disabled>` controls now preserve `attrs.disabled`, so
    compact action plans can avoid suggesting type/clear work on unavailable
    fields.
  - Native `<select disabled>` controls now preserve `attrs.disabled`, keeping
    dropdown availability visible to SDK/parser action-plan helpers.
  - ARIA widgets with `aria-required="true"` now promote `attrs.required`,
    making custom SaaS controls queryable by the same state contract as native
    required inputs.
  - ARIA widgets with `aria-disabled="true"` now promote `attrs.disabled`
    while still preserving the nested `attrs.aria.disabled` state for
    accessibility parity.
  - Added focused compiler coverage for disabled textarea/select controls and
    ARIA required/disabled custom controls.
- 2026-05-12:
  - ARIA landmark role parsing is now case-insensitive, so uppercase
    `role="MAIN"` and `role="NAVIGATION"` still compile into proper SOM
    regions.
  - Declarative shadow DOM extraction now recurses through non-semantic wrapper
    containers, preserving nested controls inside web components.
  - Python/Node parser and Go SDK action-plan helpers now include compact
    placeholder, description, required, disabled, and group metadata.
  - Added focused tests for uppercase ARIA landmarks, nested shadow-root
    controls, and enriched action-plan payloads.
  - Rust SOM compilation now emits labelled `group` elements for native
    `<fieldset>` controls and ARIA `group`/`radiogroup` widgets.
  - Fieldset groups now derive labels and `attrs.legend` from the first
    `<legend>`, and preserve `attrs.disabled` for disabled fieldsets.
  - CDP accessibility and DOM mappings now understand SOM `group` roles.
  - SOM schema, spec, Python/Node SDK types, Python/Node parser types, and Go
    SDK attrs now accept `group` roles and `attrs.legend`.
  - Parser and Go tests now cover group/legend payload compatibility.
  - Added a shared conformance fixture for fieldset/legend and ARIA radiogroup
    semantics.
- 2026-05-11:
  - Rust SOM compilation now resolves nested `<label>` controls, including
    wrapped checkboxes and selects, without leaking option text into labels.
  - Landmark and form region labels now resolve `aria-labelledby`, aligning
    region naming with browser accessibility snapshots.
  - Input buttons now expose `value` as their accessible label and retain
    normalized `attrs.input_type` for `submit`, `button`, and `reset` controls.
  - Go SDK types now parse current SOM fields for `shadow`, accessible
    descriptions, `name`, `autocomplete`, ARIA state, details, and iframe attrs.
  - Go query helpers now traverse shadow-root elements for id, role, text,
    interactivity, and flattened element queries.
  - Go now exposes `FindByAction`, `FindByHint`, and `GetActionPlan` so action
    planning is available across Rust output, Python/Node parser packages, and
    Go consumers.
  - SOM metadata now counts elements and interactive controls inside shadow
    roots, keeping reported counts aligned with the full queryable SOM tree.
  - `aria-labelledby` now takes precedence over `aria-label` when resolving
    element labels, matching browser accessible-name expectations more closely.
  - SOM attrs now include accessible descriptions resolved from
    `aria-describedby` and `aria-description`, with schema and Python/Node type
    parity.
  - Compiler tests now cover label precedence, accessible descriptions, and
    shadow-root meta counts.
- 2026-05-10:
  - Rust SOM compilation now resolves labels from `aria-labelledby` and
    external `<label for="...">` controls, improving accessible-name parity
    with Playwright-style snapshots.
  - Python parser `from_plasmate()` now extracts SOM JSON from mixed CLI output
    with progress/log lines while preserving support for wrapped `{ "som": ... }`
    responses.
  - Node parser `fromPlasmate()` now accepts wrapped `{ som: ... }` payloads in
    clean and mixed CLI output, matching Python parser behavior.
- 2026-05-07:
  - SOM JSON Schema now accepts current Rust core output for `shadow`,
    `iframe`, `details`, ARIA state, and actionability attrs.
  - Python and Node parser packages now expose shadow-root types and traverse
    `shadow.elements` in element, text, link, and interactive queries.
  - Python and Node SDK query helpers now find shadow-root elements by id, role,
    text, and actionability so web-component UIs stay reachable to agents.
- 2026-05-09:
  - Python and Node parser packages now expose action and hint lookup helpers
    so agents can query `click`, `type`, `required`, and similar SOM metadata
    directly.
  - Python and Node parser packages now expose compact action-plan helpers with
    element ids, roles, actions, labels, hrefs, names, and input types for
    Stagehand-style planning without re-walking the full SOM.
  - Node parser compression-ratio handling now matches Python by returning
    infinity when `som_bytes` is zero instead of reporting a misleading zero.
- 2026-05-06:
  - SOM link deduplication now preserves case-sensitive URL paths while still
    stripping fragments and duplicate trailing slashes.
  - Input type and ARIA role parsing is more tolerant of real-world casing, so
    `type="SUBMIT"` and upper-case custom roles no longer lose actionability.
  - Custom controls now retain `contenteditable`, `tabindex`, `name`, and
    `autocomplete` attributes in SOM attrs, improving parity with
    accessibility-snapshot competitors.
  - MCP `extract_text` truncation is UTF-8 safe, preventing panics when
    `max_chars` cuts through multibyte content.
- 2026-05-05:
  - Cache prefetch URL extraction now walks nested SOM children and shadow-root
    elements, deduplicates URLs while preserving order, and excludes non-HTTP
    schemes.
  - Cache URL normalization now lowercases scheme/host through URL parsing
    without corrupting case-sensitive paths.
  - MCP `extract_text` and `extract_links` now include shadow-root content, so
    declarative web components are not invisible to agents.
- 2026-05-04:
  - Added region-id selector support while keeping HTML id selection.
  - Added common ARIA widget role mapping into actionable SOM elements.
  - Hardened inline hidden-style stripping against spacing and casing variants.
  - Updated roadmap direction around cached structured actions, MCP
    distribution, and accessibility/SOM parity.

## Next Steps

- Implement selector-aware SOM cache entries for `main`, `form`, and `#id`
  prompts.
- Add trace export for MCP/AWP sessions so users can debug why an agent clicked
  or selected an element.
- Add conformance cases for ARIA-heavy SaaS pages, especially disabled and
  required custom controls, and compare output against Playwright MCP
  snapshots.
- Wire `015-action-state` into cross-adapter parser/SDK conformance runners so
  inherited disabled state stays synchronized outside Rust.
- Promote the new Browser Use and LangChain adapter availability checks into a
  shared cross-adapter fixture runner that also exercises the Vercel AI
  availability helper once package dependencies are pinned in this repo.
- Promote fieldset/legend group semantics into shared conformance fixtures
  alongside cross-adapter accessible-description cases.
- Add shared conformance for nested shadow-root controls and enriched
  action-plan metadata.
- Promote the new SDK/parser shadow-root and Go action-plan tests into shared
  conformance fixtures that run against every adapter before release.
- Audit ecosystem repos for stale install docs, tool counts, and schema drift.
- Promote action-plan helper parity into framework integrations so every
  adapter exposes the same compact action target contract.
