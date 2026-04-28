# Plasmate Product Requirements

Last updated: 2026-04-28

## Product Position

Plasmate is the local-first semantic browser engine for AI agents. It should stay focused on the problem competitors do not solve well: turning the live web into compact, deterministic, actionable structure without forcing teams into managed Chrome fleets or screenshot-heavy agent loops.

## Target Users

- Agent framework maintainers who need lower-token browser state for tools like Browser Use, LangChain, CrewAI, AutoGen, and Playwright MCP.
- AI application teams running recurring web workflows where token cost, latency, and privacy matter more than pixel-perfect rendering.
- Scraping and research teams that want structured extraction with reproducible selectors, local execution, and no mandatory cloud browser provider.
- Standards-oriented developers who need MCP, CDP, AWP, SDKs, and a stable SOM schema across languages.

## 2026 Market Read

Current market pull is toward hybrid browser automation: deterministic Playwright/CDP for reliable repeated steps, LLM/vision for variable steps, and managed browser fleets for scale, proxies, CAPTCHA handling, session replay, and observability.

Competitor signals:

- Browserbase and Stagehand are winning mindshare on developer experience, managed sessions, action caching, and hosted MCP.
- Playwright MCP is becoming the default free structured browser tool in IDE agent workflows, using accessibility snapshots instead of screenshots.
- Browser Use remains the Python agent framework gravity well and now pushes both local and cloud execution.
- Steel, Hyperbrowser, and similar cloud browser APIs compete on fleet infrastructure, proxying, CAPTCHA, and session startup.
- Skyvern-style products are moving upmarket with visual workflow builders, credential vaults, audit logs, and compliance.
- Firecrawl is strong in URL/domain-level structured extraction and agentic data collection.

Implication: Plasmate should not pivot into a generic hosted Chrome provider first. The stickier wedge is becoming the semantic layer that these tools can call when they need cheap, private, structured page state.

## Stickiness Requirements

1. Schema parity across Rust, Python, Node, Go, and parser packages.
2. Drop-in adapters for high-growth agent ecosystems, starting with Browser Use, Playwright MCP-shaped snapshots, and Stagehand-style observe/extract flows.
3. Durable session identity: cookies, auth profiles, replayable action traces, and stable SOM element IDs.
4. Trust surface: local-first execution, explicit MCP safety posture, audit logs, and no hidden cloud dependency.
5. Measurable cost advantage: benchmark pages, token deltas, latency, and repeatable examples in docs and SDK tests.

## Current Run Changes

- SDK and parser packages now model newer SOM fields: `html_id`, `details`, `iframe`, iframe attrs, and declarative `shadow` roots.
- Python and Node query helpers now traverse shadow DOM, so agents can find interactive elements inside web components.
- Selector handling now trims whitespace and accepts `article` as a `content` alias.

## Near-Term Acceptance Criteria

- All SDKs accept SOM output emitted by the Rust compiler without rejecting known fields.
- Query helpers in each SDK return the same element set for regular children and shadow-root children.
- Roadmap pages clearly state the integration strategy against Playwright MCP, Browser Use, Stagehand, Skyvern, and Firecrawl.
- Benchmarks include at least one comparison against accessibility snapshots, one against markdown extraction, and one against screenshot/vision flows.

## Research Sources

- Browserbase MCP and Stagehand: https://www.browserbase.com/mcp
- Stagehand v3 release notes: https://www.browserbase.com/blog/stagehand-v3
- Playwright MCP docs: https://playwright.dev/docs/getting-started-mcp
- Browser Use repository: https://github.com/browser-use/browser-use
- Steel browser infrastructure: https://steel.dev/
- Skyvern product overview: https://www.skyvern.com/products
- Firecrawl structured extraction docs: https://docs.firecrawl.dev/features/extract
