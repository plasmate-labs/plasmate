# PRD: Agent Stickiness and Roadmap Direction

Last updated: 2026-05-15

## Product Thesis

Plasmate should be the local-first browser engine agents keep installed because it turns web pages into compact, stable, actionable state. The product is not a general scraper and should not compete feature-for-feature with hosted browser clouds. Its advantage is fast SOM output, predictable element ids, MCP-native tooling, and reusable page understanding without sending browsing data to a cloud API.

## Target Audience

- AI agent developers building MCP tools, coding agents, and research agents.
- Framework maintainers who need cheaper page context for Browser Use, LangChain, LlamaIndex, CrewAI, and adjacent stacks.
- Growth, sales, and ops teams that monitor authenticated or repetitive web workflows and need structured change detection.
- Privacy-sensitive teams that want local web extraction instead of hosted scraping APIs.

## Competitive Context

- Playwright MCP has made structured accessibility snapshots a baseline for browser-agent tools.
- Firecrawl is broadening from scraping into MCP search, extraction, browser sessions, and hosted deep research.
- Browserbase/Stagehand is emphasizing action caching and observability for repeated automation flows.
- Crawl4AI remains strong for open-source Python crawling and extraction, but carries Chromium/Playwright operational weight.

2026-05-05 market read: the strongest retention hooks are reusable structured state, cached repeated actions, and ecosystem-native distribution. Playwright MCP returns accessibility snapshots with stable refs for interaction, Stagehand now markets action caching as an LLM-cost and latency reduction path, and Firecrawl's MCP surface combines scraping, search, browser sessions, and deep research. Plasmate should not chase hosted anti-bot infrastructure as the main wedge; it should make local SOM snapshots more complete, reusable, and easy to verify across its many adapters.

2026-05-06 market read: competitors are converging on "agent-ready page state" as a retention mechanism. Playwright MCP's structured snapshots set the expectation that interactive elements carry stable refs, Stagehand's `observe()` and action caching make repeated workflows feel deterministic after the first run, Firecrawl's current MCP docs include interactive browser sessions alongside scrape/search/extract, and Skyvern continues to package screenshots plus DOM context for multi-step visual workflows. The clearest Plasmate answer is not a pivot into hosted browser clouds; it is tighter local SOM actionability, conformance fixtures, and deterministic cache/diff behavior across the many SDK and integration repos.

2026-05-07 market read: Playwright MCP keeps accessibility snapshots and stable refs as the default interaction layer, Stagehand v3 emphasizes `observe()` planning, action caching, and targeted iframe/shadow-root operation, while Firecrawl and Browser Use package managed browser sessions and persistent cloud profiles. Plasmate should stay local-first, but SDK/parser parity is now product stickiness: every adapter must parse and query the SOM shape the Rust core emits.

2026-05-09 market read: the trend has moved from "give the model page text" to "give the agent a reusable decision surface." Playwright MCP and Cloudflare's fork continue to validate structured accessibility snapshots without vision models, Stagehand is positioning `observe()` plus action caching as the path from natural-language intent to deterministic repeated actions, Firecrawl is bundling scrape/search/extract with agent and browser-session APIs, and Skyvern is selling visual workflow reliability with credential and enterprise controls. Plasmate should keep avoiding a hosted browser-cloud pivot. The stickier move is to make local SOM output easy to query as an action plan across every SDK and integration.

2026-05-10 market read: competitor messaging keeps converging on structured, reusable browser state. Playwright MCP and Cloudflare Browser Run emphasize accessibility snapshots over screenshots, Stagehand foregrounds `observe()` planning plus local/managed action caching, Firecrawl keeps broadening scrape/search/extract with browser sessions, and Skyvern focuses on end-to-end visual workflows. Plasmate should stay local-first, but accessible labels and parser tolerance are now stickiness features: agents need controls named the same way across Rust, Python, Node, CLI, and MCP output.

2026-05-11 market read: official docs still point to the same sticky layer: Playwright MCP centers accessibility snapshots with refs, Stagehand uses `observe()` to plan, validate, and cache executable actions, and Firecrawl plus Browser Use make managed sessions and persistent profiles easy to buy. Plasmate should refine local-first SOM contracts around names, descriptions, and full-tree metadata before hosted-browser features.

2026-05-11 Go parity read: Stagehand's action caching and Playwright MCP's snapshot refs both teach agent developers to expect a compact, reusable action surface in every language they use. Plasmate's repo already spans Rust, Python, Node, Go, MCP, CDP, AWP, parser packages, and framework integrations, so stickiness now depends on contract parity as much as core extraction quality. Go should not lag Python and Node on shadow-root traversal, action/hint lookup, or accessible description fields because teams adopting Plasmate across services will judge the product by the weakest SDK surface.

2026-05-11 Browser Run read: Cloudflare has rebranded Browser Rendering as Browser Run and is now positioning global headless browser sessions, Live View, recordings, human-in-loop, MCP/CDP support, and structured extraction as an AI agent browser platform. That makes the hosted infrastructure lane more crowded, not more attractive for Plasmate's near-term wedge. Plasmate should deepen the portable local snapshot contract: controls and regions need browser-like names, repeat runs need cacheable state, and every adapter should consume the same SOM shape without bespoke DOM recovery.

2026-05-12 market read: official docs continue to validate Plasmate's local SOM-first direction. Playwright MCP's snapshot model makes structured accessibility refs table stakes, Stagehand now packages `observe()` planning with local/managed action caching, Cloudflare Browser Run is selling hosted global browser sessions plus structured extraction, and Browser Use Cloud is packaging agents, direct CDP sessions, profiles, skills, proxies, and managed scale. The sticky counter-position is not another hosted browser fleet. It is a portable local action snapshot that accurately represents the form semantics agents need to reuse plans on SaaS pages.

2026-05-12 action-plan read: competitors are turning page understanding into validated action menus. Playwright MCP snapshots teach agents to select refs from the current accessibility tree, Stagehand `observe()` returns executable actions that can be cached, Firecrawl's MCP surface spans scrape/search/extract plus browser interaction, and Cloudflare Browser Run is layering MCP/CDP/WebMCP onto hosted sessions. Plasmate should keep hosted scale optional and make local SOM action plans richer.

2026-05-13 state-fidelity read: 2026 browser-agent commentary keeps converging on deterministic execution plus selective AI planning. Playwright/Playwright MCP owns stable execution and snapshots, Stagehand-style `observe()` APIs make ambiguous state cacheable, Browserbase/Browser Use/Skyvern compete on managed sessions and traces, and WebMCP remains a standards watch item. Plasmate should stay local-first and make the portable SOM action-state contract more exact across disabled, required, grouped, described, and shadow-root controls.

2026-05-13 conformance read: current competitor positioning raises the bar from "structured output exists" to "the action surface is safe to reuse." Playwright MCP snapshots expose refs after each action, Stagehand 3.3 adds strict structured outputs and clearer upload/action state, and managed browser platforms sell traces for post-run debugging. Plasmate's local-first response should be to make disabled and required state portable enough that agents do not need raw DOM recovery before reusing a cached plan.

2026-05-13 availability read: official Playwright MCP docs still make fresh structured snapshots with refs the interaction unit, Stagehand v3 documents `observe()` as a cacheable action menu, and Firecrawl/Browser Use keep expanding managed browser sessions. Plasmate should not pivot into hosted session infrastructure; the higher-stickiness move is to make local action plans safer by surfacing an explicit availability gate across SDKs.

2026-05-13 adapter read: current docs keep validating the framework edge as a retention surface. Playwright MCP tells agents to act from structured snapshots with fresh refs, Stagehand `observe()` turns page state into cacheable actions, Firecrawl Interact and Browser Use Cloud package managed browser sessions, profiles, and CDP access, and Cloudflare Browser Run is widening hosted MCP/CDP/WebMCP distribution. Plasmate should still avoid a hosted-infra pivot; the stickier move is to make Browser Use, LangChain, and Vercel AI adapters surface the same local action availability cues already present in parser/SDK helpers.

2026-05-13 fixture read: Playwright MCP snapshots and Stagehand cached actions both make state drift costly. Plasmate should turn shared adapter fixtures into a release gate so Browser Use, LangChain, Vercel AI, parser packages, and SDKs all preserve availability, required, group, type, and description cues.

2026-05-13 Vercel AI helper read: app frameworks are where reusable action menus become sticky. Stagehand `observe()` and Browserbase cached selectors normalize precomputed action menus, while Playwright MCP keeps refs tied to the current snapshot. Plasmate should make Vercel AI apps normalize, filter, limit, and format SOM action targets before the model sees them.

2026-05-13 Vercel AI extraction read: official docs continue to reward application-level action menus. Playwright MCP snapshots expose fresh refs, Stagehand v3 `observe()` returns cacheable structured actions, Firecrawl Interact resumes scraped browser sessions, Browser Use Cloud sells CDP sessions with profile state, and Cloudflare Browser Run/WebMCP is experimenting with typed browser-native tools. Plasmate should keep the local-first wedge but make raw SOM responses directly consumable by framework apps, so Vercel AI projects can extract, filter, and format action menus without reimplementing parser logic.

2026-05-13 cache-key read: reusable action memory is becoming an expectation, not a premium add-on. Playwright MCP refs remain snapshot-bound while Stagehand/Browserbase action caching pushes teams toward stored selectors and actions. Plasmate should keep local SOM ids as the execution target, but action plans also need deterministic `cache_key` values so apps can compare, dedupe, and cache recurring actions without hosted selector memory.

2026-05-13 cache-key parity read: action memory needs to be portable at the framework edge. Playwright MCP snapshots remain current-state refs, Stagehand/Browserbase cache resolved actions, Firecrawl Interact resumes hosted browser sessions, Browser Use Cloud sells CDP/profile sessions, and Cloudflare WebMCP points toward typed browser-native tools. Plasmate should make local cache keys consistent in Go, Browser Use, LangChain, Vercel AI, and parser packages before pursuing hosted selector storage.

2026-05-13 fixture-manifest read: structured refs, cacheable actions, and hosted session traces only retain users if the action surface is trustworthy. Plasmate should treat its broad repo surface as one product contract. The next stickiness step is a shared expectation manifest that Browser Use, LangChain, and Vercel AI consume directly so availability, blocked reasons, required state, group context, descriptions, and cache keys cannot drift in separate adapter tests.

2026-05-13 SDK-manifest read: browser-agent products are turning reusable action state into an audited contract, not a helper detail. Plasmate should extend the shared expectation manifest into parser packages and SDKs so local action memory remains portable across app code, framework prompts, and durable workers.

2026-05-13 release-gate read: Playwright MCP snapshots keep refs scoped to the current page state, Stagehand v3 `observe()` turns page state into cacheable actions, and Firecrawl Interact resumes browser sessions after scraping. Plasmate should make release conformance the local-first retention feature: one command should prove the broad SDK and adapter action-menu contract has not drifted.

2026-05-13 CI gate read: reusable action state is only sticky when developers can trust it before release. Playwright MCP refs, Stagehand cached actions, Firecrawl Interact sessions, Browser Use Cloud profiles, and Cloudflare WebMCP all push toward action surfaces that behave like infrastructure. Plasmate should make its local action manifest cheap to verify in CI and keep the full release gate available for semantic changes.

2026-05-13 semantics-polish read: current docs keep compressing the category around reusable action surfaces. Playwright MCP snapshots expose roles and refs from the accessibility tree, Stagehand `observe()` and caching reward stable action descriptions, Firecrawl Browser Sandbox sells managed browser execution, Crawl4AI is broadening LLM-friendly crawling toward cloud scale, and Cloudflare WebMCP points toward typed website-provided tools. Plasmate should keep its local-first SOM wedge, but small semantic gaps now hurt stickiness: search landmarks, ARIA menu item variants, CSS-hidden content, and case-sensitive URL contracts need to behave like browser accessibility state without forcing agents back to raw DOM.

2026-05-13 fixture-contract read: current competitor comparisons keep rewarding tools that make action state reusable in normal app code, not only inside the browser engine. Browser Use, Stagehand, Playwright MCP, Browserbase, and Firecrawl all compete on whether agents can trust compact targets after the first observation. Plasmate's broad repo surface should turn that pressure into a release habit: every newly supported browser semantic needs a shared fixture or manifest entry before it is treated as a durable product contract.

2026-05-13 role-fallback read: fresh official-doc review keeps pointing to the same retention layer. Playwright MCP snapshots expose current refs from accessibility state, Stagehand documents `observe()` as a planning and caching surface, Firecrawl Interact combines scrape sessions with prompt/code interaction and persistent profiles, Browser Use Cloud sells CDP browser sessions with profile state, and Crawl4AI is expanding LLM-friendly crawling toward cloud scale. Plasmate should continue avoiding a hosted-browser pivot and instead make the local SOM compiler behave more like browser accessibility state on messy production markup, including fallback ARIA role tokens and hidden-state variants.

2026-05-13 control-state read: official docs continue to make action menus the durable product layer. Playwright MCP refs are useful only when they match current accessibility state, Stagehand's `observe()` cache is valuable only when reused actions still reflect page state, and managed browser platforms sell session continuity around the same problem. Plasmate's local-first answer is to make compact action targets carry enough live control state (`value`, `checked`, and ARIA checked state) for agents to reuse cached plans without re-reading raw DOM.

2026-05-13 ARIA state-cues read: the newest competitor docs sharpen the same lesson. Playwright MCP snapshots require agents to act from current page state, Stagehand v3 documents local and Browserbase action caches that must validate before reuse, Cloudflare Browser Run/WebMCP pushes typed website actions, and hosted browser products keep selling persistent sessions and traces. Plasmate should stay local-first, but compact action plans need ARIA expanded, pressed, and selected state so agents can tell whether menus, toggle buttons, and selectable custom controls already match the intended workflow.

2026-05-13 ARIA relationship read: current docs add one more retention signal. Playwright MCP still binds refs to fresh snapshots, Stagehand and Browserbase reward cached actions that can be validated before reuse, Browser Use Cloud sells profiles/CDP sessions for repeated workflows, Firecrawl Interact keeps browser state alive after scrape, and Cloudflare Browser Run/WebMCP is testing typed page-provided tools. Plasmate should not pivot into hosted execution, but action menus should expose relationship state (`aria-current`, `aria-controls`, and `aria-haspopup`) so agents know which target is already current, what panel a control affects, and whether an action opens a menu, listbox, or dialog.

2026-05-13 validation-constraint read: current browser-agent docs keep making the same distinction: refs and cached actions are only useful when they carry enough live form context to validate before replay. Playwright MCP snapshots are fresh accessibility state, Stagehand `observe()` actions can be cached locally or on Browserbase, and Browser Use/Firecrawl keep selling stateful sessions around repetitive form workflows. Plasmate should keep the local-first wedge by carrying input guidance and validation state in compact action menus: `autocomplete`, `minlength`, `maxlength`, `pattern`, and `aria-invalid` should mean the same thing in Rust, schema, parser packages, SDKs, and adapters.

2026-05-13 input-affordance read: current browser-agent docs keep validating small, browser-like action menus over broad hosted pivots. Playwright MCP refs remain tied to current accessibility snapshots, while Stagehand and Browserbase cached actions are only valuable when the cached target still matches the field's affordances. Plasmate should carry input modality and autocomplete-widget cues (`inputmode`, `enterkeyhint`, `aria-autocomplete`, and `aria-activedescendant`) through the same shared manifest so agents can choose credential data, keyboard submit behavior, and active suggestion state without raw DOM recovery.

2026-05-13 keyboard-affordance read: current Playwright MCP and Stagehand docs keep emphasizing fresh, validated action state before replay, while Browserbase and Browser Use sell observability around repeated workflows. Plasmate's local-first answer should include keyboard and custom-role cues in the same portable action contract: `accesskey`, `aria-keyshortcuts`, and `aria-roledescription` help agents choose and explain reusable targets without falling back to raw DOM or screenshots.

2026-05-14 form-relation read: current official docs continue to reward action menus that can be validated before replay. Playwright MCP refs remain snapshot-scoped, Stagehand `observe()` and action caches depend on matching current page state, and Firecrawl plus Browser Use sell persistent sessions around repeated form work. Plasmate should keep the local-first wedge by preserving form association and error relationships in compact targets: `form`, `list`, and `aria-errormessage` let agents choose the right submit scope, understand datalist suggestions, and explain invalid fields without raw DOM recovery.

2026-05-14 live-region read: current official docs make validation-before-replay even more important. Playwright MCP refs are scoped to the current accessibility snapshot, Stagehand caches only pay off when the observed page state still matches, and Browser Use/CDP sessions keep dynamic app state alive for repeated workflows. Plasmate should expose lightweight ARIA live-region state in compact targets: `aria-busy`, `aria-live`, `aria-atomic`, and `aria-relevant` tell agents whether a control or result region is updating and how status feedback will announce without forcing raw DOM recovery.

2026-05-14 popover-command read: browser action surfaces are expanding beyond ARIA-only state. MDN now documents Popover API invoker relationships and the newer `commandfor`/`command` button attributes, while Chrome positions command buttons as a declarative replacement path for popover-specific controls. Plasmate should carry these native relationships in SOM action menus so agents can tell which button opens, hides, or toggles which panel before replaying a cached local action.

2026-05-14 relationship-context read: official Playwright MCP docs continue to make current structured snapshots and refs the agent interaction unit, while Stagehand and Browserbase emphasize cached action replay only after validating the target still matches. Plasmate's sticky local-first answer is richer relationship context in compact targets: `aria-owns`, `aria-flowto`, and `aria-details` let agents understand custom widget ownership, guided workflow order, and detailed help panels without pulling raw DOM back into the prompt.

2026-05-14 range-orientation read: current browser-agent competitors keep validating cached actions against fresh state before replay. Range controls, sortable headers, and oriented composite widgets are common SaaS surfaces where an agent needs numeric bounds and current ARIA value state before choosing an action. Plasmate should keep cache keys target-focused while exposing `min`, `max`, `step`, `aria-valuemin`, `aria-valuemax`, `aria-valuenow`, `aria-valuetext`, `aria-orientation`, and `aria-sort` as compact action-plan context.

2026-05-14 widget-affordance read: current browser-agent products keep moving from element identity toward current, validated widget state before replay. ARIA textboxes, listboxes, and custom inputs often expose read-only, multiline, and multiselectable affordances without native HTML equivalents. Plasmate should keep cache keys stable while surfacing `aria-readonly`, `aria-multiline`, and `aria-multiselectable` in compact action targets so agents avoid typing into read-only custom controls and choose the right selection strategy for composite widgets.

2026-05-14 set-position read: current Playwright MCP and Stagehand docs keep making fresh structured state and cache validation the action surface. Tree, menu, and listbox widgets need ordinal context as much as current value state: `aria-level`, `aria-posinset`, and `aria-setsize` tell agents whether a target is nested, where it sits in a collection, and whether a cached navigation plan still points at the expected item. Plasmate should surface these cues without changing deterministic action cache keys.

2026-05-14 text-entry-affordance read: current competitor docs and developer commentary keep validating compact, fresh action menus over full-DOM recovery. Stagehand-style cached actions only stay useful when the field's typing affordances have not drifted, and Playwright MCP-style snapshots make the current accessibility state the selection surface. Plasmate should preserve small but practical text-entry cues such as `spellcheck`, `autocapitalize`, `dirname`, and `aria-placeholder` across the same manifest so agents understand keyboard behavior, language direction capture, and custom textbox prompt text without changing deterministic cache keys.

2026-05-14 upload-affordance read: current browser-agent products keep converging on replayable action menus, but production SaaS workflows often block on file evidence, screenshots, resumes, and media uploads. Plasmate should treat upload controls as first-class local action targets by surfacing `name`, `accept`, `capture`, and native `multiple` state across SDKs and adapters so agents can validate cached upload plans before asking for a file.

2026-05-14 form-submission-context read: Playwright MCP refs remain scoped to the current snapshot, while Stagehand and Browserbase make cached action replay depend on validating that the current target still matches the stored plan. For SaaS workflows, the target is often the whole submission contract, not only one input. Plasmate should surface form-level `action`, `method`, `target`, `enctype`, `novalidate`, `accept-charset`, and `autocomplete` as compact action-plan context so agents can distinguish upload, checkout, and settings forms before reusing local action memory.

2026-05-14 submitter-override read: current browser-agent docs keep validating action replay against fresh structured state, and SaaS pages often route different submit buttons from the same form to different endpoints, methods, targets, or validation modes. Plasmate should expose native submitter override cues (`button_type`, `formaction`, `formmethod`, `formenctype`, `formtarget`, and `formnovalidate`) across the shared action manifest so local cached submit actions can verify the exact submission path without raw DOM recovery.

2026-05-14 ARIA action-role read: current Playwright MCP docs keep structured accessibility refs as the interaction surface, while Stagehand v3 documents `observe()` as a way to discover, validate, and cache executable actions. Production SaaS apps often expose controls through ARIA-only roles instead of native elements, so Plasmate should continue widening local role parity before hosted infrastructure work. `slider`, `spinbutton`, and `option` are small but sticky action-role gaps because agents need to adjust numeric settings and choose custom listbox options without falling back to raw DOM recovery.

2026-05-14 inert-availability read: current action-replay products keep validating whether a target is safe before reuse. Playwright MCP refs are snapshot-scoped, Stagehand/Browserbase cache actions only when page state still matches, and Browser Run/WebMCP is making typed interaction contracts more prominent. Plasmate should keep inert targets visible for planning while marking them unavailable with `blocked_reason="inert"` in compact action menus.

2026-05-14 image submitter read: current Playwright MCP, Stagehand, and Browser Run docs keep pushing browser-agent work toward fresh structured snapshots plus validated replay. Plasmate should keep the local HTML action contract exact: graphical submit inputs should compile as clickable submitters with `button_type`, `alt` labels, and `src` context so cached plans can recognize icon-only form buttons without raw DOM recovery.

2026-05-14 hidden-descendant text read: current browser-agent products make fresh structured state the action source, while Stagehand/Browserbase action caching only works when observed targets still match the current page. Plasmate should filter hidden descendants from visible parent text, labels, select options, lists, and table cells so compact SOM evidence matches what users and accessibility snapshots expose.

2026-05-14 select-option state read: current Playwright MCP docs still bind actions to fresh structured snapshots, Browserbase/Stagehand now emphasizes validated action caching, and Browser Run/WebMCP is expanding hosted browser interaction contracts. Plasmate should keep the local-first wedge by making ordinary select menus browser-accurate in SOM: default option values, disabled options, optgroup labels, and multi-select state let cached local plans validate menu choices without raw DOM recovery.

2026-05-14 select parity read: the sticky product promise is not just that Rust emits better select state; every public contract has to accept and reuse it. Single-select default values, disabled optgroup inheritance, select `size`, option `group`, option `disabled`, and `selected_values` should move through schema, parser packages, SDKs, and prompt renderers so cached menu plans remain portable across the project's broad integration surface.

2026-05-15 HTML-id parity read: current competitor docs keep making structured refs and validated cached actions the reusable browser-agent surface. Playwright MCP refs are snapshot-scoped, Stagehand `observe()` returns cacheable action objects, and Browser Run/WebMCP is exploring browser-native tool contracts. Plasmate should keep `html_id` as portable local DOM provenance across parser packages and SDKs so agents can resolve from compact SOM targets back to live elements without raw DOM recovery or cloud selector memory.

## Ecosystem Surface

The project already spans a large number of package and integration surfaces: Rust CLI/daemon/MCP/CDP/AWP core, Python SDK, Node SDK, Go SDK, LangChain, Browser Use, Vercel AI, SOM parser packages for Python and Node, plugin examples, smoke tests, generated docs, comparison pages, and marketing assets. This breadth is a distribution advantage only if contracts stay synchronized. Short-term roadmap work should favor conformance fixtures, shared schema tests, and adapter docs over one-off integration logic.

## Requirements

1. Preserve actionable structure: SOM must capture common accessibility roles, stable ids, labels, forms, links, state, and selectors that agents can reuse.
2. Reduce repeated-work cost: SOM cache, SOM diff, and selector-aware cache entries should make repeat visits cheaper than first visits.
3. Improve inspectability: expose traces, coverage scorecards, and reproducible fixtures so teams can trust extraction behavior.
4. Keep ecosystem adapters thin: SDKs and integrations should share conformance expectations instead of forking extraction logic.
5. Stay local-first by default: hosted competitors can own scale infrastructure; Plasmate should own local speed, privacy, and open protocol fit.

## Current Run Changes

- 2026-05-15:
  - Python and Node parser packages now parse and expose `html_id`, add original-HTML-id lookup helpers, and include `html_id` in compact action plans without changing deterministic `cache_key` values.
  - Python, Node, and Go SDKs now preserve `html_id`, expose lookup helpers, and carry DOM provenance into action-plan items for live-page resolution.
  - Browser Use, LangChain, and Vercel AI action-plan renderers now surface `html_id` so framework prompts keep live-DOM provenance alongside cache identity.
  - The shared action-availability manifest now asserts `html_id` parity across parser packages, SDKs, and framework adapters.
- 2026-05-14:
  - Rust SOM select extraction now follows browser default option-value semantics when an `<option>` omits `value`.
  - Select option summaries now preserve disabled option state and optgroup labels, giving agents enough context to avoid unavailable choices and explain grouped menus.
  - Multi-select controls now expose `attrs.selected_values` alongside the existing first selected `value`, so cached action plans can validate multiple current choices.
  - Single-select controls without explicit `selected` markup now mark the browser-default first option as selected and expose that value.
  - Disabled `<optgroup>` elements now propagate `disabled=true` to child option summaries, and explicit select `size` is preserved.
  - JSON Schema/SOM spec, Python/Node parser packages, Python/Node/Go SDKs, Browser Use, LangChain, and Vercel AI now accept or render `selected_values`, select `size`, option `disabled`, and option `group`.
  - The Rust SOM compiler and JSON Schema now preserve native range/value constraints (`min`, `max`, and `step`) plus ARIA range, orientation, and sort cues (`aria-valuemin`, `aria-valuemax`, `aria-valuenow`, `aria-valuetext`, `aria-orientation`, and `aria-sort`).
  - Parser packages, SDKs, Browser Use, LangChain, and Vercel AI action-plan surfaces now expose `min`, `max`, `step`, `orientation`, `sort`, `valuemin`, `valuemax`, `valuenow`, and `valuetext` without changing deterministic action `cache_key` values.
  - The shared action-availability manifest now asserts range and ARIA orientation/value cues across parser, SDK, and framework outputs.
  - The Rust SOM compiler and JSON Schema now preserve ARIA widget affordance cues: `aria-readonly`, `aria-multiline`, and `aria-multiselectable`.
  - Parser packages, SDKs, Browser Use, LangChain, and Vercel AI action-plan surfaces now expose `readonly`, `multiline`, and `multiselectable`; ARIA read-only targets are marked unavailable with `blocked_reason="readonly"` without changing deterministic action `cache_key` values.
  - The shared action-availability manifest now asserts ARIA read-only gating plus multiline and multiselectable widget cues across parser, SDK, and framework outputs.
  - The Rust SOM compiler and JSON Schema now preserve ARIA set-position cues: `aria-level`, `aria-posinset`, and `aria-setsize`.
  - Parser packages, SDKs, Browser Use, LangChain, and Vercel AI action-plan surfaces now expose `level`, `posinset`, and `setsize` without changing deterministic action `cache_key` values.
  - The shared action-availability manifest now asserts ARIA set-position cues across parser, SDK, and framework outputs.
  - The Rust SOM compiler and JSON Schema now preserve text-entry affordance cues: native `spellcheck`, `autocapitalize`, `dirname`, and ARIA `aria-placeholder`.
  - Parser packages, SDKs, Browser Use, LangChain, and Vercel AI action-plan surfaces now expose `spellcheck`, `autocapitalize`, `dirname`, and `aria_placeholder` without changing deterministic action `cache_key` values.
  - The shared action-availability manifest and `016-action-semantics` conformance fixture now assert these text-entry affordance cues across Rust, parser, SDK, and framework outputs.
  - The Rust SOM compiler and JSON Schema now preserve upload action cues: native `accept`, `capture`, and input `multiple`, while the shared manifest now includes field `name` identity for deterministic target caching.
  - Parser packages, SDKs, Browser Use, LangChain, and Vercel AI action-plan surfaces now expose `name`, `accept`, `capture`, and `multiple` for upload and multi-select workflows.
  - The shared action-availability manifest now asserts upload constraints and native multiple-selection state across parser, SDK, and framework outputs.
  - The Rust SOM compiler and JSON Schema now preserve form submission context: `target`, `enctype`, `novalidate`, `accept-charset`, and form-level `autocomplete`, alongside existing `action` and `method`.
  - Parser packages, SDKs, Browser Use, LangChain, and Vercel AI action-plan surfaces now expose `form_action`, `form_method`, `form_target`, `form_enctype`, `form_novalidate`, `form_accept_charset`, and `form_autocomplete` without changing deterministic action `cache_key` values.
  - The shared action-availability manifest now asserts form submission context across parser, SDK, and framework outputs.
  - The Rust SOM compiler now preserves submit-button override cues, and JSON Schema/SOM spec docs accept them: `button_type`, `formaction`, `formmethod`, `formenctype`, `formtarget`, and `formnovalidate`.
  - Parser packages, SDKs, Browser Use, LangChain, and Vercel AI action-plan surfaces now expose submit override cues so cached submit actions can validate endpoint, method, encoding, target, and validation mode before replay.
  - The shared action-availability manifest now asserts submit-button override context across parser, SDK, and framework outputs.
  - The Rust SOM compiler now maps ARIA `slider` and `spinbutton` roles to actionable `text_input` targets and maps ARIA `option` to an actionable `button` target.
  - The `016-action-semantics` conformance fixture now asserts `slider`, `spinbutton`, and `option` action-role coverage, including current ARIA value and selected state.
  - SOM spec and generated docs now document the expanded ARIA action-role mapping so SDK and adapter maintainers know these roles are product surface.
  - The Rust SOM compiler and JSON Schema now preserve native `inert` state, including inherited inert context for nested interactive controls.
  - Parser packages, SDKs, Browser Use, LangChain, and Vercel AI action-plan surfaces now expose `inert`, mark inert targets unavailable with `blocked_reason="inert"`, and keep deterministic action `cache_key` values target-focused.
  - The `015-action-state` conformance fixture and shared action-availability manifest now assert inert availability gating across Rust, parser, SDK, and framework surfaces.
  - The Rust SOM compiler now maps `input type="image"` to an actionable button, adds `button_type` to input-backed submitters, resolves graphical submitter labels from `alt`, and preserves `alt`/`src` context.
  - The Rust SOM compiler now filters stylesheet-hidden descendants from parent text, interactive names, accessible label indexing, select options, list items, table captions, and table cells.
  - The Rust SOM compiler and JSON Schema now preserve ARIA relationship cues: `aria-owns`, `aria-flowto`, and `aria-details`.
  - Parser packages, SDKs, Browser Use, LangChain, and Vercel AI action-plan surfaces now expose `owns`, `flowto`, and `details` without changing deterministic action `cache_key` values.
  - The shared action-availability manifest now asserts ARIA owns/flowto/details relationship cues across parser, SDK, and framework outputs.
  - The Rust SOM compiler and JSON Schema now preserve native popover and command relationships: `popovertarget`, `popovertargetaction`, `commandfor`, `command`, and `popover`.
  - Parser packages, SDKs, Browser Use, LangChain, and Vercel AI action-plan surfaces now expose `popovertarget`, `popovertargetaction`, `commandfor`, and `command` without changing deterministic action `cache_key` values.
  - The shared action-availability manifest now asserts popover/command relationship cues across parser, SDK, and framework outputs.
  - The Rust SOM compiler and JSON Schema now preserve ARIA live-region cues: `aria-busy`, `aria-live`, `aria-atomic`, and `aria-relevant`.
  - Parser packages, SDKs, Browser Use, LangChain, and Vercel AI action-plan surfaces now expose `busy`, `live`, `atomic`, and `relevant` without changing deterministic action `cache_key` values.
  - The shared action-availability manifest now asserts live-region state across parser, SDK, and framework outputs.
  - The Rust SOM compiler and JSON Schema now preserve `form`, `list`, and `aria-errormessage`, adding form ownership, datalist, and error-message relationships to the compact action-state contract.
  - Parser packages, SDKs, Browser Use, LangChain, and Vercel AI action-plan surfaces now expose `form`, `list`, and `errormessage` without changing deterministic action `cache_key` values.
  - The shared action-availability manifest now asserts form-relation and error-message cues across parser, SDK, and framework outputs.
  - The Rust SOM compiler and JSON Schema now preserve native `accesskey` plus ARIA `keyshortcuts` and `roledescription`, adding keyboard/custom-role affordances to the compact action-state contract.
  - Parser packages, SDKs, Browser Use, LangChain, and Vercel AI action-plan surfaces now expose `accesskey`, `keyshortcuts`, and `roledescription` without changing deterministic action `cache_key` values.
  - The shared action-availability manifest now asserts keyboard and custom-role cues across parser, SDK, and framework outputs.
  - The Rust SOM compiler and JSON Schema now preserve `inputmode`, `enterkeyhint`, `aria-autocomplete`, and `aria-activedescendant`, extending validation-state work into input-affordance cues for cached form actions.
  - Parser packages, SDKs, Browser Use, LangChain, and Vercel AI action-plan surfaces now expose `inputmode`, `enterkeyhint`, `aria_autocomplete`, and `active_descendant` without changing deterministic action `cache_key` values.
  - The shared action-availability manifest now asserts input modality and autocomplete-widget state across parser, SDK, and framework outputs.
  - The Rust SOM compiler and JSON Schema now preserve form-entry constraints (`minlength`, `maxlength`, `pattern`) plus `aria-invalid`, extending current action-state fidelity into validation state.
  - Parser packages, SDKs, Browser Use, LangChain, and Vercel AI action-plan surfaces now expose `autocomplete`, length constraints, `pattern`, and `invalid` without changing deterministic `cache_key` values.
  - The shared action-availability manifest now asserts validation constraints and invalid state across parser, SDK, and framework outputs.
  - The Rust SOM compiler and JSON Schema now preserve `aria-controls` and `aria-haspopup` in `attrs.aria`, joining existing `aria-current` support for browser-like action relationship state.
  - Parser packages, SDKs, Browser Use, LangChain, and Vercel AI action-plan helpers now expose `current`, `controls`, and `haspopup` cues without changing deterministic `cache_key` generation.
  - The shared action-availability manifest now asserts current-page links, controlled popup targets, and popup type cues across parser, SDK, and framework surfaces.
  - Parser packages, SDKs, Browser Use, LangChain, and Vercel AI action-plan helpers now expose ARIA `expanded`, `pressed`, and `selected` cues while keeping deterministic `cache_key` generation target-focused.
  - Prompt renderers now include expanded/pressed/selected state alongside value and checked state so agents can reuse cached action menus without raw DOM recovery.
  - The shared action-availability manifest now asserts ARIA expanded, pressed, and selected state across parser, SDK, and framework surfaces.
  - Parser packages, Python/Node/Go SDKs, Browser Use, LangChain, and Vercel AI action-plan helpers now preserve current control `value` fields for interactive targets without changing deterministic `cache_key` values.
  - Compact action targets now expose `checked` state from native `attrs.checked` and ARIA `aria.checked`, covering both checkbox/radio inputs and custom menu widgets.
  - The shared action-availability manifest now asserts value and checked state so framework prompt renderers cannot drift from parser/SDK action plans.
  - ARIA landmark role parsing now honors space-separated fallback tokens, so `role="utility search"` still compiles into a labelled search/navigation region instead of falling back to generic content.
  - ARIA widget role parsing now honors fallback tokens, preserving `menuitemcheckbox` and `menuitemradio` action targets when production markup includes unknown role tokens before the known role.
  - Hidden-element stripping now treats uppercase `aria-hidden="TRUE"` and inline `opacity: 0` as hidden state, aligning inline visibility handling with the stylesheet visibility parser.
  - The `016-action-semantics` conformance fixture now covers role fallback tokens, uppercase ARIA-hidden state, and inline opacity hiding alongside search landmarks, menu action targets, and stylesheet whitespace/casing.
  - Shared action-availability fixtures now include ARIA menu checkbox and radio targets so Browser Use, LangChain, Vercel AI, parser packages, and SDKs prove the same menu action contract.
  - Added `specs/conformance/016-action-semantics.html` and expected output to cover labelled search landmarks, menuitem checkbox/radio targets, and stylesheet hidden-rule whitespace/casing in one reusable fixture.
  - Added Rust compiler regression coverage for the new action-semantics fixture so the shared fixture cannot drift as documentation-only coverage.
  - ARIA `role="search"` now compiles into a labelled navigation region, preserving search landmarks that agents commonly use before selecting a query field.
  - ARIA `menuitemcheckbox` and `menuitemradio` now map to actionable checkbox and radio SOM roles, improving custom menu parity with browser accessibility snapshots.
  - Stylesheet visibility parsing now ignores all declaration whitespace and casing, so `DISPLAY\t:\nnone` and similar CMS output is stripped like simpler `display:none` rules.
  - A stale integration test now matches the case-sensitive URL path contract: `/Page`, `/page`, and `/PAGE` remain distinct while fragments and trailing duplicate slashes are still deduped.
  - Added `integrations/fixtures/action-availability.expected.json` as the shared expected compact action-target contract for the action availability SOM fixture.
  - Added `scripts/action-manifest-conformance.sh` to run Browser Use, LangChain, Vercel AI, parser-package, and SDK fixture checks from one release command.
  - Added quick/full modes to the action-manifest conformance script so CI can run focused shared-manifest checks while local releases can run the full gate.
  - Added a GitHub Actions action-manifest job that installs Python, Node, and Go dependencies and runs the quick gate on pushes and pull requests.
  - Fixture docs now explain quick vs full action-manifest usage and when maintainers should run each gate.
  - Node SDK `npm test` now builds and runs the action-plan fixture tests against the shared manifest.
  - Root and fixture docs now point maintainers at the shared action-manifest release gate.
  - Python SDK query helpers now expose `get_action_plan()` and `get_action_plan_cache_key()`.
  - Node SDK query helpers now expose `getActionPlan()` and `getActionPlanCacheKey()`.
  - Python parser, Node parser, Go SDK, Python SDK, and Node SDK tests now consume the shared action availability manifest.
  - Browser Use adapter tests now validate rendered page context against the shared expectation manifest instead of hard-coded local cache-key and availability assertions.
  - LangChain adapter tests now validate SOM text output against the same expectation manifest, keeping text-only prompts aligned with Browser Use and Vercel AI.
  - Vercel AI runtime fixture tests now compare extracted action targets with the shared manifest and verify cache-key uniqueness across the fixture.
  - Added integration fixture documentation so future adapter updates know to update SOM fixtures and expected action contracts together.
  - Go SDK action plans now include deterministic `CacheKey` values and `GetActionPlanCacheKey()`.
  - Browser Use page contexts now render action-plan `cache_key` flags beside availability state.
  - LangChain SOM text now computes and renders deterministic `cache_key` flags for interactive targets.
  - Added focused Go, Browser Use, and LangChain fixture coverage for action cache keys.
  - Vercel AI compact action targets now include deterministic `cache_key` values for cached menus and trace logs.
  - Node SOM parser action plans now include `cache_key` and export `getActionPlanCacheKey()`.
  - Python SOM parser action plans now include `cache_key` and export `get_action_plan_cache_key()`.
  - Added focused Vercel AI, Node parser, and Python parser coverage for deterministic action cache keys.
  - Vercel AI SDK integration now exports `extractPlasmateActionTargets()` for deriving compact action targets directly from raw SOM responses, including nested children and shadow-root elements.
  - Vercel AI prompt formatting now includes blocked reasons, input type, and placeholder metadata so cached action menus carry parser-equivalent field-selection cues.
  - Added an executable Vercel AI fixture test that builds the package and validates extraction, availability filtering, and prompt formatting against the shared adapter SOM fixture.
  - Vercel AI action-target availability now treats any `blocked_reason` as unavailable, not only `blocked_reason="disabled"`.
  - Vercel AI SDK integration now exports `normalizePlasmateActionTarget()`, `preparePlasmateActionPlan()`, and `formatPlasmateActionPlan()` for action-menu preparation before Vercel AI SDK calls.
  - Added fixture-style TypeScript compile coverage for Vercel AI action helpers using availability, required, group, and description metadata.
  - Added a shared adapter fixture for action availability, required fields, groups, input type, and descriptions.
  - Browser Use and LangChain adapter tests now consume the same fixture, reducing drift between framework context output and parser action plans.
  - LangChain now marks normal interactive targets as `[enabled]` when SOM omits `attrs.disabled`, and includes `[blocked_reason=disabled]` for disabled targets.
  - Vercel AI SDK integration now exports `PlasmateActionTarget` and `isPlasmateActionTargetAvailable()` so apps can filter cached action menus before prompting.
  - Browser Use and LangChain package `__version__` exports now match their `pyproject.toml` versions.
  - Browser Use integration page contexts now render compact action-plan targets with `enabled`, disabled `blocked_reason`, required, type, group, and description context.
  - Browser Use integration now exposes sync and async `extract_action_plan` helpers so agents can ask directly for reusable SOM action targets.
  - LangChain SOM text output now marks disabled, enabled, required, group, and description state on interactive elements before click/type planning.
  - Vercel AI SDK integration now exports `plasmateActionGuidance`, a concise system-prompt helper that tells agents to honor SOM availability fields.
  - Added focused adapter tests for Browser Use and LangChain availability rendering.
  - Python SOM parser action plans now include `enabled` and `blocked_reason`, so agents can skip disabled targets without re-walking attrs.
  - Node SOM parser action plans now expose the same availability contract in `ActionPlanItem`.
  - Go SDK action plans now expose `Enabled` and `BlockedReason`, keeping durable worker services aligned with Python and Node planners.
  - Parser and Go tests now cover disabled action-plan targets.
  - Disabled native `<fieldset>` state now propagates to descendant native controls.
  - Added shared conformance fixture `015-action-state` for disabled fieldset inheritance plus ARIA required/disabled promotion.
  - Updated the conformance index so adapter maintainers can promote action-state checks into release tests.
  - Native `<textarea disabled>` controls now preserve `attrs.disabled`.
  - Native `<select disabled>` controls now preserve `attrs.disabled`.
  - ARIA widgets with `aria-required="true"` now promote `attrs.required` for action-plan parity with native controls.
  - ARIA widgets with `aria-disabled="true"` now promote `attrs.disabled` while preserving nested `attrs.aria.disabled`.
  - Added focused compiler coverage for disabled textarea/select controls and ARIA required/disabled custom controls.
- 2026-05-12:
  - ARIA landmark role parsing is now case-insensitive for uppercase production markup.
  - Declarative shadow DOM extraction now recurses through wrapper containers so nested web-component controls survive.
  - Python/Node parser and Go SDK action-plan helpers now include placeholder, description, required, disabled, and group metadata.
  - Rust SOM compilation now emits labelled `group` elements for native `<fieldset>` controls and ARIA `group`/`radiogroup` widgets.
  - Fieldset groups now derive labels and `attrs.legend` from the first `<legend>`, and preserve `attrs.disabled` for disabled fieldsets.
  - CDP accessibility and DOM mappings now understand SOM `group` roles.
  - SOM schema, spec, Python/Node SDK types, Python/Node parser types, and Go SDK attrs now accept `group` roles and `attrs.legend`.
  - Parser and Go tests now cover group/legend payload compatibility.
  - Added a shared conformance fixture for fieldset/legend and ARIA radiogroup semantics.
- 2026-05-11:
  - Rust SOM compilation now resolves nested `<label>` controls, including wrapped checkboxes and selects, without leaking option text into labels.
  - Landmark and form region labels now resolve `aria-labelledby`, aligning region naming with browser accessibility snapshots.
  - Input buttons now expose `value` as their accessible label and retain normalized `attrs.input_type` for `submit`, `button`, and `reset` controls.
  - Go SDK types now parse current SOM fields for `shadow`, accessible descriptions, `name`, `autocomplete`, ARIA state, details, and iframe attrs.
  - Go query helpers now traverse shadow-root elements for id, role, text, interactivity, and flattened element queries.
  - Go now exposes `FindByAction`, `FindByHint`, and `GetActionPlan` so action planning is available across Rust output, Python/Node parser packages, and Go consumers.
  - SOM metadata now counts elements and interactive controls inside shadow roots.
  - `aria-labelledby` now takes precedence over `aria-label` when resolving element labels.
  - SOM attrs now include accessible descriptions from `aria-describedby` and `aria-description`.
  - Schema and Python/Node types now accept `attrs.description`.
- 2026-05-10:
  - Rust SOM compilation resolves `aria-labelledby` and external `<label for="...">` text for interactive controls.
  - Python `from_plasmate()` extracts SOM JSON from mixed CLI output with progress/log lines and wrapped `{ "som": ... }` responses.
  - Node `fromPlasmate()` now accepts wrapped `{ som: ... }` payloads in clean and mixed CLI output, matching Python behavior.
- 2026-05-07:
  - SOM JSON Schema now accepts current Rust core output for `shadow`, `iframe`, `details`, ARIA state, and actionability attrs.
  - Python and Node parser packages now expose shadow-root types and traverse `shadow.elements` in element, text, link, and interactive queries.
  - Python and Node SDK query helpers now find shadow-root elements by id, role, text, and actionability.
- 2026-05-09:
  - Python and Node parser packages now expose action and hint lookup helpers so agents can query `click`, `type`, `required`, and similar SOM metadata directly.
  - Python and Node parser packages now expose compact action-plan helpers with element ids, roles, actions, labels, hrefs, names, and input types for Stagehand-style planning without re-walking the full SOM.
  - Node parser compression-ratio handling now matches Python by returning infinity when `som_bytes` is zero instead of reporting a misleading zero.
- 2026-05-06:
  - SOM link deduplication now preserves case-sensitive URL paths while still stripping fragments and duplicate trailing slashes.
  - Input type and ARIA role parsing is more tolerant of real-world casing, so `type="SUBMIT"` and upper-case custom roles no longer lose actionability.
  - Custom controls now retain `contenteditable`, `tabindex`, `name`, and `autocomplete` attributes in SOM attrs, improving parity with accessibility-snapshot competitors.
  - MCP `extract_text` truncation is UTF-8 safe, preventing panics when `max_chars` cuts through multibyte content.
- 2026-05-05:
  - Cache prefetch URL extraction now walks nested SOM children and shadow-root elements, deduplicates URLs while preserving order, and excludes non-HTTP schemes.
  - Cache URL normalization now lowercases scheme/host through URL parsing without corrupting case-sensitive paths.
  - MCP `extract_text` and `extract_links` now include shadow-root content, so declarative web components are not invisible to agents.
- 2026-05-04:
  - Added region-id selector support while keeping HTML id selection.
  - Added common ARIA widget role mapping into actionable SOM elements.
  - Hardened inline hidden-style stripping against spacing and casing variants.
  - Updated roadmap direction around cached structured actions, MCP distribution, and accessibility/SOM parity.

## Next Steps

- Implement selector-aware SOM cache entries for `main`, `form`, and `#id` prompts.
- Add trace export for MCP/AWP sessions so users can debug why an agent clicked or selected an element.
- Add conformance cases for ARIA-heavy SaaS pages, especially disabled and required custom controls, and compare output against Playwright MCP snapshots.
- Wire `015-action-state` into cross-adapter parser/SDK conformance runners so inherited disabled state stays synchronized outside Rust.
- Promote the GitHub Actions action-manifest job from quick shared-manifest checks to full conformance once runtime and dependency caching are stable.
- Add dependency-cache tuning for the action-manifest job so cross-runtime conformance stays cheap enough to keep required.
- Wire `016-action-semantics` into parser/SDK and adapter conformance runners so search landmarks, fallback-token ARIA roles, menu roles, ARIA-hidden casing, and visibility-hidden variants stay synchronized outside Rust.
- Promote ARIA relationship-state cases (`aria-controls`, `aria-haspopup`, `aria-owns`, `aria-flowto`, and `aria-details`) from the shared action availability manifest into the broader `015-action-state`/`016-action-semantics` conformance suites.
- Promote range and orientation cases (`min`, `max`, `step`, `aria-orientation`, `aria-sort`, and ARIA value state) into broader Rust/parser/SDK and adapter conformance fixtures.
- Promote ARIA widget affordance cases (`aria-readonly`, `aria-multiline`, and `aria-multiselectable`) into broader Rust/parser/SDK and adapter conformance fixtures.
- Add compiler/schema conformance for form validation constraints and `aria-invalid`, then promote the shared manifest cases into broader parser, SDK, and adapter fixtures.
- Promote input-affordance cases (`inputmode`, `enterkeyhint`, autocomplete widget state, active descendants, `spellcheck`, `autocapitalize`, `dirname`, and `aria-placeholder`) into broader parser, SDK, and adapter conformance fixtures once the shared action manifest remains stable.
- Promote upload-affordance cases (`accept`, `capture`, `multiple`, and stable field `name`) into broader Rust/parser/SDK and adapter conformance fixtures.
- Promote form-submission context cases (`form_action`, `form_method`, `form_target`, `form_enctype`, `form_novalidate`, `form_accept_charset`, and `form_autocomplete`) into broader Rust/parser/SDK and adapter conformance fixtures.
- Promote submit-button override cases (`button_type`, `formaction`, `formmethod`, `formenctype`, `formtarget`, and `formnovalidate`) into broader Rust/parser/SDK and adapter conformance fixtures.
- Promote graphical submitter cases (`input type="image"`, `button_type`, `alt`, and `src`) into the shared action manifest and adapter conformance fixtures.
- Promote `html_id` DOM-provenance cases into shared parser, SDK, and adapter conformance so compact targets remain resolvable back to live page elements.
- Promote keyboard-affordance cases (`accesskey`, `aria-keyshortcuts`, and `aria-roledescription`) into broader Rust/parser/SDK conformance fixtures once the shared action manifest remains stable.
- Promote form-relation cases (`form`, `list`, and `aria-errormessage`) into broader parser, SDK, and adapter conformance fixtures.
- Promote live-region cases (`aria-busy`, `aria-live`, `aria-atomic`, and `aria-relevant`) into broader Rust/parser/SDK conformance fixtures.
- Promote fieldset/legend group semantics into shared conformance fixtures alongside cross-adapter accessible-description cases.
- Add shared conformance for nested shadow-root controls and enriched action-plan metadata.
- Promote the new SDK/parser shadow-root and Go action-plan tests into shared conformance fixtures that run against every adapter before release.
- Audit ecosystem repos for stale install docs, tool counts, and schema drift.
- Promote action-plan helper parity into framework integrations so every adapter exposes the same compact action target contract.
