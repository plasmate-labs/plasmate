---
name: plasmate
description: Browse the web using Plasmate for 16x fewer tokens than raw HTML. Use when fetching documentation, reading web pages, extracting structured data, or interacting with websites.
config:
  mcp:
    command: npx
    args: ["-y", "plasmate-mcp"]
---

# Plasmate Web Browsing

Browse web pages using Plasmate's Semantic Object Model (SOM) instead of raw HTML. SOM compresses web content by 16.6x on average while preserving page structure, element roles, and interactive affordances.

## When to use

- Reading documentation, API references, or READMEs
- Fetching web page content for research or analysis
- Extracting structured data from websites
- Interacting with web pages (clicking, typing, navigating)
- Any task where you need web content with minimal token usage

## Available tools

Once the MCP server is connected, you have these tools:

### fetch_page
Fetch any URL and get the Semantic Object Model (SOM). Returns structured JSON with semantic regions (navigation, main, footer), typed elements (links, buttons, headings, tables), and available actions.

```
fetch_page(url: "https://example.com")
```

### extract_text
Get clean, readable text from any URL. Use this when you only need the text content without structural metadata.

```
extract_text(url: "https://docs.example.com/api")
```

### open_page
Open a persistent browser session for multi-step interactions.

```
open_page(url: "https://example.com/login")
```

### click
Click an element by its SOM ID within an open page session.

```
click(session_id: "...", element_id: "btn-submit")
```

### evaluate
Execute JavaScript in an open page session.

```
evaluate(session_id: "...", expression: "document.title")
```

### close_page
Close a browser session when done.

```
close_page(session_id: "...")
```

## Setup

The skill auto-configures the MCP server. If you need to install Plasmate manually:

```bash
# Install the MCP wrapper
npm install -g plasmate-mcp

# Install the Plasmate engine (pick one)
brew tap plasmate-labs/plasmate && brew install plasmate
# or: cargo install plasmate
# or: pip install plasmate
```

Then add the MCP server:

```bash
claude mcp add plasmate -- npx plasmate-mcp
```

## Tips

- Prefer `extract_text` for simple reading tasks; it returns fewer tokens than `fetch_page`
- Use `fetch_page` when you need to understand page structure or find interactive elements
- Use `open_page` + `click` for multi-step workflows like form filling or navigation
- Set the `PLASMATE_CACHE_API_KEY` environment variable for faster cached lookups via the SOM Cache CDN

## Why Plasmate instead of raw HTML?

- **16.6x fewer tokens** on average across real websites
- **50x faster** than headless Chrome
- **30MB memory** vs Chrome's 300MB+
- **Structured output** with semantic roles, not tag soup

## Links

- [Plasmate](https://plasmate.app)
- [Documentation](https://docs.plasmate.app)
- [GitHub](https://github.com/plasmate-labs/plasmate)
- [SOM Spec](https://docs.plasmate.app/som-spec)
