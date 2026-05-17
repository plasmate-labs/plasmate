# Roadmap

Plasmate's roadmap is public and standards-first. We ship compression and correctness before scale.

## 2026 Market Adjustment

Browser-agent infrastructure is converging on structured context instead of raw page dumps. Playwright MCP has normalized accessibility snapshots, Firecrawl is packaging search/scrape/browser sessions behind MCP, and Browserbase/Stagehand is pushing cached actions to reduce repeated LLM calls.

Plasmate should keep its local-first position, but the roadmap now emphasizes three sticky advantages:

- **Actionable SOM snapshots**: selectors, ARIA widget parity, and stable ids are product features agents depend on.
- **Cheaper repeated workflows**: SOM cache and diff should become the local, page-level answer to cloud selector/action caching.
- **Ecosystem distribution**: MCP, Browser Use, SDKs, and comparison pages should remain conformance-tested so partner repos do not drift.

Near-term target: make Plasmate the fastest local way to turn authenticated or repetitive web workflows into compact, inspectable, reusable state.

### 2026-05-17 Label Collision and Replay Safety Adjustment

Current browser-agent products are teaching users to choose actions by readable names while validating replay against fresh page state. Playwright MCP keeps refs scoped to the current snapshot, Stagehand/Browserbase caches selectors only after validation, Firecrawl keeps distributing MCP/API browser workflows, and Cloudflare Browser Run is widening CDP/MCP compatibility. Plasmate should keep labels as assisted-recovery signals while making ambiguity explicit:

- **Label collisions must be inspectable**: Node, Python, and Go action-plan indexes should keep all exact matches in `byLabelAll` / `by_label_all` instead of exposing only the first label match.
- **Unique-label replay should fail closed**: helper APIs should return a target only when an exact label maps to one compact action target.
- **Stable ids remain the unattended contract**: docs, MCP, CDP, and adapters should keep steering replay caches toward SOM ids, deterministic cache keys, HTML ids, and test ids, with labels reserved for debugging and recovery.

### 2026-05-17 CDP Attribute-Selector Adjustment

Current browser-agent infrastructure keeps turning protocol compatibility into distribution. Browserbase/Stagehand highlights observe/action primitives, cached actions, session replay, and local-to-cloud portability; Cloudflare Browser Rendering added CDP endpoints and MCP client support; Firecrawl keeps exposing MCP scrape/search/extract plus browser interaction. Plasmate should keep the local-first wedge and improve the compatibility path where ordinary CDP clients already start: selectors and node attributes.

- **Common selectors should hit SOM targets**: `DOM.querySelector` and `DOM.querySelectorAll` should support tag-qualified and bare attribute selectors for `name`, `href`, `type`, `aria-label`, `aria-labelledby`, `aria-describedby`, `role`, test ids, and boolean availability attrs.
- **Human labels should be forgiving**: text and label selector matching should be case-insensitive so agents can reuse natural-language targets without exact casing.
- **DOM nodes should expose replay cues**: CDP DOM nodes should include `data-plasmate-id`, `data-som-role`, HTML id, test id, ARIA label, href/name/type, and disabled/readonly/required flags when the SOM has them.

### 2026-05-17 CDP DOM/AX Selector Parity Adjustment

Current browser-agent tools are turning structured page state into a protocol-level adoption path. Playwright MCP uses accessibility snapshots and snapshot-scoped refs, Stagehand/Browserbase caches observed actions and selectors for repeated workflows, Firecrawl distributes scrape/search/extract through MCP, and Cloudflare Browser Run now positions CDP, MCP, recordings, and structured-data endpoints as hosted browser-session features. Plasmate should keep the local-first wedge and make existing CDP clients resolve SOM-backed targets without raw DOM recovery:

- **Replay selectors should work through CDP DOM**: `DOM.querySelector` and `DOM.querySelectorAll` should resolve `#html_id`, `#som_id`, test-id selectors, SOM role names, text, and labels against the SOM-backed node tree in document order.
- **AX trees should match SOM reachability**: `Accessibility.getFullAXTree` should include nested children and shadow-root elements, not just top-level region children.
- **Availability belongs in compatibility output**: AX nodes should carry backend node ids and disabled/readonly properties so Puppeteer, Playwright, and MCP bridge clients can validate cached targets before replay.

### 2026-05-17 CDP Action-Menu Parity Adjustment

Current browser-agent tools increasingly expose action state at the protocol edge, not only in SDK helpers. Playwright MCP uses fresh accessibility snapshot refs for interaction, Stagehand/Browserbase caches validated actions to remove repeat LLM calls, Firecrawl exposes browser sandbox sessions through API, CLI, SDK, and MCP surfaces, and Cloudflare Browser Rendering/Browser Run now supports CDP plus MCP client workflows. Plasmate should keep the local-first SOM wedge and make CDP compatibility more agent-native:

- **Full-tree CDP action menus**: `Plasmate.getInteractiveElements` should traverse nested children and shadow roots, matching MCP/session lookup and parser/SDK action-plan behavior.
- **Protocol filters before prompts**: CDP clients should narrow by SOM role, action, accessible label, and enabled state before handing targets to an LLM.
- **Replay cues on the wire**: CDP output should expose SOM role names, `html_id`, `test_id`, `enabled`, and `blocked_reason` directly so Puppeteer and Playwright users do not need raw DOM recovery for simple cached replay.

### 2026-05-17 CDP Replay-Key Adjustment

Current browser-agent competitors are making reusable action identity a protocol-level expectation. Playwright MCP gives agents snapshot refs, Stagehand/Browserbase caches validated selectors after first observation, Firecrawl distributes browser sessions through MCP/API surfaces, and Cloudflare Browser Run sells CDP/MCP browser infrastructure. Plasmate should answer with a local, deterministic replay surface in its CDP domain:

- **Cache keys should cross the protocol boundary**: CDP action targets should include the same `plasmate-action:v1:*` cache key as Python, Node, Go, and adapter action plans.
- **Replay lookup should be boring**: CDP clients should filter by SOM id, `cache_key`, `html_id`, or `test_id` before resorting to label search or raw DOM recovery.
- **Large pages need paging**: `offset`/`limit` support should let clients inspect or stream action menus without handing every target to an LLM.

### 2026-05-17 SDK Discoverability and Label Parity Adjustment

Current browser-agent competitors keep converging on compact action menus that can be validated before replay. Playwright MCP keeps refs scoped to the current accessibility snapshot, Stagehand/Browserbase makes action caching a repeated workflow feature, Firecrawl exposes scrape/search/extract/browser interaction through MCP and APIs, and Cloudflare Browser Run/WebMCP is expanding hosted browser-native tool surfaces. Plasmate should not pivot into hosted execution; the stickier move is to make local SOM action menus easier to discover and query in every SDK and adapter:

- **Go should match Python/Node label lookup**: durable worker code needs `ByLabel`, exact-label resolution, and label search helpers for debugging and human-facing recovery.
- **Docs should teach pre-prompt scoping**: public SDK and integration pages should show role/action grouping before agents spend tokens on a full SOM.
- **Replay ids stay the default**: labels are lookup hints; unattended replay should continue storing SOM ids, cache keys, HTML ids, or test ids.

### 2026-05-16 Role/Action Grouping Adjustment

Current browser-agent products are making action discovery a reusable app-layer surface. Playwright MCP gives agents current snapshot refs, Stagehand `observe()` produces cacheable action plans, Firecrawl keeps broad MCP/browser session distribution, and Cloudflare Browser Run/WebMCP points toward typed browser-native tools. Plasmate should keep the local-first wedge and make action menus easier to scope in ordinary SDK code:

- **Group before replay**: parser packages and SDKs should expose `by_role`/`by_action` action-plan buckets plus helpers for role/action target lists.
- **Stable ids remain the replay contract**: role/action buckets are for planning and narrowing; unattended replay should still store SOM ids, cache keys, HTML ids, or test ids.
- **Conformance should follow the action surface**: Browser Use, LangChain, Vercel AI, Go, and shared fixtures should adopt the same grouped target contract so the broad repo surface stays one product promise.

### 2026-05-16 Adapter Grouping Adjustment

The grouped action-target contract now needs to live where agent developers actually build workflows: Browser Use, LangChain, Vercel AI, and Go durable workers. Current competitors keep turning browser state into reusable app-layer action menus, so Plasmate's next retention step is cross-adapter parity rather than another hosted-browser feature.

- **Adapters should scope plans directly**: framework helpers should expose role/action groups instead of forcing users to scan compact action lists by hand.
- **Go workers should match orchestration SDKs**: durable services need `ByRole`/`ByAction` buckets and helper functions to reuse the same target-selection logic as Python and Node agents.
- **Release gates should include grouping**: the action-manifest conformance path should prove grouped buckets and enabled-only filtering across adapters before release.

### 2026-05-16 Label-Addressable Action Adjustment

Current browser-agent products keep training users to choose actions by human-facing structured names, then reuse validated targets later. Playwright MCP snapshot refs make accessible names the selection layer, Stagehand caches observed actions after validation, and Browser Run/WebMCP raises the bar for inspectable action state. Plasmate should keep stable identifiers as the default replay contract, but make labels easy to use explicitly:

- **Labels are lookup hints**: parser packages and SDKs should expose element-level label search plus compact action-target label search.
- **Replay remains stable by default**: auto lookup should continue to prefer SOM ids, cache keys, HTML ids, and test ids because labels can duplicate.
- **Docs should teach the distinction**: app developers can use labels for user-facing recovery and debugging while storing stable ids for unattended replay.

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
- **Schema fields need SDK homes**: `attrs.description`, `attrs.name`, `attrs.accept`, `attrs.capture`, `attrs.multiple`, `attrs.autocomplete`, ARIA state, details attrs, iframe attrs, and `shadow` should be treated as public contract across all SDKs.

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

### 2026-05-13 ARIA Fallback and Visibility Adjustment

Official docs and current competitor positioning continue to reward compact, browser-like action surfaces over raw DOM access. Playwright MCP snapshots use fresh accessibility refs, Stagehand `observe()` returns actions that can be cached locally or on Browserbase, Firecrawl Interact resumes scrape sessions for prompt/code actions with profiles, Browser Use Cloud exposes CDP browser sessions with profile state, and Crawl4AI is broadening LLM-friendly crawling toward cloud extraction. Plasmate should keep the local-first wedge and close the small production-markup gaps that force agents back to raw DOM recovery.

- **ARIA roles need fallback-token tolerance**: landmark and widget roles should honor the first known role in a space-separated `role` list.
- **Hidden state should match browser intent**: uppercase ARIA booleans and inline opacity/zero-size hiding should be stripped like equivalent stylesheet rules.
- **Conformance fixtures should absorb semantic polish**: every production tolerance fix should be attached to `016-action-semantics` or another shared fixture before adapter release gates consume it.

### 2026-05-13 Control-State Action Menu Adjustment

The latest docs keep reinforcing that reusable actions are only sticky when state is current. Playwright MCP refs are scoped to fresh accessibility snapshots, Stagehand's `observe()` cache has to validate before acting, and Firecrawl/Browser Use sell browser/session continuity around forms that change between runs. Plasmate should keep the local-first action memory wedge, but compact targets need enough live state to keep cached plans honest.

- **Current values are planning context**: text inputs and selects should surface non-empty `value` fields in action plans and prompt renderers so agents know whether a form is already filled.
- **Checked state must cross custom controls**: native `checked` attrs and ARIA `checked` state should normalize into one compact action-plan field for checkbox, radio, menuitemcheckbox, and menuitemradio targets.
- **Cache keys stay target-focused**: live value/checked state should be visible to agents without changing deterministic target `cache_key` values, preserving local action memory while still exposing state drift.

### 2026-05-13 ARIA State-Cues Adjustment

Current competitor movement keeps raising the value of state-aware action menus. Playwright MCP snapshots are valid only against the current page, Stagehand v3 action caches need local or Browserbase validation before reuse, Browser Run/WebMCP points toward typed page actions, and hosted browser platforms sell traces and persistent sessions around the same drift problem. Plasmate should keep the local-first wedge by making compact SOM targets carry the ARIA state agents need before they choose a cached action.

- **Expanded state prevents stale menu actions**: action plans should surface `aria-expanded` so agents know whether disclosure menus and comboboxes already expose the target content.
- **Pressed state matters for toggle buttons**: `aria-pressed` should travel with compact targets just like `checked`, because repeated workflows often need to avoid toggling an already-correct state.
- **Selected state is reusable context**: custom tabs/options using `aria-selected` should expose that state across parser packages, SDKs, and framework prompt renderers without changing target cache keys.

### 2026-05-13 ARIA Relationship-State Adjustment

The newest competitor docs reinforce that reusable action menus need relationship context, not just live boolean state. Playwright MCP refs are scoped to the current snapshot, Stagehand/Browserbase cached actions need validation, Browser Use Cloud profiles/CDP sessions keep repeated workflows warm, Firecrawl Interact resumes scrape sessions, and Cloudflare Browser Run/WebMCP is pushing typed page-provided actions. Plasmate should keep the local-first action-menu wedge and expose the relationship state agents need before reusing a cached target.

- **Current targets reduce redundant actions**: compact plans should surface `aria-current` so agents can avoid clicking an already-current tab, page link, or step.
- **Controlled panels are action context**: `aria-controls` should travel into action targets so agents can connect a disclosure or filter button to the affected region.
- **Popup type shapes the next step**: `aria-haspopup` should be visible across SDKs and prompt renderers so agents know whether a click opens a menu, listbox, tree, grid, or dialog without raw DOM recovery.

### 2026-05-13 Validation-Constraint Action Menu Adjustment

The newest competitor docs keep making cached actions depend on validation state, not just target identity. Playwright MCP refs are current-snapshot handles, Stagehand can cache `observe()`/action results locally or on Browserbase, and Browser Use/Firecrawl package session continuity around repetitive forms. Plasmate should keep the local-first wedge by making compact action plans carry the field constraints agents need before reusing a cached type action.

- **Autocomplete is planning context**: action targets should expose `autocomplete` tokens so agents can pick the right credential/profile data without re-walking SOM attrs.
- **Validation constraints reduce bad retries**: `minlength`, `maxlength`, and `pattern` should travel through Rust, schema, SDKs, parser packages, and adapters so agents know what a field accepts before typing.
- **Invalid state blocks blind replay**: `aria-invalid` should surface as compact `invalid` state without changing target `cache_key` values, letting cached plans stay stable while validation drift remains visible.

### 2026-05-13 Input-Affordance Action Menu Adjustment

Current browser-agent products keep making repeated actions depend on the target's current browser affordances. Playwright MCP refs still belong to a fresh accessibility snapshot, while Stagehand/Browserbase action caches only pay off when field modality and autocomplete state are visible before replay. Plasmate should keep the local-first wedge by making compact action targets carry input hints that affect credential selection, keyboard flow, and autocomplete suggestion state.

- **Input modality is planning context**: `inputmode` should travel through Rust, schema, SDKs, parser packages, and adapters so agents know whether a field expects email, decimal, numeric, search, or URL-style values.
- **Keyboard intent matters for form flow**: `enterkeyhint` should be exposed in compact menus so repeated workflows know whether Enter advances, searches, submits, or sends.
- **Autocomplete widgets need live state**: `aria-autocomplete` and `aria-activedescendant` should surface as compact action-plan cues without changing target `cache_key` values, preserving local action memory while making suggestion drift visible.

### 2026-05-14 Form-Relation Action Menu Adjustment

Current browser-agent products keep turning page state into reusable action menus. Playwright MCP refs remain valid only for the current snapshot, Stagehand local/server caches need page-state validation before replay, and Firecrawl plus Browser Use keep monetizing persistent sessions for repeated form workflows. Plasmate should keep the local-first wedge by making compact targets carry the relationships agents need before typing or submitting.

- **Form ownership prevents wrong-submit actions**: `form` should travel through Rust, schema, SDKs, parser packages, and adapters so controls outside a `<form>` still show which submission scope owns them.
- **Datalist references shape value choice**: `list` should surface in action plans so agents know when an input is backed by a suggestion source.
- **Error-message relationships explain invalid state**: `aria-errormessage` should surface as compact `errormessage` state without changing target `cache_key` values.

### 2026-05-14 Live-Region Action Menu Adjustment

Current browser-agent docs keep making repeated actions depend on current page state. Playwright MCP refs expire when the snapshot changes, Stagehand caching validates page state before replay, and Browser Use/CDP sessions preserve dynamic app state for long-running workflows. Plasmate should keep the local-first wedge by surfacing live-region state in the same portable compact target contract.

- **Busy state gates replay**: `aria-busy` should surface as compact `busy` state so agents know whether results or controls are still updating.
- **Live politeness shapes waiting**: `aria-live` should travel as `live` so agents can distinguish polite status updates from urgent alert feedback.
- **Announcement scope explains drift**: `aria-atomic` and `aria-relevant` should surface as `atomic` and `relevant` without changing deterministic `cache_key` values.

### 2026-05-14 Range and Orientation Action Menu Adjustment

Current browser-agent tools keep turning cached actions into validated replay. Range inputs, sliders, sortable tables, and oriented menus are common SaaS controls where agents need bounds and current value state before acting. Plasmate should expose those cues locally instead of forcing raw DOM recovery.

- **Range constraints bound replay**: `min`, `max`, and `step` should travel through Rust, schema, SDKs, parser packages, and adapters so agents know valid values before typing or dragging.
- **ARIA value state explains drift**: `aria-valuemin`, `aria-valuemax`, `aria-valuenow`, and `aria-valuetext` should surface as compact value cues without changing target `cache_key` values.
- **Orientation and sort guide action choice**: `aria-orientation` and `aria-sort` should remain visible so agents can distinguish vertical/horizontal controls and already-sorted columns.

### 2026-05-14 ARIA Set-Position Action Menu Adjustment

Current structured-browser tools make agents choose from the current page state, while action caches only stay useful when the target still has the expected context. Tree, menu, and listbox widgets often expose position through ARIA instead of visible text. Plasmate should preserve that ordinal context locally so repeated navigation plans can validate the same item without raw DOM recovery.

- **Nested depth is action context**: `aria-level` should surface as `level` so agents know whether a target is nested under the expected branch.
- **Ordinal position reduces ambiguity**: `aria-posinset` should surface as `posinset` for menu, tree, and listbox items that share labels or repeated actions.
- **Collection size helps validate drift**: `aria-setsize` should surface as `setsize` so cached plans can tell when a list has grown, shrunk, or reordered.

### 2026-05-14 Text-Entry Affordance Adjustment

Current browser-agent docs and recent developer commentary keep validating the same retention surface: compact action menus must expose the current field state before a cached typing plan is replayed. Playwright MCP keeps interaction tied to fresh accessibility snapshots, while Stagehand/Browserbase action caching only remains trustworthy when a field's keyboard and prompt affordances still match the cached target. Plasmate should keep the local-first wedge by surfacing small text-entry cues across the shared manifest.

- **Typing behavior is replay context**: `spellcheck` and `autocapitalize` should travel through Rust, schema, SDKs, parser packages, and adapters so agents understand language and virtual-keyboard behavior.
- **Direction capture matters for global forms**: `dirname` should be exposed as compact target context for bidirectional text-entry workflows.
- **Custom textboxes need prompt text**: `aria-placeholder` should surface as `aria_placeholder` without changing target `cache_key` values, preserving local action memory while making custom-field prompt drift visible.

### 2026-05-14 Form Submission Context Adjustment

Validated action menus are only sticky when they include the submission contract around the target. Playwright MCP keeps refs scoped to a current snapshot, while Stagehand/Browserbase action caches need the page state to match before replay. Plasmate should preserve the form-level metadata agents need to tell whether a cached type, upload, or submit step still belongs to the same workflow.

- **Destination changes risk**: `action`, `method`, and `target` should travel into compact targets as form context so agents can distinguish settings, checkout, and background-submit flows.
- **Encoding changes artifacts**: `enctype` and `accept-charset` should be visible before file uploads or internationalized form submissions are replayed.
- **Validation and autofill change readiness**: `novalidate` and form-level `autocomplete` should surface across SDKs and adapters so local action memory can be checked before a browser action is spent.

### 2026-05-14 Submitter Override Adjustment

Repeated SaaS forms often contain several submit buttons with different endpoints or validation behavior. Browser-agent competitors keep teaching users to validate cached actions against the current structured state before replay, so Plasmate should preserve the button-level submission contract as compact target context.

- **Submit buttons need identity beyond label**: `button_type` should expose whether a button submits, resets, or acts as a plain command.
- **Button overrides can change destination**: `formaction`, `formmethod`, `formenctype`, and `formtarget` should travel with the action target.
- **Validation mode is replay context**: `formnovalidate` should be visible so cached submit actions do not assume browser validation will run.

### 2026-05-14 Popover and Command Relationship Adjustment

Browser-native action relationships are becoming more important for agents that replay cached plans on modern app UIs. The Popover API gives buttons a declarative target and action, and `commandfor`/`command` generalize that model for popovers, dialogs, and custom commands. Plasmate should preserve those relationships as compact target context instead of forcing agents to rediscover them from raw DOM.

- **Invoker targets are replay context**: `popovertarget` should surface so an agent knows which panel a button affects.
- **Native action verbs reduce guesswork**: `popovertargetaction` and `command` should travel through action plans so cached clicks can be validated as show, hide, toggle, or custom commands.
- **Command ownership complements ARIA controls**: `commandfor` should sit alongside `aria-controls` as a native relationship cue without changing deterministic `cache_key` values.

### 2026-05-14 ARIA Relationship Context Adjustment

Current browser-agent products reward action menus that explain why a target is safe to reuse, not just that it is clickable. Playwright MCP keeps refs scoped to the fresh accessibility snapshot, Stagehand caches observed actions only when the page still validates, and hosted browser products make traces easy to inspect. Plasmate should make local compact targets carry more relationship context before users need those hosted traces.

- **Custom ownership is action context**: `aria-owns` should surface as `owns` so agents understand menu, listbox, and composite-widget ownership.
- **Guided flow order should be portable**: `aria-flowto` should surface as `flowto` for multi-step forms and custom onboarding flows.
- **Detailed help can stay out of raw DOM**: `aria-details` should surface as `details` so agents can locate extended help or validation panels without changing deterministic `cache_key` values.

### 2026-05-13 Keyboard-Affordance Action Menu Adjustment

Current Playwright MCP and Stagehand docs keep validating action surfaces that are current, inspectable, and reusable. Browserbase, Browser Use, and Firecrawl add hosted sessions and traces around that same workflow, but Plasmate's sticky local-first wedge is still a portable action contract that carries browser-like affordances everywhere.

- **Native shortcuts are action context**: `accesskey` should travel through Rust, schema, SDKs, parser packages, and adapters so agents can understand page-provided keyboard activation paths.
- **ARIA shortcuts reduce ambiguity**: `aria-keyshortcuts` should surface as compact `keyshortcuts` state for buttons, menu items, and custom SaaS controls.
- **Custom role descriptions help prompt selection**: `aria-roledescription` should be visible as `roledescription` without changing deterministic `cache_key` values, giving agents more human-facing context for custom widgets.

### 2026-05-14 ARIA Action Role Coverage Adjustment

Current browser-agent products are teaching developers to treat accessibility state as the action menu. Playwright MCP exposes interaction refs from current accessibility snapshots, while Stagehand v3 `observe()` returns structured actions that teams validate and cache. Plasmate should keep closing small local role-parity gaps before chasing hosted browser infrastructure.

- **Numeric widgets need actions**: ARIA `slider` and `spinbutton` should map to actionable `text_input` targets so agents can adjust SaaS quotas, limits, and settings from local SOM.
- **Listbox options are choices**: ARIA `option` should map to an actionable target with selected state preserved, giving custom selects parity with native select options.
- **Conformance keeps adapters thin**: the `016-action-semantics` fixture should cover these roles so parser, SDK, and framework work can promote the same role contract without bespoke DOM recovery.

### 2026-05-14 Inert Availability Adjustment

Current action-replay products keep validating whether a target is safe before reuse. Playwright MCP refs are snapshot-scoped, Stagehand/Browserbase cache actions only when page state still matches, and Browser Run/WebMCP is making typed interaction contracts more prominent. Plasmate should make native inert state part of the portable local action contract.

- **Inert blocks replay**: controls inside an `inert` subtree should remain visible in SOM but surface `inert` and `blocked_reason=inert` in compact action plans.
- **Cache keys stay target-focused**: inert state should not alter deterministic `cache_key` values, so local action memory can still compare the same target while seeing current availability drift.
- **Shared fixtures prevent adapter drift**: `015-action-state` and the action-availability manifest should assert inert gating across Rust, parser packages, SDKs, and framework prompt renderers.

### 2026-05-14 Graphical Submitter Adjustment

Browser-agent products keep turning repeated form work into validated action replay: Playwright MCP refs are fresh snapshot handles, Stagehand/Browserbase cache observed actions after state validation, and Browser Run is broadening hosted MCP/CDP browser access. Plasmate should keep closing local HTML submitter gaps before considering hosted infrastructure.

- **Image submitters are buttons**: `input type="image"` should compile as an actionable button because it submits forms on click.
- **Input submitters need button identity**: input-backed `submit`, `button`, `reset`, and `image` controls should expose `button_type`.
- **Icon-only submitters need context**: graphical submitters should resolve labels from `alt` and preserve `alt` plus `src` so agents can recognize branded/icon-only actions without raw DOM recovery.

### 2026-05-14 Hidden Descendant Text Adjustment

Current browser-agent products keep tying replay to fresh structured state: Playwright MCP refs belong to the current accessibility snapshot, Stagehand/Browserbase cache actions after validating the page still matches, and Browser Run/WebMCP is widening hosted interaction contracts. Plasmate should make local SOM text match visible page state across the same surfaces agents use for action planning.

- **Visible parent text must stay visible**: stylesheet-hidden descendants should not leak into paragraph or button text.
- **Labels are cache evidence**: `label for` and `aria-labelledby` indexes should skip hidden fragments so cached form plans compare against visible names.
- **Structured summaries need parity**: select options, list items, table captions, and table cells should ignore hidden descendants.

### 2026-05-14 Select Option State Adjustment

Current competitor direction keeps validating cached action plans against the current structured page state. Playwright MCP snapshots are fresh interaction surfaces, Stagehand/Browserbase cache actions only after state checks, and Browser Run/WebMCP is widening hosted browser contracts. Plasmate's local-first roadmap should make native select menus precise enough for repeated SaaS workflows without raw DOM recovery.

- **Option values should match browsers**: when an `<option>` omits `value`, SOM should expose the visible option text as the submitted value.
- **Unavailable choices need to be visible**: disabled options should remain in the compact menu with `disabled=true` so agents can explain why they did not select them.
- **Grouped and multi-select menus need current state**: optgroup labels and `selected_values` should become shared action-plan context before this lands in adapter conformance.
- **Schema and SDK parity decide stickiness**: option groups, disabled option state, implicit single-select values, and select `size` need to validate in the public schema and flow through parser/SDK action plans before agents can safely cache menu choices across runtimes.

### 2026-05-15 HTML ID Provenance Adjustment

Current competitor docs keep making the interaction surface both structured and replay-aware. Playwright MCP gives snapshot-scoped refs, Stagehand `observe()` returns cacheable action objects, and Browser Run/WebMCP is testing browser-native tool contracts. Plasmate should treat `html_id` as local DOM provenance for compact targets, not just a Rust compiler field.

- **Live DOM resolution matters**: SDK and parser action plans should carry `html_id` so agents can move from a compact SOM target back to `document.getElementById()` or selector execution without re-reading raw DOM.
- **Cache keys stay stable**: `html_id` is useful provenance, but should not change deterministic action `cache_key` values because the SOM id, role, label, actions, and field identity remain the cache target.
- **Parity beats isolated fields**: Python, Node, Go, and shared fixtures should expose the same `html_id` lookup and action-plan context before adapters advertise live-DOM replay support.

### 2026-05-15 Locator Provenance Adjustment

Official docs continue to split reusable action identity from current-page handles. Playwright MCP refs are stable only inside the current accessibility snapshot, Stagehand's local/server caches replay observed actions after matching current state, and Firecrawl/Browser Use profiles keep hosted sessions warm. Plasmate should answer with local non-cache provenance that helps agents resolve, test, and debug compact targets without changing deterministic action memory.

- **Developer anchors are replay evidence**: preserve `title`, raw `role` tokens, and `data-testid`/`data-test`/`data-qa` locators as compact attrs for action targets.
- **Provenance must stay non-cache**: these fields help live DOM resolution and diagnostics, but should not change `cache_key` values because they can churn independently of semantic target identity.
- **Adapters should render the same cues**: Browser Use, LangChain, Vercel AI, parser packages, and SDKs should expose `title`, `source_role`, and `test_id` together so no integration needs raw DOM recovery for common app-authored anchors.

### 2026-05-15 Drag/Drop Replay Adjustment

Current competitor docs keep tying repeat automation to validated current page state: Playwright MCP refs are scoped to fresh snapshots, Stagehand/Browserbase action caching depends on DOM validation, and Browser Use/Firecrawl package hosted sessions plus skills for recurring workflows. Plasmate should keep its local-first replay wedge by preserving drag/drop state in the same compact action targets agents already cache.

- **Drag/drop is a replay cue**: native `draggable`, ARIA `grabbed`, and ARIA `dropeffect` should travel through Rust SOM, schema, parser packages, SDKs, and framework prompt renderers.
- **Do not destabilize cache keys**: drag state can change while the semantic target stays the same, so it should be validation context rather than deterministic action-key material.
- **Boards and builders are sticky SaaS cases**: kanban boards, upload builders, scheduling grids, and workflow canvases need drag/drop cues before local replay can compete with hosted action-memory products.

### 2026-05-16 Link Navigation Replay Adjustment

The browser-agent market keeps rewarding tools that validate current page state before reusing an action. Playwright MCP refs are snapshot-scoped, Stagehand and Browserbase action caches depend on selector/DOM validation, and hosted browser products package profiles and traces around repeated workflows. Plasmate's local-first answer should keep semantic cache keys stable while making link side effects visible enough for agents to reject stale clicks.

- **Locale matters for repeated navigation**: `hreflang` should travel through Rust SOM, schema, SDKs, parser packages, and framework renderers so agents can distinguish same-label links that target different languages.
- **Resource type is replay context**: link MIME `type` should surface beside `href` so cached actions can tell a normal HTML navigation from a PDF, CSV, or alternate representation.
- **Privacy policy changes are side effects**: `referrerpolicy` should be compact action context without becoming cache-key material, preserving local action memory while exposing navigation risk.

### 2026-05-16 ARIA Naming Provenance Adjustment

Validated action reuse remains the durable retention hook: Playwright MCP refs are scoped to the current accessibility snapshot, Stagehand/Browserbase validate cached actions against DOM state, and managed browser platforms sell profile continuity for repeated workflows. Plasmate should preserve raw ARIA naming relationships beside resolved labels/descriptions so cached local targets can be checked without raw DOM recovery.

- **Raw names are replay evidence**: `aria-label` should surface as compact `aria_label` action context.
- **Label relationships are DOM anchors**: `aria-labelledby` should travel as `labelledby` through schema, SDKs, parser packages, and adapters.
- **Descriptions need provenance**: `aria-describedby` should stay beside resolved `description` text as `describedby`.

### 2026-05-16 Selector-Aware Cache Adjustment

Scoped cache identity is now a direct competitive point: Playwright MCP refs remain current-snapshot handles, while Stagehand recommends selector-scoped action caching so unrelated surrounding DOM does not invalidate repeated automation. Plasmate should keep the same benefit local and SOM-native by deriving compact selector views from validated full-page SOM cache hits.

- **Selectors are cache identity**: `main`, `form`, role selectors, `interactive`, and `action:<name>` cache entries should be distinct from the full page while sharing content-hash validation.
- **HTML ids stay browser-like**: `#id` selector cache keys should preserve case, while action and role selectors normalize case for practical agent prompts.
- **Full SOM can feed narrow views**: a fresh full-page cache entry should materialize selector-specific JSON without another compile or hosted selector store.

### 2026-05-16 Daemon Selector Cache Adjustment

The cache work now needs to live where repeat users feel latency: the warm daemon path. Stagehand/Browserbase is training users to expect repeated action planning to skip expensive reasoning once a selector is validated; Plasmate's local answer is to let the daemon reuse content-hash-validated SOM cache entries for full-page and selector-filtered fetches.

- **Selectors travel to the warm process**: CLI fetch requests should pass `selector` into the daemon so the daemon owns narrow SOM cache identity.
- **Cache hits avoid recompilation**: after fetch and hash validation, daemon requests should return cached full or selector-filtered SOM JSON before rerunning JS execution and SOM compilation.
- **Next visibility step**: daemon health/status should expose cache hit, miss, stale, and selector-entry counts so repeated-work savings are inspectable during agent workflows.

### 2026-05-16 Daemon Cache Observability Adjustment

Current browser-agent products are making cache reuse explainable. Playwright MCP shows the fresh snapshot refs an agent can use, Stagehand/Browserbase pairs selector/action caching with observability and replay, and Firecrawl/Browser Use keep broadening hosted session continuity. Plasmate should not pivot into hosted infrastructure, but local repeated-work savings need to be visible from the daemon.

- **Health carries cache inventory**: daemon health should expose full-entry, selector-entry, hit, miss, stale, eviction, cached-byte, and avoided-HTML counters so users can tell whether repeated prompts are benefiting from local SOM memory.
- **Status is an operator surface**: `plasmate daemon status` should summarize cache state in readable CLI output rather than requiring users to parse raw health JSON.
- **Stats count every cache path**: unvalidated cache reads should update hit/miss and avoided-byte counters too, keeping future instant/prefetch status output honest.

### 2026-05-16 MCP Cache Surface Adjustment

The latest competitor docs keep making repeated-work memory visible where agents actually call tools. Stagehand now documents local and Browserbase action caches, Browserbase markets action caching with prompt observability and session replay, Firecrawl keeps broad MCP scrape/search/extract distribution, and Browser Run exposes MCP/CDP sessions plus recordings and WebMCP labs. Plasmate should keep the local-first wedge by making stateless MCP calls reuse and inspect selector-aware SOM cache entries before adding heavier trace or hosted-browser features.

- **MCP fetches should warm up**: `fetch_page`, `extract_text`, and `extract_links` should share an in-process SOM cache so repeated MCP agent calls avoid JS/SOM recompilation after content-hash validation.
- **Selector views are first-class MCP cache entries**: MCP calls with `selector='interactive'`, `action:<name>`, role selectors, or `#id` should reuse the same full-page-derived selector cache path as the daemon.
- **Agents need cache introspection**: an MCP `cache_status` tool should expose hit/miss/stale/eviction counters, full versus selector inventory, cached bytes, avoided HTML bytes, and capacity so users can tell whether repeated workflows are getting cheaper.

### 2026-05-16 MCP Session Observability Adjustment

The latest market read keeps moving from raw browser sessions toward inspectable, replayable action state. Playwright MCP makes fresh snapshot refs the unit of action, Stagehand documents local and managed action caches, Browserbase markets prompt observability and replay around cached actions, Cloudflare Browser Run exposes CDP/MCP sessions with recordings, and Firecrawl keeps browser sessions in its MCP distribution. Plasmate should still avoid a hosted-browser pivot, but stateful MCP sessions need the same local trust-building surface as stateless cache calls.

- **Session inventory is an MCP tool**: agents should be able to inspect active session count, capacity, oldest age, and idle time before creating more browser state.
- **Open/navigate rebuilds replay indexes**: `open_page` and `navigate_to` should refresh structured data and CDP node maps after compiling SOM so follow-up interaction tools operate on the current action surface.
- **Nested targets are stateful targets**: click lookup and CDP SOM-id lookup should traverse child and shadow-root elements, matching the compiler and parser contract for modern web-component UIs.

### 2026-05-16 MCP Interaction Replay-Readiness Adjustment

Current competitor direction keeps pushing from sessions toward replayable, inspectable session state. Playwright MCP refreshes structured refs after page changes, Stagehand validates cached actions against the current DOM before replay, Browserbase and Cloudflare Browser Run sell recordings/replay around browser sessions, and Firecrawl keeps browser-session workflows in MCP. Plasmate should keep the local-first wedge by making every stateful MCP interaction refresh the same replay indexes as navigation before adding trace export or hosted scale.

- **Mutation tools must rebuild replay indexes**: `click`, `type_text`, `select_option`, `scroll`, `toggle`, and `clear` should preserve structured data and rebuild CDP node maps after recompiling SOM.
- **Session status should show loaded state**: `session_status` should expose available capacity, loaded URLs, titles, SOM sizes, element/interactive counts, node-map counts, and structured-data presence so agents can inspect replay readiness.
- **Cache restore should reuse the same update path**: stateful cache hits should restore `effective_html`, structured data, and node maps through the centralized page-state updater before they are allowed into replay flows.

### 2026-05-16 MCP Session Cache-Restore Adjustment

Current competitor docs keep validating the same product wedge: structured browser state has to be inspectable before it is replayable. Playwright MCP keeps refs scoped to a fresh accessibility snapshot, Stagehand and Browserbase cache actions only after validating page state, Browserbase and Cloudflare Browser Run package sessions with replay/recordings/Live View, and Firecrawl continues to distribute hosted browser sessions through MCP. Plasmate should not pivot into a hosted browser fleet. The stickier local move is to make MCP session creation reuse validated SOM cache entries only when they carry the post-JS HTML needed for follow-up interaction.

- **Cache entries need replay payloads**: full-page SOM cache entries should optionally carry `effective_html` so stateful sessions can bootstrap `evaluate`, click, type, and CDP node maps from a validated cache hit.
- **Fetch/extract should seed sessions**: stateless MCP fetch/extract calls should store restorable full-page cache entries when JavaScript runs, so a later `open_page` does not pay a second JS/SOM compile for the same content.
- **Restore must stay observable**: `open_page` and `navigate_to` should return `cache_restored`, while `cache_status` and `session_status` expose effective-HTML inventory and per-session raw/effective HTML sizes.

### 2026-05-16 Label Parity Adjustment

Current browser-agent tools keep making human-facing names the unit of interaction. Playwright MCP refs are selected from accessibility snapshots, while Stagehand/Browserbase only reuse cached actions when current page state still validates. Plasmate's local-first retention path is to make every helper honor the same label surface the compiler emits, especially for icon-only links and label-only controls that otherwise disappear from search or markdown views.

- **SDK search must include labels**: Node and Python SDK text search should match `label` as well as visible `text`.
- **Link inventories need accessible names**: parser `get_links()` helpers should fall back to labels so icon links remain usable in summaries.
- **Markdown is agent context**: parser markdown renderers should preserve label-only links instead of emitting empty link text.

### 2026-05-16 Framework Replay Lookup Adjustment

Competitor pressure keeps moving reusable action state from browser engines into application code. Playwright MCP gives agents snapshot-scoped refs, Stagehand/Browserbase makes validated cached actions a workflow primitive, and hosted browser tools wrap replay with traces. Plasmate's local-first answer is to let framework adapters resolve cached action targets by stable local identifiers instead of forcing every app to scan compact menus.

- **Framework helpers should index targets**: Browser Use, LangChain, and Vercel AI need first-class lookup buckets for SOM id, `cache_key`, `html_id`, and `test_id`.
- **Defaults should stay action-safe**: enabled-only filtering should remain the easy path where the framework already filters unavailable targets.
- **Release fixtures are next**: framework lookup helpers should be promoted into shared cross-adapter conformance so replay resolution keeps matching parser and SDK behavior.

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
- [x] ARIA landmark fallback role tokens compile into labelled search/navigation regions
- [x] ARIA widget fallback role tokens preserve menu checkbox/radio action targets
- [x] Uppercase `aria-hidden="TRUE"` and inline `opacity: 0` are stripped as hidden state
- [x] Action-semantics conformance covers role fallback tokens and inline/ARIA hidden variants
- [x] Compact action plans expose non-empty control `value` fields across parser packages, SDKs, and framework adapters
- [x] Compact action plans normalize native and ARIA checked state across parser packages, SDKs, and framework adapters
- [x] Shared action-availability manifest asserts value and checked state without changing deterministic action cache keys
- [x] Compact action plans expose ARIA expanded, pressed, and selected state across parser packages, SDKs, and framework adapters
- [x] Browser Use, LangChain, and Vercel AI prompt renderers include expanded/pressed/selected action-state cues
- [x] Shared action-availability manifest asserts ARIA state cues without changing deterministic action cache keys
- [x] Rust compiler and SOM schema preserve ARIA controls and haspopup relationship state
- [x] Compact action plans expose current, controls, and haspopup cues across parser packages, SDKs, and framework adapters
- [x] Shared action-availability manifest asserts ARIA relationship cues without changing deterministic action cache keys
- [x] Rust compiler and SOM schema preserve form validation constraints and ARIA invalid state
- [x] Compact action plans expose autocomplete, minlength, maxlength, pattern, and invalid cues across parser packages, SDKs, and framework adapters
- [x] Shared action-availability manifest asserts validation constraints without changing deterministic action cache keys
- [x] Rust compiler and SOM schema preserve inputmode, enterkeyhint, ARIA autocomplete, and active-descendant state
- [x] Compact action plans expose inputmode, enterkeyhint, aria_autocomplete, and active_descendant across parser packages, SDKs, and framework adapters
- [x] Shared action-availability manifest asserts input-affordance cues without changing deterministic action cache keys
- [x] Rust compiler and SOM schema preserve accesskey, ARIA keyshortcuts, and ARIA roledescription cues
- [x] Compact action plans expose accesskey, keyshortcuts, and roledescription across parser packages, SDKs, and framework adapters
- [x] Shared action-availability manifest asserts keyboard/custom-role cues without changing deterministic action cache keys
- [x] Rust compiler and SOM schema preserve form, list, and ARIA errormessage relationship cues
- [x] Compact action plans expose form, list, and errormessage across parser packages, SDKs, and framework adapters
- [x] Shared action-availability manifest asserts form-relation cues without changing deterministic action cache keys
- [x] Rust compiler and SOM schema preserve ARIA busy, live, atomic, and relevant cues
- [x] Compact action plans expose busy, live, atomic, and relevant across parser packages, SDKs, and framework adapters
- [x] Shared action-availability manifest asserts live-region cues without changing deterministic action cache keys
- [x] Rust compiler and SOM schema preserve popover and command relationship cues
- [x] Compact action plans expose popovertarget, popovertargetaction, commandfor, and command across parser packages, SDKs, and framework adapters
- [x] Shared action-availability manifest asserts popover/command cues without changing deterministic action cache keys
- [x] Rust compiler and SOM schema preserve ARIA owns, flowto, and details relationship cues
- [x] Compact action plans expose owns, flowto, and details across parser packages, SDKs, and framework adapters
- [x] Shared action-availability manifest asserts ARIA owns/flowto/details cues without changing deterministic action cache keys
- [x] Rust compiler and SOM schema preserve range constraints plus ARIA orientation, sort, and value cues
- [x] Compact action plans expose min, max, step, orientation, sort, valuemin, valuemax, valuenow, and valuetext across parser packages, SDKs, and framework adapters
- [x] Shared action-availability manifest asserts range/orientation/value cues without changing deterministic action cache keys
- [x] Rust compiler and SOM schema preserve ARIA readonly, multiline, and multiselectable widget cues
- [x] Compact action plans expose readonly, multiline, and multiselectable across parser packages, SDKs, and framework adapters
- [x] Shared action-availability manifest asserts ARIA readonly gating and widget affordances without changing deterministic action cache keys
- [x] Rust compiler and SOM schema preserve ARIA level, posinset, and setsize cues
- [x] Compact action plans expose level, posinset, and setsize across parser packages, SDKs, and framework adapters
- [x] Shared action-availability manifest asserts ARIA set-position cues without changing deterministic action cache keys
- [x] Rust compiler and SOM schema preserve spellcheck, autocapitalize, dirname, and ARIA placeholder cues
- [x] Compact action plans expose spellcheck, autocapitalize, dirname, and aria_placeholder across parser packages, SDKs, and framework adapters
- [x] Shared action-availability manifest and 016-action-semantics fixture assert text-entry affordance cues without changing deterministic action cache keys
- [x] Rust compiler and SOM schema preserve upload constraints with accept, capture, and native multiple state
- [x] Compact action plans expose name, accept, capture, and multiple across parser packages, SDKs, and framework adapters
- [x] Shared action-availability manifest asserts upload constraints and multiple-selection state for cacheable action targets
- [x] Rust compiler and SOM schema preserve form target, enctype, novalidate, accept-charset, and autocomplete context
- [x] Compact action plans expose form_action, form_method, form_target, form_enctype, form_novalidate, form_accept_charset, and form_autocomplete across parser packages, SDKs, and framework adapters
- [x] Shared action-availability manifest asserts form submission context for cacheable action targets
- [x] Rust compiler and SOM schema preserve submit-button override cues with button_type, formaction, formmethod, formenctype, formtarget, and formnovalidate
- [x] Compact action plans expose submit-button override cues across parser packages, SDKs, and framework adapters
- [x] Shared action-availability manifest asserts submit-button override context for cacheable action targets
- [x] ARIA slider and spinbutton roles compile into actionable text-input targets
- [x] ARIA option roles compile into actionable button targets with selected state
- [x] Action-semantics conformance covers slider, spinbutton, and option action roles
- [x] Rust compiler and SOM schema preserve inherited inert state for action targets
- [x] Compact action plans expose inert availability gating across parser packages, SDKs, and framework adapters
- [x] Shared action-state and action-availability fixtures assert inert targets without changing deterministic action cache keys
- [x] Rust compiler maps graphical submit inputs to actionable buttons, resolves alt labels, and preserves button_type, alt, and src context
- [x] Single-select controls infer the browser-default first selected option when markup omits `selected`
- [x] Disabled optgroups propagate disabled state to child option summaries
- [x] SOM schema/spec, parser packages, SDKs, Browser Use, LangChain, and Vercel AI carry selected_values and select size context
- [x] Parser packages and SDKs preserve html_id, expose original-HTML-id lookup helpers, and carry html_id in compact action plans
- [x] Browser Use, LangChain, and Vercel AI action-plan renderers surface html_id
- [x] Shared action-availability manifest asserts html_id parity without changing deterministic cache keys
- [x] Rust SOM attrs and schema preserve title, source_role, and test_id locator provenance
- [x] Parser packages, SDKs, Browser Use, LangChain, and Vercel AI action plans surface title, source_role, and test_id without changing deterministic cache keys
- [x] Shared action-availability manifest asserts locator provenance for parser, SDK, and framework adapters
- [x] Python/Node parser packages, Python/Node SDKs, and Go SDK expose compact action target lookup/index helpers by id, cache_key, html_id, and test_id
- [x] Parser and SDK tests cover enabled-only action plans plus replay lookup by cache key, DOM id, and test id
- [x] Rust SOM attrs and schema preserve draggable, aria-grabbed, and aria-dropeffect replay cues
- [x] Parser packages, SDKs, Browser Use, LangChain, and Vercel AI action plans surface draggable, grabbed, and dropeffect without changing deterministic cache keys
- [x] Shared action-availability manifest asserts drag/drop replay cues across parser, SDK, and framework adapters
- [x] Rust compiler and SOM schema preserve link replay cues with hreflang, type, and referrerpolicy
- [x] Parser packages, SDKs, Browser Use, LangChain, and Vercel AI action plans surface hreflang, type, and referrerpolicy without changing deterministic cache keys
- [x] Shared action-availability manifest asserts link locale, resource type, and referrer-policy cues across parser, SDK, and framework adapters
- [x] Rust compiler and SOM schema preserve ARIA naming provenance with aria-label, aria-labelledby, and aria-describedby
- [x] Parser packages, SDKs, Browser Use, LangChain, and Vercel AI action plans surface aria_label, labelledby, and describedby without changing deterministic cache keys
- [x] Shared action-availability manifest asserts ARIA naming provenance beside resolved description text
- [x] Selector-aware SOM cache entries for repeated daemon agent prompts
- [x] Daemon cache health/status observability for selector hit/miss behavior
- [x] MCP stateless fetch/text/link SOM cache reuse
- [x] MCP cache_status observability for selector hit/miss behavior
- [x] MCP session_status observability for stateful browser sessions
- [x] MCP session_status loaded-session replay inventory
- [x] Stateful MCP open/navigate rebuilds structured data and CDP node maps
- [x] Stateful MCP mutation tools rebuild structured data and CDP node maps
- [x] Stateful MCP click/CDP lookup traverses nested and shadow-root elements
- [x] MCP stateful open/navigate restore validated cached SOM and effective HTML
- [x] MCP cache_status/session_status expose effective-HTML replay inventory
- [x] Python and Node SDK text search matches label-only controls
- [x] Python and Node parser link/markdown helpers preserve label-only accessible links
- [x] Browser Use, LangChain, and Vercel AI expose action-target replay lookup helpers
- [x] Python/Node parser packages and Python/Node/Go SDKs auto-resolve replay ids across SOM id, cache_key, html_id, and test_id buckets
- [x] Browser Use, LangChain, and Vercel AI direct replay lookup helpers auto-resolve stored replay ids while preserving enabled-only filtering
- [x] Package and adapter tests cover auto replay lookup without bespoke action-menu scans
- [x] Python/Node parser packages and SDKs expose explicit accessible-label lookup for elements and compact action targets
- [x] CDP DOM query selectors resolve `#html_id`, `#som_id`, test-id selectors, roles, text, and labels in SOM document order
- [x] CDP accessibility trees include nested and shadow-root SOM elements with backend node ids
- [x] CDP accessibility nodes expose disabled/readonly availability properties for replay validation
- [ ] Session replay/trace export for debugging agent runs
- [ ] Wire `016-action-semantics` into parser/SDK and adapter conformance runners for fallback roles and hidden-state variants
- [ ] Promote shadow-DOM and web-component cases into shared cross-adapter fixtures
- [ ] Promote html_id DOM-provenance cases into adapter conformance fixtures
- [ ] Promote locator-provenance cases into broader Rust/parser/SDK and adapter conformance fixtures
- [ ] Promote auto replay lookup into the shared action-manifest release gate
- [ ] Add cross-adapter fixtures for enriched compact action-plan metadata
- [ ] Promote ARIA relationship-state cases, including owns/flowto/details, into the broader action-state/action-semantics conformance suites
- [ ] Promote range and orientation cases into broader parser, SDK, and adapter conformance fixtures
- [ ] Promote ARIA widget affordance cases into broader parser, SDK, and adapter conformance fixtures
- [ ] Promote ARIA set-position cases into broader Rust/parser/SDK and adapter conformance fixtures
- [ ] Promote text-entry affordance cases into broader parser, SDK, and adapter conformance fixtures
- [ ] Promote upload-affordance cases into broader Rust/parser/SDK and adapter conformance fixtures
- [ ] Promote form-submission context cases into broader Rust/parser/SDK and adapter conformance fixtures
- [ ] Promote submit-button override cases into broader Rust/parser/SDK and adapter conformance fixtures
- [ ] Promote graphical submitter cases into the shared action manifest and adapter conformance fixtures
- [ ] Promote inert availability cases into broader parser, SDK, and adapter conformance fixtures
- [ ] Promote validation-constraint cases into broader parser, SDK, and adapter conformance fixtures
- [ ] Promote keyboard-affordance cases into broader Rust/parser/SDK conformance fixtures
- [ ] Promote drag/drop replay cues into broader Rust/parser/SDK and adapter conformance fixtures
- [ ] Promote link navigation replay cues into broader Rust/parser/SDK and adapter conformance fixtures
- [ ] Promote label-only link/control parity into shared parser, SDK, and adapter conformance fixtures
- [ ] Add cross-adapter accessible-description fixtures
- [ ] Wire disabled/required action-state fixtures into cross-adapter parser/SDK conformance runners
- [x] Promote adapter availability checks into shared cross-adapter fixtures
- [x] Add runtime Vercel AI fixture tests once the package has a local test runner
- [x] Extend shared action-availability expectations into parser-package and SDK conformance tests
- [ ] Promote the action-manifest CI job from quick checks to full conformance after dependency-cache tuning
- [ ] WebMCP/watchlist research spike: track whether browser-native tool exposure changes SOM adapter strategy
