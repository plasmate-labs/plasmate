# Plasmate PRD: Agent Stickiness and Roadmap Direction

Last updated: 2026-05-15

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

2026-05-13 Vercel AI helper read: current competitor docs keep pushing action
menus closer to app code. Stagehand documents `observe()` as a way to plan,
cache, and validate actions; Browserbase frames repeated automation around
cached selectors and inspectable sessions; Playwright MCP keeps fresh refs tied
to the current snapshot; and hosted browser platforms keep making managed
sessions easy to buy. Plasmate's sticky local-first answer is to make framework
apps consume SOM action menus directly: Vercel AI users need a helper that
normalizes availability, filters blocked targets, and formats compact action
menus before the model sees them.

2026-05-13 Vercel AI extraction read: official docs continue to reward
application-level action menus. Playwright MCP snapshots expose fresh refs,
Stagehand v3 `observe()` returns cacheable structured actions, Firecrawl
Interact resumes scraped browser sessions, Browser Use Cloud sells CDP sessions
with profile state, and Cloudflare Browser Run/WebMCP is experimenting with
typed browser-native tools. Plasmate should keep the local-first wedge but make
raw SOM responses directly consumable by framework apps, so Vercel AI projects
can extract, filter, and format action menus without reimplementing parser
logic.

2026-05-13 cache-key read: current browser-agent products are teaching teams
to expect reusable action memory, not only a fresh page snapshot. Playwright MCP
keeps refs bound to the current accessibility snapshot, while Stagehand and
Browserbase emphasize cached actions/selectors for repeated workflows. Plasmate
should keep ids as the execution target, but SDK and framework action plans
also need deterministic `cache_key` values so apps can compare, store, and
dedupe recurring local SOM actions without cloud selector memory.

2026-05-13 cache-key parity read: the current competitive pressure is not just
to invent action memory, but to make it portable in the places developers
already wire agents. Playwright MCP snapshots remain current-state refs,
Stagehand/Browserbase cache resolved actions, Firecrawl Interact resumes hosted
browser sessions, Browser Use Cloud sells CDP/profile sessions, and
Cloudflare's WebMCP work points toward typed browser-native tools. Plasmate's
stickier answer is local action memory with a consistent `cache_key` in Go,
Browser Use, LangChain, Vercel AI, and parser packages so teams can reuse
targets without switching to hosted selector storage.

2026-05-13 fixture-manifest read: official docs and competitor positioning keep
raising the same adoption bar: structured refs, cacheable actions, and hosted
session traces only retain users if the action surface is trustworthy. Plasmate
should treat its broad repo surface as one product contract. The next
stickiness step is a shared expectation manifest that Browser Use, LangChain,
and Vercel AI consume directly so availability, blocked reasons, required
state, group context, descriptions, and cache keys cannot drift in separate
adapter tests.

2026-05-13 SDK-manifest read: current browser-agent positioning keeps moving
trust from isolated helpers into repeatable conformance. Playwright MCP refs
are snapshot-scoped, Stagehand action caching depends on validated reusable
targets, and Browserbase/Cloudflare traces make drift inspectable after a run.
Plasmate's local-first response is to make the shared action manifest a
cross-language contract: parser packages and SDKs should prove they emit the
same compact action targets as framework adapters before action memory becomes
a release claim.

2026-05-13 release-gate read: current official docs sharpen the same lesson.
Playwright MCP documents accessibility snapshots as the interaction surface
with refs stable only inside the current snapshot, Stagehand v3 frames
`observe()` as a way to plan, validate, and cache executable actions, and
Firecrawl Interact resumes scraped browser sessions with optional persistent
profiles. Plasmate should keep the local-first wedge and make release
conformance the retention feature: broad SDK and adapter coverage matters only
if one command proves the action menu contract has not drifted.

2026-05-13 semantics-polish read: current docs keep compressing the category
around reusable action surfaces. Playwright MCP snapshots expose roles and refs
from the accessibility tree, Stagehand `observe()` and caching reward stable
action descriptions, Firecrawl Browser Sandbox sells managed browser
execution, Crawl4AI is broadening LLM-friendly crawling toward cloud scale,
and Cloudflare WebMCP points toward typed website-provided tools. Plasmate
should keep its local-first SOM wedge, but small semantic gaps now hurt
stickiness: search landmarks, ARIA menu item variants, CSS-hidden content, and
case-sensitive URL contracts need to behave like browser accessibility state
without forcing agents back to raw DOM.

2026-05-13 fixture-contract read: current competitor comparisons keep
rewarding tools that make action state reusable in normal app code, not only
inside the browser engine. Browser Use, Stagehand, Playwright MCP,
Browserbase, and Firecrawl all compete on whether agents can trust compact
targets after the first observation. Plasmate's broad repo surface should turn
that pressure into a release habit: every newly supported browser semantic
needs a shared fixture or manifest entry before it is treated as a durable
product contract.

2026-05-13 role-fallback read: fresh official-doc review keeps pointing to
the same retention layer. Playwright MCP snapshots expose current refs from
accessibility state, Stagehand documents `observe()` as a planning and caching
surface, Firecrawl Interact combines scrape sessions with prompt/code
interaction and persistent profiles, Browser Use Cloud sells CDP browser
sessions with profile state, and Crawl4AI is expanding LLM-friendly crawling
toward cloud scale. Plasmate should continue avoiding a hosted-browser pivot
and instead make the local SOM compiler behave more like browser
accessibility state on messy production markup, including fallback ARIA role
tokens and hidden-state variants.

2026-05-13 control-state read: official docs continue to make action menus
the durable product layer. Playwright MCP refs are useful only when they match
current accessibility state, Stagehand's `observe()` cache is valuable only
when reused actions still reflect page state, and managed browser platforms
are selling session continuity around the same problem. Plasmate's local-first
answer is to make compact action targets carry enough live control state
(`value`, `checked`, and ARIA checked state) for agents to reuse cached plans
without re-reading raw DOM.

2026-05-13 ARIA state-cues read: the newest competitor docs sharpen the same
lesson. Playwright MCP snapshots require agents to act from current page state,
Stagehand v3 documents local and Browserbase action caches that must validate
before reuse, Cloudflare Browser Run/WebMCP pushes typed website actions, and
hosted browser products keep selling persistent sessions and traces. Plasmate
should stay local-first, but compact action plans need ARIA expanded, pressed,
and selected state so agents can tell whether menus, toggle buttons, and
selectable custom controls already match the intended workflow.

2026-05-13 ARIA relationship read: current docs add one more retention signal.
Playwright MCP still binds refs to fresh snapshots, Stagehand and Browserbase
reward cached actions that can be validated before reuse, Browser Use Cloud
sells profiles/CDP sessions for repeated workflows, Firecrawl Interact keeps
browser state alive after scrape, and Cloudflare Browser Run/WebMCP is testing
typed page-provided tools. Plasmate should not pivot into hosted execution,
but action menus should expose relationship state (`aria-current`,
`aria-controls`, and `aria-haspopup`) so agents know which target is already
current, what panel a control affects, and whether an action opens a menu,
listbox, or dialog.

2026-05-13 readonly/value read: fresh official-doc review keeps reinforcing
state validation before action reuse. Playwright MCP refs stay scoped to the
current snapshot, Stagehand `observe()` caches need validation before replay,
Firecrawl and Browser Use package persistent sessions around changing forms,
and hosted browser platforms sell traces for debugging state drift. Plasmate's
local-first answer should keep compact action menus honest by preserving
read-only blockers and current textarea/select values, while parsing ARIA
boolean state like production markup rather than ideal lowercase examples.

2026-05-13 validation-constraint read: current browser-agent docs keep making
the same distinction: refs and cached actions are only useful when they carry
enough live form context to validate before replay. Playwright MCP snapshots
are fresh accessibility state, Stagehand `observe()` actions can be cached
locally or on Browserbase, and Browser Use/Firecrawl keep selling stateful
sessions around repetitive form workflows. Plasmate should keep the local-first
wedge by carrying input guidance and validation state in compact action menus:
`autocomplete`, `minlength`, `maxlength`, `pattern`, and `aria-invalid` should
mean the same thing in Rust, schema, parser packages, SDKs, and adapters.

2026-05-13 input-affordance read: current browser-agent docs keep validating
small, browser-like action menus over broad hosted pivots. Playwright MCP refs
remain tied to the current accessibility snapshot, while Stagehand and
Browserbase make cached actions valuable only when the cached target still
matches the field's current affordances. Plasmate should carry input modality
and autocomplete-widget cues (`inputmode`, `enterkeyhint`,
`aria-autocomplete`, and `aria-activedescendant`) through the same shared
manifest so agents can choose credential data, keyboard submit behavior, and
active suggestion state without raw DOM recovery.

2026-05-13 keyboard-affordance read: current Playwright MCP and Stagehand
docs keep emphasizing fresh, validated action state before replay, while
Browserbase and Browser Use sell observability around repeated workflows.
Plasmate's local-first answer should include keyboard and custom-role cues in
the same portable action contract: `accesskey`, `aria-keyshortcuts`, and
`aria-roledescription` help agents choose and explain reusable targets without
falling back to raw DOM or screenshots.

2026-05-14 form-relation read: current official docs continue to reward
action menus that can be validated before replay. Playwright MCP refs remain
snapshot-scoped, Stagehand `observe()` and action caches depend on matching
current page state, and Firecrawl plus Browser Use sell persistent sessions
around repeated form work. Plasmate should keep the local-first wedge by
preserving form association and error relationships in compact targets:
`form`, `list`, and `aria-errormessage` let agents choose the right submit
scope, understand datalist suggestions, and explain invalid fields without
raw DOM recovery.

2026-05-14 live-region read: current official docs make validation-before-replay
even more important. Playwright MCP refs are scoped to the current
accessibility snapshot, Stagehand caches only pay off when the observed page
state still matches, and Browser Use/CDP sessions keep dynamic app state alive
for repeated workflows. Plasmate should expose lightweight ARIA live-region
state in compact targets: `aria-busy`, `aria-live`, `aria-atomic`, and
`aria-relevant` tell agents whether a control or result region is updating and
how status feedback will announce without forcing raw DOM recovery.

2026-05-14 popover-command read: browser action surfaces are expanding beyond
ARIA-only state. MDN now documents Popover API invoker relationships and the
newer `commandfor`/`command` button attributes, while Chrome positions command
buttons as a declarative replacement path for popover-specific controls.
Plasmate should carry these native relationships in SOM action menus so agents
can tell which button opens, hides, or toggles which panel before replaying a
cached local action.

2026-05-14 relationship-context read: official Playwright MCP docs continue to
make current structured snapshots and refs the agent interaction unit, while
Stagehand and Browserbase emphasize cached action replay only after validating
the target still matches. Plasmate's sticky local-first answer is richer
relationship context in compact targets: `aria-owns`, `aria-flowto`, and
`aria-details` let agents understand custom widget ownership, guided workflow
order, and detailed help panels without pulling raw DOM back into the prompt.

2026-05-14 link-navigation read: official Playwright MCP docs still teach
agents to act from fresh structured refs, while Stagehand action caching and
managed browser-session products make replay validation the retention layer.
Links are action targets too: Plasmate should preserve `target`, `rel`, and
`download` so local cached clicks can distinguish same-tab navigation,
new-context opens, relationship hints, and download side effects without
falling back to raw DOM inspection.

2026-05-14 range-orientation read: current browser-agent competitors keep
validating cached actions against fresh state before replay. Range controls,
sortable headers, and oriented composite widgets are common SaaS surfaces where
an agent needs numeric bounds and current ARIA value state before choosing an
action. Plasmate should keep cache keys target-focused while exposing `min`,
`max`, `step`, `aria-valuemin`, `aria-valuemax`, `aria-valuenow`,
`aria-valuetext`, `aria-orientation`, and `aria-sort` as compact action-plan
context.

2026-05-14 widget-affordance read: current browser-agent products keep moving
from element identity toward current, validated widget state before replay.
ARIA textboxes, listboxes, and custom inputs often expose read-only,
multiline, and multiselectable affordances without native HTML equivalents.
Plasmate should keep cache keys stable while surfacing `aria-readonly`,
`aria-multiline`, and `aria-multiselectable` in compact action targets so
agents avoid typing into read-only custom controls and choose the right
selection strategy for composite widgets.

2026-05-14 set-position read: current Playwright MCP and Stagehand docs keep
making fresh structured state and cache validation the action surface. Tree,
menu, and listbox widgets need ordinal context as much as current value state:
`aria-level`, `aria-posinset`, and `aria-setsize` tell agents whether a target
is nested, where it sits in a collection, and whether a cached navigation plan
still points at the expected item. Plasmate should surface these cues without
changing deterministic action cache keys.

2026-05-14 text-entry-affordance read: current competitor docs and developer
commentary keep validating compact, fresh action menus over full-DOM recovery.
Stagehand-style cached actions only stay useful when the field's typing
affordances have not drifted, and Playwright MCP-style snapshots make the
current accessibility state the selection surface. Plasmate should preserve
small but practical text-entry cues such as `spellcheck`, `autocapitalize`,
`dirname`, and `aria-placeholder` across the same manifest so agents understand
keyboard behavior, language direction capture, and custom textbox prompt text
without changing deterministic cache keys.

2026-05-14 upload-affordance read: current browser-agent products keep
converging on replayable action menus, but production SaaS workflows often
block on file evidence, screenshots, resumes, and media uploads. Plasmate
should treat upload controls as first-class local action targets by surfacing
`name`, `accept`, `capture`, and native `multiple` state across SDKs and
adapters. The practical retention reason is simple: agents can validate that a
cached upload plan still targets the right field and file type before asking a
user or toolchain for a file.

2026-05-14 form-submission-context read: Playwright MCP refs remain scoped to
the current snapshot, while Stagehand and Browserbase make cached action replay
dependent on validating that the current target still matches the stored plan.
For SaaS workflows, the target is often the whole submission contract, not only
one input. Plasmate should surface form-level `action`, `method`, `target`,
`enctype`, `novalidate`, `accept-charset`, and `autocomplete` as compact
action-plan context so agents can distinguish upload, checkout, and settings
forms before reusing local action memory.

2026-05-14 submitter-override read: current browser-agent docs keep
validating action replay against fresh structured state, and SaaS pages often
route different submit buttons from the same form to different endpoints,
methods, targets, or validation modes. Plasmate should expose native submitter
override cues (`button_type`, `formaction`, `formmethod`, `formenctype`,
`formtarget`, and `formnovalidate`) across the shared action manifest so local
cached submit actions can verify the exact submission path without raw DOM
recovery.

2026-05-14 ARIA action-role read: current Playwright MCP docs keep structured
accessibility refs as the interaction surface, while Stagehand v3 documents
`observe()` as a way to discover, validate, and cache executable actions.
Production SaaS apps often expose controls through ARIA-only roles instead of
native elements, so Plasmate should continue widening local role parity before
hosted infrastructure work. `slider`, `spinbutton`, and `option` are small but
sticky action-role gaps because agents need to adjust numeric settings and
choose custom listbox options without falling back to raw DOM recovery.

2026-05-14 inert-availability read: current docs and competitor positioning
keep moving action replay toward validated current state. Playwright MCP refs
are fresh-snapshot handles, Stagehand/Browserbase cache actions only after
state validation, and Cloudflare Browser Run/WebMCP makes typed interaction
contracts more visible. Plasmate should treat native `inert` as an
availability gate in the local action contract: agents should still see the
target for planning and explanation, but compact action menus must mark it
unavailable with `blocked_reason="inert"` before replay.

2026-05-14 image submitter read: official Playwright MCP docs continue to make
fresh structured snapshots the interaction unit, while Browserbase/Stagehand
now market cached action validation as a cost and latency win and Cloudflare
Browser Run is widening CDP/MCP distribution for hosted browser sessions.
Plasmate should keep the local-first action contract precise on ordinary HTML
submitters before chasing hosted infrastructure. Graphical submit inputs are a
small but sticky SaaS gap: `input type="image"` should be a clickable submitter
with `button_type`, `alt`, and `src` context so cached submit plans can still
recognize branded/icon-only buttons.

2026-05-14 submitter-fidelity read: current competitor docs keep rewarding
validated action menus over raw browser control. Playwright MCP refs are
snapshot-scoped, Stagehand/Browserbase caches observed actions after state
validation, and Cloudflare Browser Run/WebMCP is growing hosted interaction
surfaces. Plasmate should keep tightening local SOM semantics: native
`<button>` values, browser-default button type normalization, hidden nested
controls, and inert shadow-root actions are small reliability gaps that matter
when teams replay the same SaaS form workflow hundreds of times.

2026-05-14 hidden-descendant text read: current browser-agent products make
fresh structured state the action source, while Stagehand/Browserbase action
caching only works when observed targets still match the current page.
Plasmate's local SOM contract should therefore filter hidden descendants from
every visible surface, not only from standalone elements. Hidden copy leaking
into parent paragraphs, button names, labels, select options, lists, or table
cells weakens cache validation and makes agents choose actions based on text a
human cannot see.

2026-05-14 select-option state read: current Playwright MCP docs still bind
actions to fresh structured snapshots, Browserbase/Stagehand now emphasizes
validated action caching, and Cloudflare Browser Run/WebMCP is expanding hosted
browser interaction contracts. Plasmate should keep the local-first wedge by
making ordinary select menus browser-accurate in SOM: default option values,
disabled options, optgroup labels, and multi-select state let cached local
plans validate menu choices without raw DOM recovery.

2026-05-14 select parity read: the sticky product promise is not just that Rust
emits better select state; every public contract has to accept and reuse it.
Single-select default values, disabled optgroup inheritance, select `size`,
option `group`, option `disabled`, and `selected_values` should move through
schema, parser packages, SDKs, and prompt renderers so cached menu plans remain
portable across the project's broad integration surface.

2026-05-14 shared-manifest parity read: current Playwright MCP snapshots,
Stagehand/Browserbase action caching, and Firecrawl/Browser Use managed
sessions all reinforce the same retention mechanism: action state is only
sticky when downstream app code trusts it without adapter-specific recovery.
Plasmate's broad repo surface should keep turning small native HTML cues into
shared manifest expectations. Graphical submitter `alt`/`src`, image-submit
`button_type`, and select `selected_values`/`size` are now compact target
contract fields rather than parser-only details.

2026-05-14 relationship-context read: fresh competitor review still points to
validated current-state action replay rather than a hosted-browser pivot.
Playwright MCP keeps refs tied to the current structured snapshot,
Stagehand/Browserbase makes cached actions valuable only after target
validation, and Firecrawl/Browser Use keep packaging managed sessions for teams
buying infrastructure. Plasmate should keep making local compact targets more
explainable: native `title` help text plus `aria-labelledby`/`aria-describedby`
ID relationships let agents verify why a control is named or described the way
it is without re-reading raw DOM.

2026-05-14 target-provenance read: the latest official docs still sharpen the
same adoption bar. Playwright MCP returns a fresh accessibility snapshot after
interactions, Stagehand/Browserbase caches only help when the current target
validates, Firecrawl and Browser Use sell hosted sessions, and Crawl4AI keeps
raising open-source extraction expectations. Plasmate should keep the
local-first wedge by preserving small source-level target cues: raw
`aria-label`/`aria-description` text explains label provenance, while native
`dir` and `lang` help agents validate bidirectional and multilingual form
targets before replaying a cached action.

2026-05-15 DOM-id bridge read: current browser-agent tooling keeps splitting
between fresh structured snapshots and repeatable cached execution. Playwright
MCP exposes snapshot refs, Stagehand/Browserbase cache selectors after
validated observations, and hosted browser platforms compete on session
infrastructure. Plasmate's sticky local-first answer is to keep the original
DOM id portable as `html_id`: agents can plan from stable SOM ids while still
bridging to `document.getElementById()` or CSS `#id` selectors when they need
to execute or debug against a live page.

2026-05-15 cache-key lookup read: fresh competitor research keeps validating
local action memory as the sticky surface. Playwright MCP refs are
current-snapshot handles, Stagehand/Browserbase now highlight cached action
selectors with page-state validation, and Firecrawl/Crawl4AI-style tools keep
pressuring extraction breadth. Plasmate should make deterministic `cache_key`
values directly resolvable to current compact targets across parser packages
and SDKs, not just generated.

2026-05-15 browser-default fidelity read: current Playwright MCP and Stagehand
docs keep making the fresh structured page state the validation layer before an
agent acts or replays a cached action. Plasmate should keep closing the small
HTML/browser-default gaps that make local SOM diverge from what a user can
actually submit: wrapped labels without ids, invalid input-type fallback, and
default/invalid form methods are low-level details that decide whether cached
SaaS form plans are trustworthy without raw DOM recovery.

2026-05-15 action-target ergonomics read: current official docs and market
commentary keep converging on "observe, validate, replay" loops. Playwright MCP
uses fresh accessibility snapshots for each action, Stagehand/Browserbase pair
observed actions with cached selector validation, and Firecrawl's extraction
surface keeps broadening toward agentic data workflows. Plasmate's sticky
answer is to make the local action menu cheap to use in normal app code:
resolve by SOM id, resolve by original DOM id, and filter available targets
without every SDK user hand-scanning the compact plan.

2026-05-15 action-index read: current competitor surfaces keep optimizing the
replay loop after the first observation. Playwright MCP refs are refreshed with
the page state, Stagehand/Browserbase cached actions depend on validating the
current target, Firecrawl is broadening agentic extraction, and Crawl4AI is
pressuring open-source crawler breadth. Plasmate should make the local action
menu indexable by the identifiers developers already store: stable SOM id,
deterministic `cache_key`, and original `html_id`, with an enabled-only index
for prompt-safe menus.

2026-05-15 framework-index read: current official docs keep confirming that
the sticky layer is the app's validation path, not only the engine output.
Playwright MCP returns fresh structured refs for each page state, Stagehand and
Browserbase validate cached actions before replay, Browserbase sells managed
agent sessions, and Crawl4AI keeps normalizing open-source LLM crawlers. The
next Plasmate improvement is to push indexed local replay helpers into Browser
Use, LangChain, and Vercel AI so framework users do not fork parser logic.

2026-05-15 plan-fingerprint read: current competitor motion keeps emphasizing
validated replay over one-off observations. Playwright MCP refs are only valid
for the current structured snapshot, Stagehand/Browserbase action caches rely
on validating the page still matches before execution, and Firecrawl/Crawl4AI
keep broadening extraction surfaces. Plasmate should add plan-level replay
checks on top of per-target `cache_key` lookup: a deterministic action-plan
fingerprint plus role/blocker counts lets apps detect drift before replaying
cached local actions.

2026-05-15 framework-fingerprint read: the same replay-validation pressure now
belongs at the framework edge. Browserbase/Stagehand are selling cached action
validation and observability where app developers wire agents, Playwright MCP
keeps refs scoped to the current snapshot, and Browser Use Cloud plus Firecrawl
make managed browser sessions easy to adopt. Plasmate's stickiness should come
from making Browser Use, LangChain, and Vercel AI expose the same local plan
fingerprints and summaries as the parser/SDK layer, so teams can gate replay
without writing adapter-specific drift checks.

2026-05-15 label-search parity read: current browser-agent tools keep teaching
developers to choose targets by human-facing names, not DOM trivia. Playwright
MCP snapshots present accessible names with refs, Stagehand/Browserbase cache
observed actions by matching current page state, and managed browser platforms
sell traces around repeated flows. Plasmate's broad SDK surface should make
label lookup boringly consistent: Python, Node, and Go app code should be able
to find labelled controls without depending on raw DOM traversal or parser-only
helpers.

2026-05-15 replay-coverage read: current official docs sharpen the same local
replay requirement. Playwright MCP refs are stable only inside the current
accessibility snapshot, Stagehand/Browserbase cache resolved actions only after
validating current page state, Browserbase adds session observability around
those runs, and Firecrawl continues broadening browser-session execution. The
next sticky Plasmate layer is not another hosted session surface; it is making
local action-plan summaries disclose whether current targets are actually
replay-indexable by `cache_key` and `html_id`, and whether cache-key collisions
would make replay ambiguous.

2026-05-15 replay-provenance read: current official docs keep validating a
fresh structured action surface with reusable local memory. Playwright MCP refs
remain snapshot-scoped, Stagehand/Browserbase caches actions only when current
page state still matches, and Browser Use Cloud separates browser sessions,
profiles, and agent runs for repeat work. Plasmate should keep avoiding a
hosted-browser pivot and instead preserve common app-owned replay anchors:
`data-testid`/`data-test`/`data-cy`/`data-qa`, `data-action`, and
`data-state` give local agents stable selector hints, intended action names,
and component state without raw DOM recovery.

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

- 2026-05-15:
  - Python/Node parser packages, Python/Node/Go SDKs, Browser Use, LangChain,
    and Vercel AI action-plan summaries now report replay coverage:
    `with_cache_key` / `withCacheKey`, `unique_cache_keys` /
    `uniqueCacheKeys`, `duplicate_cache_keys` / `duplicateCacheKeys`, and
    `with_html_id` / `withHtmlId`.
  - Replay validators can now reject ambiguous local action memory before
    executing a cached target, instead of discovering a duplicate cache key or
    missing source DOM-id bridge during action dispatch.
  - Focused parser, SDK, Browser Use, LangChain, and Vercel AI tests now assert
    replay-coverage summary fields against the shared action-availability
    fixture, keeping plan-level validation synchronized across runtimes.
  - Rust SOM compilation now preserves common replay provenance as
    `attrs.test_id`, `attrs.data_action`, and `attrs.data_state`.
  - JSON Schema/SOM spec, Python/Node parser packages, Python/Node/Go SDKs,
    Browser Use, LangChain, and Vercel AI now carry those provenance cues in
    compact action plans and prompt renderers.
  - Action cache keys remain unchanged for targets without provenance, while
    targets with real `test_id` or `data_action` include that anchor in their
    deterministic key so local replay memory can distinguish reused labels.
  - Wrapped `<label>` controls without an `id` now resolve accessible labels by
    DOM path, so ordinary SaaS forms no longer need explicit ids for local
    action plans to retain human-facing field names.
  - Invalid or whitespace-padded native `<input type>` values now normalize to
    browser behavior: known types are trimmed/lowercased and unknown types
    emit `attrs.input_type="text"`.
  - Form regions now expose browser-default method semantics: missing and
    invalid methods compile as `GET`, while `method="dialog"` is preserved as
    `DIALOG`.
  - Added focused Rust compiler tests for id-less wrapped labels, input type
    fallback, and form method defaults so these browser-default cases can move
    into shared conformance next.
  - Python SDK `find_by_text()` now searches both visible text and control
    labels, and adds `exact=True` for case-sensitive label/text matching.
  - Node SDK `findByText()` now searches both visible text and control labels,
    and accepts `{ exact: true }` for case-sensitive matching.
  - Go SDK now exposes `FindByTextExact()` alongside the existing
    label-aware `FindByText()`, completing SDK parity with parser package text
    lookup behavior.
  - Browser Use now exposes sync and async action-plan fingerprint and summary
    helpers, and page contexts include full/enabled fingerprints plus enabled
    and disabled counts before listing compact targets.
  - LangChain now exports `som_to_action_plan_fingerprint()` and
    `som_to_action_plan_summary()` alongside the existing action-plan and
    replay-index helpers.
  - Vercel AI now exports `getPlasmateActionPlanFingerprint()` and
    `getPlasmateActionPlanSummary()` so apps can store a plan-level drift gate
    next to cached action ids.
  - Framework docs and tests now cover plan-level drift validation across
    Browser Use, LangChain, and Vercel AI, keeping adapter behavior aligned
    with parser/SDK fingerprint helpers.
  - Auth profile plaintext detection now requires parseable JSON instead of
    checking only the first non-whitespace byte, preventing encrypted profile
    bytes that happen to start with `{` from being misclassified as legacy
    plaintext.
  - Python/Node parser packages and Python/Node/Go SDKs now expose
    deterministic action-plan fingerprints plus compact action-plan summaries
    with total/enabled/disabled counts, role counts, and blocked-reason counts.
  - These helpers make local replay validation cheaper at the plan level:
    apps can compare the current compact action surface before resolving a
    stored target by SOM id, `cache_key`, or `html_id`.
  - Vercel AI now exposes action-plan replay indexes plus lookup helpers by
    SOM id, deterministic `cache_key`, and original `html_id`.
  - Browser Use now exposes enabled-only action-plan extraction and replay
    indexes, including async variants, so agents can validate cached targets
    before showing model-facing menus.
  - LangChain now exposes parser-backed `som_to_action_plan()` and
    `som_to_action_plan_index()` helpers for raw SOM dicts, keeping structured
    replay validation out of formatted prompt text.
  - Framework adapter tests now assert enabled-only and indexed replay behavior
    against the shared action-availability fixture.
  - Python and Node parser packages now export cache-key lookup helpers
    (`find_action_target_by_cache_key()` / `findActionTargetByCacheKey()`), so
    app code can validate a stored action key against the current SOM action
    plan without rescanning manually.
  - Python, Node, and Go SDKs now expose the same cache-key lookup flow
    (`find_action_target_by_cache_key()` / `findActionTargetByCacheKey()` /
    `FindActionTargetByCacheKey()`), keeping local action memory portable
    across orchestration and worker runtimes.
  - Parser/SDK docs and tests now cover the cache-key lookup path, preserving
    deterministic `cache_key` values while improving replay ergonomics.
  - Python/Node parser packages and Python/Node/Go SDKs now expose direct
    compact-target lookup by SOM id and original DOM id, so agents can bridge
    stored plans, live selectors, and current SOM state without raw DOM scans.
  - Python/Node parser packages and Python/Node/Go SDKs now expose enabled-only
    action-plan helpers for the common "show the model usable targets" path.
  - Python/Node parser packages and Python/Node/Go SDKs now expose compact
    action-plan indexes keyed by SOM id, deterministic `cache_key`, and
    original `html_id`, giving replay validators O(1) target lookup without
    changing compact target identity.
  - Action-plan index helpers support enabled-only indexing so apps can build
    prompt menus and replay gates from the same current SOM plan.
  - Parser/SDK tests and docs now cover action-plan indexes across Python,
    Node, and Go, keeping lookup ergonomics synchronized across orchestration
    and durable worker runtimes.
  - Focused parser and SDK tests now cover id lookup, `html_id` lookup, and
    enabled-plan filtering against the shared action availability manifest.
  - Python/Node parser packages and Python/Node/Go SDK types now accept the
    Rust/SOM-spec `html_id` field, preventing parser drift when core output
    includes original DOM ids.
  - Python, Node, and Go query helpers now expose `find_by_html_id()` /
    `findByHtmlId()` / `FindByHTMLID()` so agents can resolve source DOM ids
    without scanning raw HTML.
  - Compact action-plan helpers and Browser Use, LangChain, and Vercel AI
    renderers now carry `html_id` without changing deterministic `cache_key`
    values, and the shared action-availability manifest asserts the field
    across parser, SDK, and framework surfaces.
- 2026-05-14:
  - Rust SOM now preserves source `aria-label`, `aria-description`, `dir`, and
    `lang` as `attrs.aria_label`, `attrs.aria_description`, `attrs.dir`, and
    `attrs.lang` on compact targets.
  - JSON Schema/SOM spec, Python/Node parser packages, Python/Node/Go SDKs,
    Browser Use, LangChain, and Vercel AI now carry those source-provenance and
    locale/direction cues through action-plan output without changing
    deterministic `cache_key` values.
  - The shared action-availability manifest now asserts `aria_label`,
    `aria_description`, `dir`, and `lang` on the email target so parser, SDK,
    and framework surfaces cannot drift on multilingual form context.
  - Rust SOM now preserves native `title` help text plus source
    `aria-labelledby` and `aria-describedby` IDREF relationships as
    `attrs.title`, `attrs.labelledby`, and `attrs.describedby`.
  - JSON Schema/SOM spec, Python/Node parser packages, Python/Node/Go SDKs,
    Browser Use, LangChain, and Vercel AI now carry `title`, `labelledby`, and
    `describedby` through compact action-plan output without changing
    deterministic `cache_key` values.
  - The shared action-availability manifest now asserts relationship-context
    cues on the email target, keeping parser, SDK, and framework surfaces
    synchronized for label/description provenance.
  - Rust SOM select extraction now follows browser default option-value
    semantics when an `<option>` omits `value`.
  - Select option summaries now preserve disabled option state and optgroup
    labels, giving agents enough context to avoid unavailable choices and
    explain grouped menus.
  - Multi-select controls now expose `attrs.selected_values` alongside the
    existing first selected `value`, so cached action plans can validate
    multiple current choices.
  - Single-select controls without explicit `selected` markup now mark the
    browser-default first option as selected and expose that value.
  - Disabled `<optgroup>` elements now propagate `disabled=true` to child option
    summaries, and explicit select `size` is preserved.
  - JSON Schema/SOM spec, Python/Node parser packages, Python/Node/Go SDKs,
    Browser Use, LangChain, and Vercel AI now accept or render
    `selected_values`, select `size`, option `disabled`, and option `group`.
  - The shared action-availability manifest now includes an image submitter
    with `button_type`, `input_type=image`, `alt`, `src`, `name`, and `value`,
    proving graphical submitter context across parser packages, SDKs, Browser
    Use, LangChain, and Vercel AI.
  - Compact action-plan helpers now expose `alt` and `src` for actionable
    graphical targets without changing deterministic `cache_key` values.
  - The shared manifest now asserts select `selected_values` and `size`,
    moving menu current-state parity from documentation into cross-runtime
    fixture coverage.
  - The Rust SOM compiler and JSON Schema now preserve link navigation cues:
    `target`, `rel`, and `download`.
  - Parser packages, Python/Node/Go SDKs, Browser Use, LangChain, and Vercel
    AI action-plan surfaces now expose `target`, `rel`, and `download`
    without changing deterministic action `cache_key` values.
  - The shared action-availability manifest now asserts link
    target/rel/download cues across parser, SDK, and framework outputs.
  - The Rust SOM compiler and JSON Schema now preserve native range/value
    constraints (`min`, `max`, and `step`) plus ARIA range, orientation, and
    sort cues (`aria-valuemin`, `aria-valuemax`, `aria-valuenow`,
    `aria-valuetext`, `aria-orientation`, and `aria-sort`).
  - Parser packages, Python/Node/Go SDKs, Browser Use, LangChain, and Vercel
    AI action-plan surfaces now expose `min`, `max`, `step`, `orientation`,
    `sort`, `valuemin`, `valuemax`, `valuenow`, and `valuetext` without
    changing deterministic action `cache_key` values.
  - The shared action-availability manifest now asserts range and ARIA
    orientation/value cues across parser, SDK, and framework outputs.
  - The Rust SOM compiler and JSON Schema now preserve ARIA widget affordance
    cues: `aria-readonly`, `aria-multiline`, and `aria-multiselectable`.
  - Parser packages, Python/Node/Go SDKs, Browser Use, LangChain, and Vercel
    AI action-plan surfaces now expose `readonly`, `multiline`, and
    `multiselectable`; ARIA read-only targets are marked unavailable with
    `blocked_reason="readonly"` without changing deterministic action
    `cache_key` values.
  - The shared action-availability manifest now asserts ARIA read-only gating
    plus multiline and multiselectable widget cues across parser, SDK, and
    framework outputs.
  - The Rust SOM compiler and JSON Schema now preserve ARIA set-position cues:
    `aria-level`, `aria-posinset`, and `aria-setsize`.
  - Parser packages, Python/Node/Go SDKs, Browser Use, LangChain, and Vercel
    AI action-plan surfaces now expose `level`, `posinset`, and `setsize`
    without changing deterministic action `cache_key` values.
  - The shared action-availability manifest now asserts ARIA set-position
    cues across parser, SDK, and framework outputs.
  - The Rust SOM compiler and JSON Schema now preserve text-entry affordance
    cues: native `spellcheck`, `autocapitalize`, `dirname`, and ARIA
    `aria-placeholder`.
  - Parser packages, Python/Node/Go SDKs, Browser Use, LangChain, and Vercel
    AI action-plan surfaces now expose `spellcheck`, `autocapitalize`,
    `dirname`, and `aria_placeholder` without changing deterministic action
    `cache_key` values.
  - The shared action-availability manifest and `016-action-semantics`
    conformance fixture now assert these text-entry affordance cues across
    Rust, parser, SDK, and framework outputs.
  - The Rust SOM compiler and JSON Schema now preserve upload action cues:
    native `accept`, `capture`, and input `multiple`, while the shared manifest
    now includes field `name` identity for deterministic target caching.
  - Parser packages, Python/Node/Go SDKs, Browser Use, LangChain, and Vercel
    AI action-plan surfaces now expose `name`, `accept`, `capture`, and
    `multiple` for upload and multi-select workflows.
  - The shared action-availability manifest now asserts upload constraints and
    native multiple-selection state across parser, SDK, and framework outputs.
  - The Rust SOM compiler and JSON Schema now preserve form submission context:
    `target`, `enctype`, `novalidate`, `accept-charset`, and form-level
    `autocomplete`, alongside existing `action` and `method`.
  - Parser packages, Python/Node/Go SDKs, Browser Use, LangChain, and Vercel
    AI action-plan surfaces now expose `form_action`, `form_method`,
    `form_target`, `form_enctype`, `form_novalidate`,
    `form_accept_charset`, and `form_autocomplete` without changing
    deterministic action `cache_key` values.
  - The shared action-availability manifest now asserts form submission
    context across parser, SDK, and framework outputs.
  - The Rust SOM compiler now preserves submit-button override cues:
    `button_type`, `formaction`, `formmethod`, `formenctype`, `formtarget`,
    and `formnovalidate`, and JSON Schema/SOM spec docs accept those
    button-level form overrides.
  - Parser packages, Python/Node/Go SDKs, Browser Use, LangChain, and Vercel
    AI action-plan surfaces now expose those submit override cues so cached
    submit actions can validate the exact endpoint, method, encoding, target,
    and validation mode before replay.
  - The shared action-availability manifest now asserts submit-button override
    context across parser, SDK, and framework outputs.
  - The Rust SOM compiler now maps ARIA `slider` and `spinbutton` roles to
    actionable `text_input` targets and maps ARIA `option` to an actionable
    `button` target, covering custom numeric controls and listbox choices.
  - The `016-action-semantics` conformance fixture now asserts `slider`,
    `spinbutton`, and `option` action-role coverage, including current ARIA
    value and selected state.
  - SOM spec and generated docs now document the expanded ARIA action-role
    mapping so SDK and adapter maintainers know these roles are product
    surface, not incidental compiler behavior.
  - The Rust SOM compiler and JSON Schema now preserve native `inert` state,
    including inherited inert context for nested interactive controls.
  - Parser packages, Python/Node/Go SDKs, Browser Use, LangChain, and Vercel
    AI action-plan surfaces now expose `inert`, mark inert targets unavailable
    with `blocked_reason="inert"`, and keep deterministic action `cache_key`
    values target-focused.
  - The `015-action-state` conformance fixture and shared
    action-availability manifest now assert inert availability gating across
    Rust, parser, SDK, and framework surfaces.
  - The Rust SOM compiler now maps `input type="image"` to an actionable
    button instead of a text input.
  - Input-backed submitters (`submit`, `button`, `reset`, and `image`) now
    expose `button_type`, giving cached submit plans the same native action
    cue already used for `<button>`.
  - Graphical submit inputs now resolve labels from `alt` and preserve `alt`
    plus `src`, so agents can identify icon-only submitters without falling
    back to raw DOM or screenshots.
  - Native `<button>` submitters now preserve `attrs.value`, so cached submit
    plans can distinguish multiple same-name submit intents without raw DOM
    recovery.
  - Invalid native `<button type>` values now normalize to `button_type:
    "submit"`, matching browser default behavior for malformed production
    markup.
  - Stylesheet-hidden nested controls are no longer extracted as interactive
    child actions, preventing invisible controls from entering compact action
    menus.
  - Shadow-root actions under an inert host now inherit `attrs.inert`, so
    reusable action plans mark those targets unavailable instead of treating
    them as active web-component controls.
  - Descendant text extraction is now stylesheet-visibility aware for
    paragraphs and interactive names, preventing hidden nested copy from
    leaking into visible parent text or button labels.
  - Accessible label indexing now skips stylesheet-hidden fragments, so
    hidden label text no longer pollutes `label for` / `aria-labelledby`
    control names.
  - Structured select options, list items, table captions, and table cells now
    use the same visible-text filter, keeping compact content summaries aligned
    with what users and browser accessibility snapshots expose.
  - The Rust SOM compiler and JSON Schema now preserve ARIA relationship cues:
    `aria-owns`, `aria-flowto`, and `aria-details`.
  - Parser packages, Python/Node/Go SDKs, Browser Use, LangChain, and Vercel
    AI action-plan surfaces now expose `owns`, `flowto`, and `details`
    without changing deterministic action `cache_key` values.
  - The shared action-availability manifest now asserts ARIA
    owns/flowto/details relationship cues across parser, SDK, and framework
    outputs.
  - The Rust SOM compiler and JSON Schema now preserve native popover and
    command relationships: `popovertarget`, `popovertargetaction`,
    `commandfor`, `command`, and `popover`.
  - Parser packages, Python/Node/Go SDKs, Browser Use, LangChain, and Vercel
    AI action-plan surfaces now expose `popovertarget`,
    `popovertargetaction`, `commandfor`, and `command` without changing
    deterministic action `cache_key` values.
  - The shared action-availability manifest now asserts popover/command
    relationship cues across parser, SDK, and framework outputs.
  - The Rust SOM compiler and JSON Schema now preserve ARIA live-region cues:
    `aria-busy`, `aria-live`, `aria-atomic`, and `aria-relevant`.
  - Parser packages, Python/Node/Go SDKs, Browser Use, LangChain, and Vercel
    AI action-plan surfaces now expose `busy`, `live`, `atomic`, and
    `relevant` without changing deterministic action `cache_key` values.
  - The shared action-availability manifest now asserts live-region state
    across parser, SDK, and framework outputs.
  - The Rust SOM compiler and JSON Schema now preserve `form`, `list`, and
    `aria-errormessage`, adding form ownership, datalist, and error-message
    relationships to the compact action-state contract.
  - Parser packages, Python/Node/Go SDKs, Browser Use, LangChain, and Vercel
    AI action-plan surfaces now expose `form`, `list`, and `errormessage`
    without changing deterministic action `cache_key` values.
  - The shared action-availability manifest now asserts form-relation and
    error-message cues across parser, SDK, and framework outputs.
- 2026-05-13:
  - The Rust SOM compiler and JSON Schema now preserve native `accesskey` plus
    ARIA `keyshortcuts` and `roledescription`, adding keyboard/custom-role
    affordances to the compact action-state contract.
  - Parser packages, Python/Node/Go SDKs, Browser Use, LangChain, and Vercel
    AI action-plan surfaces now expose `accesskey`, `keyshortcuts`, and
    `roledescription` without changing deterministic action `cache_key`
    values.
  - The shared action-availability manifest now asserts keyboard and
    custom-role cues across parser, SDK, and framework outputs.
  - The Rust SOM compiler and JSON Schema now preserve `inputmode`,
    `enterkeyhint`, `aria-autocomplete`, and `aria-activedescendant`, extending
    validation-state work into input-affordance cues for cached form actions.
  - Parser packages, Python/Node/Go SDKs, Browser Use, LangChain, and Vercel
    AI action-plan surfaces now expose `inputmode`, `enterkeyhint`,
    `aria_autocomplete`, and `active_descendant` without changing deterministic
    action `cache_key` values.
  - The shared action-availability manifest now asserts input modality and
    autocomplete-widget state across parser, SDK, and framework outputs.
  - The Rust SOM compiler and JSON Schema now preserve form-entry constraints
    (`minlength`, `maxlength`, `pattern`) plus `aria-invalid`, extending
    current action-state fidelity into validation state.
  - Parser packages, Python/Node/Go SDKs, Browser Use, LangChain, and Vercel
    AI action-plan surfaces now expose `autocomplete`, length constraints,
    `pattern`, and `invalid` without changing deterministic `cache_key` values.
  - The shared action-availability manifest now asserts validation constraints
    and invalid state across parser, SDK, and framework outputs.
  - Native read-only input and textarea controls now preserve `attrs.readonly`;
    parser packages, Python/Node/Go SDKs, Browser Use, LangChain, and Vercel
    AI action-plan surfaces expose `readonly`, mark those targets unavailable,
    and use `blocked_reason="readonly"` without changing deterministic
    `cache_key` values.
  - Textarea content and selected `<select>` options now surface as compact
    target `value` fields, giving agents current form state before they reuse a
    cached type/select plan.
  - ARIA boolean state preservation now trims and parses `TRUE`/`FALSE`
    case-insensitively, keeping `expanded`, `pressed`, and related cues typed
    as booleans on messy production markup.
  - The shared action-availability manifest now asserts read-only blockers and
    selected-option values across parser, SDK, and framework surfaces.
  - The Rust SOM compiler and JSON Schema now preserve `aria-controls` and
    `aria-haspopup` in `attrs.aria`, joining existing `aria-current` support
    for browser-like action relationship state.
  - Python/Node parser packages, Python/Node/Go SDKs, Browser Use, LangChain,
    and Vercel AI action-plan helpers now expose `current`, `controls`, and
    `haspopup` without changing deterministic `cache_key` generation.
  - The shared action-availability manifest now asserts current-page links,
    controlled popup targets, and popup type cues across parser, SDK, and
    framework surfaces.
  - Python/Node parser packages, Python/Node/Go SDKs, Browser Use, LangChain,
    and Vercel AI action-plan helpers now expose ARIA `expanded`, `pressed`,
    and `selected` cues for interactive targets while keeping deterministic
    `cache_key` generation target-focused.
  - Prompt renderers for Browser Use, LangChain, and Vercel AI now include
    expanded/pressed/selected state alongside value and checked state so agents
    can reuse cached action menus without recovering raw DOM state.
  - The shared action-availability manifest now asserts ARIA expanded,
    pressed, and selected state across parser, SDK, and framework surfaces.
  - Parser packages, Python/Node/Go SDKs, Browser Use, LangChain, and Vercel
    AI action-plan helpers now preserve current control `value` fields for
    interactive targets without changing deterministic `cache_key` values.
  - Compact action targets now expose `checked` state from native
    `attrs.checked` and ARIA `aria.checked`, covering both checkbox/radio
    inputs and custom menu widgets.
  - The shared action-availability manifest now asserts value and checked
    state so framework prompt renderers cannot drift from parser/SDK action
    plans.
  - ARIA landmark role parsing now honors space-separated fallback tokens, so
    `role="utility search"` still compiles into a labelled search/navigation
    region instead of falling back to generic content.
  - ARIA widget role parsing now honors fallback tokens, preserving
    `menuitemcheckbox` and `menuitemradio` action targets when production
    markup includes unknown role tokens before the known role.
  - Hidden-element stripping now treats uppercase `aria-hidden="TRUE"` and
    inline `opacity: 0` as hidden state, aligning inline visibility handling
    with the stylesheet visibility parser.
  - The `016-action-semantics` conformance fixture now covers role fallback
    tokens, uppercase ARIA-hidden state, and inline opacity hiding alongside
    search landmarks, menu action targets, and stylesheet whitespace/casing.
  - Shared action-availability fixtures now include ARIA menu checkbox and
    radio targets so Browser Use, LangChain, Vercel AI, parser packages, and
    SDKs prove the same menu action contract.
  - Added `specs/conformance/016-action-semantics.html` and expected output to
    cover labelled search landmarks, menuitem checkbox/radio targets, and
    stylesheet hidden-rule whitespace/casing in one reusable fixture.
  - Added Rust compiler regression coverage for the new action-semantics
    fixture so the shared fixture cannot drift as documentation-only coverage.
  - ARIA `role="search"` now compiles into a labelled navigation region,
    preserving search landmarks that agents commonly use before selecting a
    query field.
  - ARIA `menuitemcheckbox` and `menuitemradio` now map to actionable checkbox
    and radio SOM roles, improving custom menu parity with browser
    accessibility snapshots.
  - Stylesheet visibility parsing now ignores all declaration whitespace and
    casing, so `DISPLAY\t:\nnone` and similar CMS output is stripped like
    simpler `display:none` rules.
  - A stale integration test now matches the case-sensitive URL path contract:
    `/Page`, `/page`, and `/PAGE` remain distinct while fragments and trailing
    duplicate slashes are still deduped.
  - Added `integrations/fixtures/action-availability.expected.json` as the
    shared expected compact action-target contract for the action availability
    SOM fixture.
  - Added `scripts/action-manifest-conformance.sh`, a release-gate command
    that runs the shared manifest checks across Browser Use, LangChain, Vercel
    AI, Python/Node parser packages, and Go/Python/Node SDKs.
  - Added `--quick` and `--full` modes to the action-manifest release gate,
    giving CI a narrow shared-manifest check while preserving the full local
    release command for semantic contract changes.
  - Added a GitHub Actions action-manifest job that installs Python, Node, and
    Go dependencies and runs the quick conformance gate on pushes and pull
    requests.
  - Fixture documentation now explains quick vs full release-gate usage and
    makes the shared manifest a pre-release check for action-plan semantics.
  - Node SDK `npm test` now builds and runs the action-plan fixture tests,
    making TypeScript client parity part of the package's normal test path.
  - Root and fixture docs now point maintainers at the shared release command
    so action-plan semantics changes update the SOM fixture and expectation
    manifest together.
  - Python SDK query helpers now expose `get_action_plan()` and
    `get_action_plan_cache_key()`, closing the action-plan parity gap between
    the client SDK and Python parser package.
  - Node SDK query helpers now expose `getActionPlan()` and
    `getActionPlanCacheKey()`, giving TypeScript app code the same compact
    local action memory surface as the parser package.
  - Python parser, Node parser, Go SDK, Python SDK, and Node SDK tests now
    consume the shared action availability manifest so cache keys,
    availability, required flags, groups, placeholders, and descriptions fail
    from one expected contract.
  - Browser Use adapter tests now validate rendered page context against the
    shared expectation manifest instead of hard-coded local cache-key and
    availability assertions.
  - LangChain adapter tests now validate SOM text output against the same
    expectation manifest, keeping text-only prompts aligned with Browser Use
    and Vercel AI.
  - Vercel AI runtime fixture tests now compare extracted action targets with
    the shared manifest and verify cache-key uniqueness across the fixture.
  - Added integration fixture documentation so future adapter updates know to
    update SOM fixtures and expected action contracts together.
  - Go SDK action plans now include deterministic `CacheKey` values plus
    `GetActionPlanCacheKey()`, completing cache-key parity for durable worker
    services that consume SOM outside Python/Node agent orchestration.
  - Browser Use page contexts now render action-plan `cache_key` flags beside
    availability state, making cached local action menus visible in prompt
    context as well as raw `extract_action_plan()` results.
  - LangChain SOM text now computes and renders deterministic `cache_key`
    flags for interactive targets, keeping text-only agent prompts aligned
    with parser package action plans.
  - Added focused Go, Browser Use, and LangChain fixture coverage for the
    cache-key contract across available, disabled, and grouped action targets.
  - Vercel AI compact action targets now include deterministic `cache_key`
    values, making formatted menus easier to cache, compare, and trace across
    repeated agent steps.
  - Node SOM parser action plans now include the same `cache_key` field and
    export `getActionPlanCacheKey()` for app code that builds cached workflows
    from compact targets.
  - Python SOM parser action plans now include `cache_key` and export
    `get_action_plan_cache_key()`, keeping Python agent code aligned with the
    Node and Vercel AI surfaces.
  - Added focused Vercel AI, Node parser, and Python parser tests for the
    deterministic cache-key contract.
  - Vercel AI SDK integration now exports `extractPlasmateActionTargets()` for
    deriving compact action targets directly from raw SOM responses, including
    nested children and shadow-root elements.
  - Vercel AI prompt formatting now includes blocked reasons, input type, and
    placeholder metadata so cached action menus carry the same field-selection
    cues exposed by parser action plans.
  - Added an executable Vercel AI fixture test that builds the package and
    validates extraction, availability filtering, and prompt formatting against
    the shared adapter SOM fixture.
  - Vercel AI action-target availability now treats any `blocked_reason` as
    unavailable, not only `blocked_reason="disabled"`, matching the broader
    action-plan contract used by parser and SDK helpers.
  - Vercel AI SDK integration now exports `normalizePlasmateActionTarget()` so
    app code can make implicit enabled state explicit without mutating cached
    action targets.
  - Vercel AI SDK integration now exports `preparePlasmateActionPlan()` for
    filtering unavailable targets and limiting compact action menus before a
    `generateText` or `streamText` call.
  - Vercel AI SDK integration now exports `formatPlasmateActionPlan()` so apps
    can pass a small, stable action menu into prompts or trace logs.
  - Added a TypeScript fixture-style compile check for the Vercel AI action
    helpers using the same availability, required, group, and description shape
    as the shared adapter fixture.
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
- Add conformance cases for ARIA-heavy SaaS pages, especially disabled,
  required, and read-only custom controls, and compare output against Playwright MCP
  snapshots.
- Wire `015-action-state` into cross-adapter parser/SDK conformance runners so
  inherited disabled state stays synchronized outside Rust.
- Promote the GitHub Actions action-manifest job from quick shared-manifest
  checks to full conformance once runtime and dependency caching are stable.
- Add dependency-cache tuning for the action-manifest job so cross-runtime
  conformance stays cheap enough to keep required.
- Wire `016-action-semantics` into parser/SDK and adapter conformance runners
  so search landmarks, fallback-token ARIA roles, menu roles, ARIA-hidden
  casing, and visibility-hidden variants stay synchronized outside Rust.
- Promote ARIA relationship-state cases (`aria-controls`, `aria-haspopup`,
  `aria-owns`, `aria-flowto`, and `aria-details`) from the shared action
  availability manifest into the broader `015-action-state` /
  `016-action-semantics` conformance suites.
- Promote link navigation cases (`target`, `rel`, and `download`) from the
  shared action availability manifest into broader Rust/parser/SDK and adapter
  conformance fixtures.
- Promote range and orientation cases (`min`, `max`, `step`,
  `aria-orientation`, `aria-sort`, and ARIA value state) into broader
  Rust/parser/SDK and adapter conformance fixtures.
- Promote ARIA widget affordance cases (`aria-readonly`, `aria-multiline`, and
  `aria-multiselectable`) into broader Rust/parser/SDK and adapter
  conformance fixtures.
- Add compiler/schema conformance for form validation constraints and
  `aria-invalid`, then promote the shared manifest cases into broader parser,
  SDK, and adapter fixtures.
- Promote input-affordance cases (`inputmode`, `enterkeyhint`, autocomplete
  widget state, active descendants, `spellcheck`, `autocapitalize`,
  `dirname`, `dir`, `lang`, `aria-label`, `aria-description`, and
  `aria-placeholder`) into broader parser, SDK, and adapter conformance
  fixtures once the shared action manifest remains stable.
- Promote upload-affordance cases (`accept`, `capture`, `multiple`, and
  stable field `name`) into broader Rust/parser/SDK and adapter conformance
  fixtures.
- Promote form-submission context cases (`form_action`, `form_method`,
  `form_target`, `form_enctype`, `form_novalidate`, `form_accept_charset`,
  and `form_autocomplete`) into broader Rust/parser/SDK and adapter
  conformance fixtures.
- Promote submit-button override cases (`button_type`, `formaction`,
  `formmethod`, `formenctype`, `formtarget`, and `formnovalidate`) into
  broader Rust/parser/SDK and adapter conformance fixtures.
- Promote graphical submitter cases from the shared action manifest into
  broader Rust/parser/SDK and adapter conformance fixtures so icon-only submit
  buttons stay synchronized outside the compact action-plan fixture.
- Promote inert availability cases into broader parser, SDK, and adapter
  conformance fixtures so blocked local action targets stay synchronized.
- Promote native button `value`, invalid button-type normalization,
  stylesheet-hidden nested actions, inert shadow-host inheritance, and hidden
  descendant text filtering into shared fixtures that cover parent text,
  labels, select options, lists, and tables.
- Promote remaining select-option state (`value` fallback text, optgroup
  `group`, option `disabled`, and default single-select selection) into broader
  Rust/parser/SDK and adapter fixtures now that `selected_values` and `size`
  are covered by the shared action manifest.
- Promote replay-provenance cases (`test_id`, `data_action`, and `data_state`)
  from the shared action-availability manifest into broader Rust/parser/SDK
  and adapter fixtures.
- Promote browser-default form fidelity cases (id-less wrapped labels,
  invalid input-type fallback, and default/invalid form method normalization)
  into shared Rust/parser/SDK and adapter fixtures.
- Promote keyboard-affordance cases (`accesskey`, `aria-keyshortcuts`, and
  `aria-roledescription`) into broader Rust/parser/SDK conformance fixtures
  once the shared action manifest remains stable.
- Promote form-relation cases (`form`, `list`, and `aria-errormessage`) into
  broader parser, SDK, and adapter conformance fixtures.
- Promote live-region cases (`aria-busy`, `aria-live`, `aria-atomic`, and
  `aria-relevant`) into broader Rust/parser/SDK conformance fixtures.
- Promote popover/command relationship cases (`popovertarget`,
  `popovertargetaction`, `commandfor`, and `command`) into broader
  Rust/parser/SDK conformance fixtures.
- Promote fieldset/legend group semantics into shared conformance fixtures
  alongside cross-adapter accessible-description cases.
- Add shared conformance for nested shadow-root controls and enriched
  action-plan metadata.
- Promote the new SDK/parser shadow-root and Go action-plan tests into shared
  conformance fixtures that run against every adapter before release.
- Audit ecosystem repos for stale install docs, tool counts, and schema drift.
- Promote action-plan helper parity into framework integrations so every
  adapter exposes the same compact action target contract.
- Promote action-plan index helpers into shared conformance so id, cache-key,
  `html_id`, and enabled-only replay indexes cannot drift across runtimes.
