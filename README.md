<p align="center">
  <img src="website/brand/plasmate-mark.png" alt="Plasmate" width="80" />
</p>

<h1 align="center">Plasmate</h1>

<p align="center">
  The browser engine for agents.<br/>
  HTML in. Semantic Object Model out.
</p>

<p align="center">
  <a href="https://plasmate.app">Website</a> &middot;
  <a href="https://docs.plasmate.app">Docs</a> &middot;
  <a href="https://plasmate.app/compare">Benchmarks</a> &middot;
  <a href="https://crates.io/crates/plasmate">Crates.io</a> &middot;
  <a href="https://www.npmjs.com/package/plasmate">npm</a> &middot;
  <a href="https://pypi.org/project/plasmate/">PyPI</a>
</p>

<p align="center">
  <img src="https://github.com/plasmate-labs/plasmate/actions/workflows/release.yml/badge.svg" alt="CI" />
  <img src="https://img.shields.io/crates/v/plasmate" alt="crates.io" />
  <img src="https://img.shields.io/npm/v/plasmate" alt="npm" />
  <img src="https://img.shields.io/badge/integrations-60%2B-brightgreen" alt="60+ Integrations" />
  <img src="https://img.shields.io/badge/license-Apache--2.0-blue" alt="License" />
</p>

---

Plasmate compiles HTML into a **Semantic Object Model (SOM)**, a structured representation that LLMs can reason about directly. It runs JavaScript via V8, supports Puppeteer via CDP, and produces output that is 10-800x smaller than raw HTML.

| | Plasmate | Lightpanda | Chrome |
|---|---|---|---|
| **Per page** | **4-5 ms** | 23 ms | 252 ms |
| **Memory (100 pages)** | **~30 MB** | ~2.4 GB | ~20 GB |
| **Binary** | **43 MB** | 59-111 MB | 300-500 MB |
| **Output** | **SOM (10-800x smaller)** | Raw HTML | Raw HTML |
| **License** | **Apache-2.0** | AGPL-3.0 | Chromium |

## Install

```bash
curl -fsSL https://plasmate.app/install.sh | sh
```

Or via package managers:

```bash
cargo install plasmate       # Rust
npm install -g plasmate      # Node.js
pip install plasmate         # Python
```

## Quick Start

### Fetch a page and get structured output

```bash
plasmate fetch https://news.ycombinator.com
```

Returns SOM JSON: structured regions, interactive elements with stable IDs, and content, typically 10x smaller than the raw HTML.

### Start a CDP server (Puppeteer compatible)

```bash
plasmate serve --protocol cdp --host 127.0.0.1 --port 9222
```

Then connect with Puppeteer:

```javascript
import puppeteer from 'puppeteer-core';

const browser = await puppeteer.connect({
  browserWSEndpoint: 'ws://127.0.0.1:9222',
  protocolTimeout: 10000,
});

const page = await browser.newPage();
await page.goto('https://example.com');

const title = await page.evaluate(() => document.title);
console.log(title);

await browser.close();
```

### Start an AWP server (native protocol)

```bash
plasmate serve --protocol awp --host 127.0.0.1 --port 9222
```

AWP has 7 methods: `navigate`, `snapshot`, `click`, `type`, `scroll`, `select`, `extract`. That's the entire protocol.

### Run as an MCP tool server (Model Context Protocol)

```bash
plasmate mcp
```

This exposes Plasmate over stdio as MCP tools:
- `fetch_page` - get structured SOM from any URL
- `extract_text` - get clean readable text
- `open_page` - start an interactive session (returns session_id + SOM)
- `evaluate` - run JavaScript in the page context
- `click` - click elements by SOM element ID
- `close_page` - end a session

Example Claude Desktop config:

```json
{
  "mcpServers": {
    "plasmate": {
      "command": "plasmate",
      "args": ["mcp"]
    }
  }
}
```

## For AI Agents

Plasmate is purpose-built for AI agent pipelines. Several ways to wire it in:

### MCP (Claude Desktop, Cursor, VS Code Copilot, Windsurf)

Add to your MCP config and every tool call automatically uses Plasmate:

```json
{
  "mcpServers": {
    "plasmate": {
      "command": "plasmate",
      "args": ["mcp"]
    }
  }
}
```

Config file locations:
- **Claude Desktop** — `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS)
- **Cursor** — `~/.cursor/mcp.json`
- **VS Code Copilot** — `.vscode/mcp.json` (workspace) or user settings
- **Windsurf** — `~/.codeium/windsurf/mcp_config.json`

Once connected, 16 tools are available: `fetch_page`, `extract_text`, `extract_links`, `open_page`, `navigate_to`, `click`, `type_text`, `select_option`, `scroll`, `toggle`, `clear`, `evaluate`, `close_page`, `get_cookies`, `set_cookies`, `clear_cookies`.

**Tip:** use `selector="main"` on any fetch to strip nav/footer before the LLM sees the content.

### Vercel AI SDK

Use Plasmate via the AI SDK's built-in MCP client (AI SDK v4+):

```bash
npm install ai @ai-sdk/openai
```

```ts
import { experimental_createMCPClient as createMCPClient, generateText } from 'ai'
import { Experimental_StdioMCPTransport as StdioMCPTransport } from 'ai/mcp-stdio'
import { openai } from '@ai-sdk/openai'

const mcp = await createMCPClient({
  transport: new StdioMCPTransport({
    command: 'plasmate',
    args: ['mcp'],
  }),
})

const { text } = await generateText({
  model: openai('gpt-4o'),
  tools: await mcp.tools(),
  maxSteps: 5,
  prompt: 'Summarize the top 3 stories on news.ycombinator.com',
})

await mcp.close()
```

This wires all 13 Plasmate tools directly into any Vercel AI SDK agent. See [Vercel AI SDK MCP docs](https://ai-sdk.dev/docs/ai-sdk-core/tools-and-tool-calling#mcp-tools) for details.

### LLM context

- Machine-readable summary: [`https://plasmate.app/llms.txt`](https://plasmate.app/llms.txt)
- Codebase guide for AI coding agents: [`AGENTS.md`](./AGENTS.md)
- Listed on [MCP Registry](https://registry.modelcontextprotocol.io) as the first browser/web tool


## What is SOM?

The DOM was built for rendering. SOM was built for reasoning.

```
Wikipedia homepage:
  DOM  → 47,000 tokens
  SOM  → 4,500 tokens (10.4x compression)

accounts.google.com:
  DOM  → ~300,000 tokens
  SOM  → ~350 tokens (864x compression)
```

SOM strips layout, styling, scripts, SVGs, and boilerplate. It keeps structure, content, and interactive elements with stable IDs that agents can reference in actions.

## Token Compression (38-site benchmark)

| Site | HTML | SOM | Compression |
|---|---|---|---|
| accounts.google.com | 1.2 MB | 1.4 KB | **864x** |
| x.com | 239 KB | 1.5 KB | **159x** |
| linear.app | 2.2 MB | 21 KB | **105x** |
| bing.com | 157 KB | 1.7 KB | **93x** |
| google.com | 194 KB | 2.6 KB | **74x** |
| vercel.com | 941 KB | 22 KB | **43x** |
| ebay.com | 831 KB | 33 KB | **25x** |
| Wikipedia | 1.7 MB | 70 KB | **25x** |

Median compression: **10.2x** across 38 sites. [Full results](https://plasmate.app/compare).

## JavaScript Support

Plasmate embeds V8 and executes page JavaScript, including:

- Inline and external `<script>` tags
- `fetch()` and `XMLHttpRequest` with real HTTP requests
- `setTimeout` / `setInterval` with timer draining
- DOM mutations (createElement, appendChild, textContent, innerHTML, etc.)
- DOMContentLoaded and load events
- Promise resolution and microtask pumping

The JS pipeline runs during `plasmate fetch` and CDP `page.goto()`. The resulting DOM mutations are serialized back to HTML before SOM compilation, so JS-rendered content is captured.

## CDP Compatibility

Plasmate passes [Lightpanda's Puppeteer benchmark](https://github.com/lightpanda-io/demo) (campfire-commerce). Supported CDP methods:

- `page.goto()`, `page.content()`, `page.title()`
- `page.evaluate()`, `page.waitForFunction()`
- `browser.newPage()`, `browser.createBrowserContext()`
- `Runtime.evaluate`, `Runtime.callFunctionOn`
- `DOM.getDocument`, `DOM.querySelector`, `DOM.querySelectorAll`
- `Input.dispatchMouseEvent`, `Input.dispatchKeyEvent`
- Target management (create, attach, close)

CDP is a compatibility layer. AWP is the native protocol, designed for agents rather than debuggers.

## Architecture

```
HTML → Network (reqwest) → HTML Parser (html5ever)
  → JS Pipeline (V8: scripts, fetch, XHR, timers, DOM mutations)
    → DOM Serialization → SOM Compiler → JSON output
```

- **Network**: reqwest with TLS, HTTP/2, redirects, compression; cookie jar supported, cookie APIs and proxy configuration are still limited
- **JS Runtime**: V8 with DOM shim (80+ methods), blocking fetch bridge
- **SOM Compiler**: semantic region detection, element ID generation, interactive element preservation, smart truncation, deduplication
- **Protocols**: AWP (native, 7 methods) and CDP (Puppeteer compatibility)

## Build from Source

```bash
git clone https://github.com/plasmate-labs/plasmate.git
cd plasmate
cargo build --release
./target/release/plasmate fetch https://example.com
```

Requirements: Rust 1.75+, V8 (fetched automatically by rusty_v8).

## Docker

Prebuilt multi-arch images (linux/amd64 and linux/arm64) are published to GHCR:

```bash
# Server mode (CDP or AWP)
docker run --rm -p 9222:9222 ghcr.io/plasmate-labs/plasmate:latest

# One-shot fetch
docker run --rm ghcr.io/plasmate-labs/plasmate:latest fetch https://example.com
```

Build locally:

```bash
docker build -t plasmate .
docker run --rm -p 9222:9222 plasmate
```

## Tests

```bash
cargo test --workspace    # 252 tests
```

## Benchmarks

Run the built-in benchmark against cached pages:

```bash
cargo run --release -- bench --urls bench/urls.txt
```

Or test against live sites:

```bash
plasmate fetch https://en.wikipedia.org/wiki/Rust_(programming_language) | jq '.regions | length'
```

See [plasmate.app/compare](https://plasmate.app/compare) for the full comparison with Lightpanda and Chrome.

## Roadmap

- [x] MCP server mode (`plasmate mcp` over stdio)
- [x] MCP Phase 2: stateful tools (open_page, click, evaluate, close_page)
- [x] Docker image (GHCR multi-arch)
- [ ] Full V8 DOM mutation bridge (re-snapshot SOM after JS changes)
- [ ] Network interception (Fetch domain)
- [x] AWP cookie APIs (session.cookies.get/set/clear)
- [x] MCP cookie tools (get_cookies, set_cookies, clear_cookies)
- [x] AWP SOM mutations (page.observe mode=mutations returns diffs instead of full snapshots)
- [x] Proxy support (HTTP, HTTPS, SOCKS5 with auth)
- [ ] Real-world top-100 site coverage testing
- [ ] Web Platform Tests integration

## Ecosystem

Plasmate has **60+ integrations** across the AI and developer ecosystem:

| Category | Integrations |
|----------|--------------|
| **AI Frameworks** | [LangChain](https://github.com/plasmate-labs/langchain-plasmate), [LlamaIndex](https://github.com/plasmate-labs/llamaindex-plasmate), [CrewAI](https://github.com/plasmate-labs/crewai-plasmate), [AutoGen](https://github.com/plasmate-labs/autogen-plasmate), [Haystack](https://github.com/plasmate-labs/haystack-plasmate), [DSPy](https://github.com/plasmate-labs/dspy-plasmate), [Semantic Kernel](https://github.com/plasmate-labs/semantic-kernel-plasmate), [Vercel AI](https://github.com/plasmate-labs/plasmate/tree/master/integrations/vercel-ai) |
| **Visual Builders** | [Langflow](https://github.com/plasmate-labs/langflow-plasmate), [Flowise](https://github.com/plasmate-labs/flowise-plasmate), [Dify](https://github.com/plasmate-labs/dify-plasmate) |
| **Automation** | [n8n](https://github.com/plasmate-labs/n8n-nodes-plasmate), [Zapier](https://github.com/plasmate-labs/zapier-plasmate), [Make.com](https://github.com/plasmate-labs/make-plasmate), [Activepieces](https://github.com/plasmate-labs/activepieces-plasmate), [Temporal](https://github.com/plasmate-labs/temporal-plasmate) |
| **Web Scraping** | [Scrapy](https://github.com/plasmate-labs/scrapy-plasmate), [Crawl4AI](https://github.com/plasmate-labs/crawl4ai-plasmate), [Firecrawl](https://github.com/plasmate-labs/firecrawl-plasmate), [ScrapeGraphAI](https://github.com/plasmate-labs/scrapegraphai-plasmate) |
| **Databases** | [Supabase](https://github.com/plasmate-labs/supabase-plasmate), [Prisma](https://github.com/plasmate-labs/prisma-plasmate), [PlanetScale](https://github.com/plasmate-labs/planetscale-plasmate), [Airtable](https://github.com/plasmate-labs/airtable-plasmate) |
| **Developer Tools** | [VS Code](https://github.com/plasmate-labs/vscode-plasmate), [Cursor](https://github.com/plasmate-labs/cursor-plasmate), [Raycast](https://github.com/plasmate-labs/raycast-plasmate), [GitHub Copilot](https://github.com/plasmate-labs/copilot-plasmate) |
| **Self-Hosted LLMs** | [Open WebUI](https://github.com/plasmate-labs/openwebui-plasmate), [OpenAI GPT Actions](https://github.com/plasmate-labs/openai-gpt-plasmate) |

See [awesome-plasmate](https://github.com/plasmate-labs/awesome-plasmate) for the full list.

## License

Apache-2.0. See [LICENSE](LICENSE).

Built by [Plasmate Labs](https://plasmate.app).
