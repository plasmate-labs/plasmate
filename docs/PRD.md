# Plasmate PRD: Agent Stickiness and Roadmap Direction

Last updated: 2026-05-04

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

- Added region-id selector support while keeping HTML id selection.
- Added common ARIA widget role mapping into actionable SOM elements.
- Hardened inline hidden-style stripping against spacing and casing variants.
- Updated roadmap direction around cached structured actions, MCP distribution,
  and accessibility/SOM parity.

## Next Steps

- Implement selector-aware SOM cache entries for `main`, `form`, and `#id`
  prompts.
- Add trace export for MCP/AWP sessions so users can debug why an agent clicked
  or selected an element.
- Add conformance cases for ARIA-heavy SaaS pages and compare output against
  Playwright MCP snapshots.
- Audit ecosystem repos for stale install docs, tool counts, and schema drift.
