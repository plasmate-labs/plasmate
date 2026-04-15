---
title: "Plasmate vs Firecrawl: Web Scraping for AI Agents Compared"
description: "Compare Plasmate and Firecrawl for LLM-ready web scraping. See how SOM compression, local deployment, and MCP support stack up against Firecrawl's hosted markdown API."
---

# Plasmate vs Firecrawl

Comparing two approaches to making web content digestible for AI agents.

## What Each Tool Does

**Plasmate** is a browser engine purpose-built for AI agents. It fetches web pages and compiles them into SOM (Semantic Object Model), a structured JSON format that captures meaning, not markup. Plasmate runs locally as a CLI, Docker container, or MCP server. Apache-2.0 open source.

**Firecrawl** is a hosted web scraping API that converts websites to LLM-ready markdown. It handles JavaScript rendering, crawling, and outputs clean markdown. Firecrawl offers a cloud API with usage-based pricing.

Both tools solve the same core problem: web pages are too noisy for LLMs. They diverge in how they solve it.

---

## Feature Comparison

| Feature | Plasmate | Firecrawl |
|---------|----------|-----------|
| **Output format** | SOM (structured JSON) | Markdown |
| **Compression ratio** | 10-800x vs raw HTML | ~5-10x typical |
| **Deployment** | Local CLI, Docker, self-hosted | Cloud API |
| **JavaScript execution** | Yes (V8 engine) | Yes |
| **Pricing** | Free (open source) | API pricing (usage-based) |
| **Protocol support** | MCP, CDP, AWP | REST API |
| **License** | Apache-2.0 | Proprietary |
| **Structured data** | Built-in (JSON-LD, forms, actions) | Markdown text |
| **Action annotations** | Yes (click, type, select) | No |
| **Self-hosted option** | Yes (native) | Enterprise only |

---

## Output Format: SOM vs Markdown

The key architectural difference is output format.

**Firecrawl** produces markdown. Markdown is readable and works well for content extraction, but it's fundamentally unstructured. A link in markdown is just `[text](url)`. You don't know if it's navigation, a button, or a content link. Forms become plain text.

**Plasmate** produces SOM, a JSON structure with semantic roles, regions, and explicit action annotations:

```json
{
  "role": "link",
  "text": "Sign Up",
  "attrs": { "href": "/signup" },
  "actions": ["click"],
  "region": "navigation"
}
```

For agents that need to act on pages (clicking, filling forms, navigating), SOM provides the structured data that markdown cannot. For pure text extraction, markdown may suffice.

---

## Compression: Why It Matters

Token costs compound at scale. A 500KB HTML page costs ~125K tokens. At GPT-4 pricing ($10/M input), that's $1.25 per page.

**Plasmate** achieves 10-800x compression depending on the page:

| Site | Raw HTML | SOM Output | Compression |
|------|----------|------------|-------------|
| cloud.google.com | 1.9 MB | 16 KB | 117x |
| linear.app | 2.2 MB | 21 KB | 105x |
| reddit.com | 484 KB | 4.7 KB | 104x |
| vercel.com | 795 KB | 22 KB | 36x |
| Median (49 sites) | - | - | 10.5x |

**Firecrawl** reports ~5-10x typical compression (markdown vs HTML). Good, but an order of magnitude less than SOM on complex sites.

At 1M pages/month with GPT-4:
- Raw HTML: ~$1,000/month
- Firecrawl markdown: ~$100-200/month
- Plasmate SOM: ~$60/month

---

## Deployment Model

**Plasmate** runs locally by default:

```bash
# CLI
plasmate fetch https://example.com

# Docker
docker run -p 9222:9222 plasmate/browser

# MCP server (for Claude Code, Cursor, etc.)
plasmate mcp
```

No API keys. No rate limits. No data leaving your infrastructure. Latency is network fetch time only.

**Firecrawl** is API-first:

```bash
curl -X POST https://api.firecrawl.dev/v0/scrape \
  -H "Authorization: Bearer fc-..." \
  -d '{"url": "https://example.com"}'
```

Simpler to start, but adds API latency, requires credentials management, and means your URLs go through their servers.

---

## Protocol Support

**Plasmate** supports multiple protocols:
- **MCP** (Model Context Protocol): First-class integration with Claude Code, Cursor, and other MCP clients
- **CDP** (Chrome DevTools Protocol): Drop-in replacement for Puppeteer/Playwright workflows
- **AWP** (Agent Web Protocol): Purpose-built WebSocket protocol for agents

**Firecrawl** uses REST. Standard and well-understood, but no native integration with agent tooling.

---

## Code Examples

### Fetch a page and extract content

**Plasmate (CLI):**
```bash
plasmate fetch https://news.ycombinator.com
```

**Plasmate (Python with AWP):**
```python
import asyncio
import websockets
import json

async def fetch_page():
    async with websockets.connect("ws://127.0.0.1:9222") as ws:
        await ws.send(json.dumps({
            "id": 1,
            "method": "page.navigate",
            "params": {"url": "https://news.ycombinator.com"}
        }))
        result = json.loads(await ws.recv())
        return result["result"]["som"]

som = asyncio.run(fetch_page())
print(f"Title: {som['title']}")
print(f"Regions: {len(som['regions'])}")
```

**Firecrawl (Python):**
```python
import requests

response = requests.post(
    "https://api.firecrawl.dev/v0/scrape",
    headers={"Authorization": "Bearer fc-..."},
    json={"url": "https://news.ycombinator.com"}
)
result = response.json()
print(result["data"]["markdown"])
```

### With Claude Code (MCP)

**Plasmate:** Native MCP support. Claude Code can browse the web directly:

```bash
# Start MCP server
plasmate mcp

# In Claude Code, Plasmate tools are available automatically
```

**Firecrawl:** Requires custom MCP wrapper or manual API calls in code.

---

## When to Use Plasmate

- **Local-first workflows**: No API keys, no external dependencies, no data egress
- **Token-sensitive agents**: 10-800x compression vs markdown's 5-10x
- **Agent automation**: Structured actions (click, type, select) for tool-use agents
- **MCP integration**: First-class support for Claude Code, Cursor, and other MCP clients
- **Self-hosted requirements**: Run entirely on your infrastructure
- **High-volume scraping**: No rate limits, no per-page costs

## When to Use Firecrawl

- **Quick prototypes**: API is faster to integrate than running a local service
- **Managed service preference**: Let someone else handle infrastructure
- **Markdown output needed**: If your pipeline expects markdown specifically
- **Crawling features**: Firecrawl has built-in site crawling and sitemap handling
- **No local resources**: When you can't or don't want to run local processes

---

## Summary

Plasmate and Firecrawl take different approaches to the same problem.

Firecrawl is a hosted API that produces markdown. Quick to integrate, no infrastructure to manage, usage-based pricing.

Plasmate is a local tool that produces SOM. Deeper compression, structured output, self-hosted, open source. Better suited for agents that need to understand and act on page structure, not just read text.

If you're building token-sensitive agents, need MCP integration, or prefer local-first tools, Plasmate is the better fit.

---

## Get Started with Plasmate

```bash
# Install
curl -fsSL https://plasmate.app/install.sh | sh

# Fetch your first page
plasmate fetch https://example.com

# Start MCP server for Claude Code
plasmate mcp
```

[Read the docs](https://plasmate.app/docs/overview) | [View on GitHub](https://github.com/plasmate-labs/plasmate)
