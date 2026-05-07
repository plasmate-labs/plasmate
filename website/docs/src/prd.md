# PRD: Agent Stickiness and Roadmap Direction

Last updated: 2026-05-07

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

## Ecosystem Surface

The project already spans a large number of package and integration surfaces: Rust CLI/daemon/MCP/CDP/AWP core, Python SDK, Node SDK, Go SDK, LangChain, Browser Use, Vercel AI, SOM parser packages for Python and Node, plugin examples, smoke tests, generated docs, comparison pages, and marketing assets. This breadth is a distribution advantage only if contracts stay synchronized. Short-term roadmap work should favor conformance fixtures, shared schema tests, and adapter docs over one-off integration logic.

## Requirements

1. Preserve actionable structure: SOM must capture common accessibility roles, stable ids, labels, forms, links, state, and selectors that agents can reuse.
2. Reduce repeated-work cost: SOM cache, SOM diff, and selector-aware cache entries should make repeat visits cheaper than first visits.
3. Improve inspectability: expose traces, coverage scorecards, and reproducible fixtures so teams can trust extraction behavior.
4. Keep ecosystem adapters thin: SDKs and integrations should share conformance expectations instead of forking extraction logic.
5. Stay local-first by default: hosted competitors can own scale infrastructure; Plasmate should own local speed, privacy, and open protocol fit.

## Current Run Changes

- 2026-05-07:
  - SOM JSON Schema now accepts current Rust core output for `shadow`, `iframe`, `details`, ARIA state, and actionability attrs.
  - Python and Node parser packages now expose shadow-root types and traverse `shadow.elements` in element, text, link, and interactive queries.
  - Python and Node SDK query helpers now find shadow-root elements by id, role, text, and actionability.
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
- Add conformance cases for ARIA-heavy SaaS pages and compare output against Playwright MCP snapshots.
- Add accessible-name conformance for `aria-labelledby`, `<label for>`, and custom controls that rely on `contenteditable`/`tabindex`.
- Promote the new SDK/parser shadow-root tests into shared conformance fixtures that run against every adapter before release.
- Audit ecosystem repos for stale install docs, tool counts, and schema drift.
