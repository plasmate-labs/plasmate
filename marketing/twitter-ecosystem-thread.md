# Plasmate Ecosystem Growth - Twitter Thread

## Tweet 1 (Hook)

Plasmate now has 60+ integrations across the AI ecosystem.

The browser engine built for agents is becoming the default way to give LLMs web access.

Here's what the ecosystem looks like (thread)

**[Image: Plasmate logo with integration logos arranged around it - LangChain, Vercel AI, CrewAI, n8n, Scrapy, etc. arranged in a wheel pattern]**

---

## Tweet 2 (The Problem)

Why does this matter?

Raw HTML is destroying your token budgets:
- Wikipedia homepage: 47,000 tokens
- Google login page: ~300,000 tokens

Plasmate's SOM output:
- Wikipedia: 4,500 tokens (10x smaller)
- Google login: 350 tokens (864x smaller)

Same information. Fraction of the cost.

---

## Tweet 3 (AI Frameworks)

AI Frameworks (8 integrations):

- LangChain - drop-in document loaders and tools
- LlamaIndex - native data connectors
- CrewAI - web browsing for agent crews
- AutoGen - multi-agent web research
- Vercel AI SDK - one-liner MCP integration
- Haystack, DSPy, Semantic Kernel

All use the same SOM output format. Learn once, use everywhere.

**[Image: Code snippet showing LangChain PlasmateFetchTool - 5 lines of code to add web browsing]**

---

## Tweet 4 (Browser Automation)

Browser Automation (4 integrations):

- browser-use: 10x token reduction vs default DOM serializer
- Scrapy: spider middleware for SOM extraction
- Crawl4AI: structured scraping at scale
- Firecrawl: drop-in replacement for web research

The browser-use integration is a one-line change that cuts your Claude/GPT costs by 90%.

---

## Tweet 5 (No-Code/Low-Code)

No-code & Automation (7 integrations):

- n8n - native Plasmate node
- Zapier - web parsing actions
- Make.com - scenario components
- Langflow - visual agent builder
- Flowise - drag-and-drop chains
- Dify - workflow blocks
- Activepieces - automation pieces

Build web-aware AI workflows without writing code.

**[Image: n8n workflow canvas showing Plasmate node connected to OpenAI and Slack nodes]**

---

## Tweet 6 (Developer Tools)

Developer Tools (4 integrations):

- VS Code extension
- Cursor integration
- Raycast commands
- GitHub Copilot extension

MCP support means Claude Desktop, Cursor, Windsurf, and VS Code Copilot all work out of the box.

One config line:
```
"plasmate": { "command": "plasmate", "args": ["mcp"] }
```

---

## Tweet 7 (SDKs)

Official SDKs in every major language:

- Node.js (npm install plasmate)
- Python (pip install plasmate)
- Go (go get github.com/nickel-org/plasmate-go)
- Rust (cargo install plasmate)

Full TypeScript types, async/await, query helpers for traversing SOM documents.

All SDKs spawn `plasmate mcp` and communicate via JSON-RPC over stdio. Zero network config.

---

## Tweet 8 (Performance)

Performance that makes this practical:

- 4-5ms per page (vs 252ms Chrome)
- 30MB memory for 100 pages (vs 20GB Chrome)
- 43MB binary (vs 300-500MB Chrome)

You can run this in Lambda, in containers, on a $5 VPS.

No Chrome. No Playwright. No headaches.

---

## Tweet 9 (Call to Action)

Try it in 30 seconds:

```bash
curl -fsSL https://plasmate.app/install.sh | sh
plasmate fetch https://news.ycombinator.com | jq
```

Star the repo: github.com/plasmate-labs/plasmate

Full integration list: github.com/plasmate-labs/awesome-plasmate

We're building the browser engine for the agentic web. Join us.

**[Image: Terminal screenshot showing Plasmate fetch output - clean SOM JSON structure with regions, elements, and compression stats]**
