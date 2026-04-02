# MCP Server

> **Plasmate is listed on the [official MCP Registry](https://registry.modelcontextprotocol.io/).** Claude, Cursor, and other MCP clients can discover and install it automatically.

Plasmate's MCP (Model Context Protocol) server lets AI coding assistants browse the web through Plasmate -  structured SOM output instead of raw HTML.

Source: [`plasmate-mcp`](https://github.com/nicepkg/plasmate/tree/master/integrations/mcp)

## Supported Clients

| Client | Status |
|--------|--------|
| Claude Code | ✅ Supported |
| Claude Desktop | ✅ Supported |
| Cursor | ✅ Supported |
| Windsurf | ✅ Supported |
| Any MCP client | ✅ Supported |

## Setup

### Claude Code

```bash
claude mcp add plasmate -- npx plasmate-mcp
```

That's it. Claude Code will now have `plasmate_fetch`, `plasmate_navigate`, `plasmate_click`, and `plasmate_type` tools available.

### Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or `%APPDATA%\Claude\claude_desktop_config.json` (Windows):

```json
{
  "mcpServers": {
    "plasmate": {
      "command": "npx",
      "args": ["-y", "plasmate-mcp"]
    }
  }
}
```

Restart Claude Desktop after editing.

### Cursor

Open Settings → MCP Servers → Add:

```json
{
  "plasmate": {
    "command": "npx",
    "args": ["-y", "plasmate-mcp"]
  }
}
```

### Windsurf

Add to your Windsurf MCP config:

```json
{
  "mcpServers": {
    "plasmate": {
      "command": "npx",
      "args": ["-y", "plasmate-mcp"]
    }
  }
}
```

### Manual (stdio)

```bash
npx plasmate-mcp
```

The server communicates over stdin/stdout using the MCP JSON-RPC protocol.

## Available Tools

| Tool | Description |
|------|-------------|
| `plasmate_fetch` | Stateless page fetch -  returns SOM text |
| `plasmate_navigate` | Open URL in persistent session |
| `plasmate_click` | Click element by SOM index |
| `plasmate_type` | Type into input by SOM index |

### Example: Fetch a page

```
Use plasmate_fetch to get the contents of https://news.ycombinator.com
```

The LLM receives structured SOM output (~1,500 tokens) instead of raw HTML (~22,000 tokens).

### Example: Multi-step browsing

```
Navigate to https://github.com, search for "plasmate", and tell me the top result.
```

The agent uses `plasmate_navigate` → `plasmate_type` → `plasmate_click` in sequence.

## Configuration

Environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `PLASMATE_BINARY` | `plasmate` | Path to plasmate binary |
| `PLASMATE_TIMEOUT` | `30` | Request timeout in seconds |
| `PLASMATE_BUDGET` | -  | Optional SOM token budget |

## Why MCP + Plasmate?

- **10-16x fewer tokens** per page load → cheaper, faster agent loops
- **Structured output** → LLM understands page layout, not HTML soup
- **Interactive elements indexed** → click/type by `[N]` reference
- **No Chrome dependency** → works on headless servers, CI, containers
