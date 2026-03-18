# Plasmate MCP Server Specification

## Overview

`plasmate mcp` exposes Plasmate as an MCP (Model Context Protocol) tool server over stdio. Any MCP-compatible client (Claude Desktop, Cursor, Windsurf, OpenClaw, Continue, etc.) can discover and use Plasmate for web browsing, scraping, and interaction.

## Client Configuration

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

Docker:
```json
{
  "mcpServers": {
    "plasmate": {
      "command": "docker",
      "args": ["run", "--rm", "-i", "ghcr.io/plasmate-labs/plasmate:latest", "mcp"]
    }
  }
}
```

## Transport

- **Protocol**: MCP over stdio (JSON-RPC 2.0)
- **Input**: stdin (newline-delimited JSON)
- **Output**: stdout (newline-delimited JSON)
- **Lifecycle**: long-running process, one instance per client session

## Server Info

```json
{
  "name": "plasmate",
  "version": "0.1.0",
  "capabilities": {
    "tools": {}
  }
}
```

## Tools

### 1. `fetch_page`

Fetch a URL and return its Semantic Object Model.

**When to use**: Getting structured content from any web page. Best for reading articles, documentation, product pages, search results. Returns 10x fewer tokens than raw HTML.

```json
{
  "name": "fetch_page",
  "description": "Fetch a web page and return its Semantic Object Model (SOM) - a structured, token-efficient representation of the page content. Use this instead of raw HTML fetching for 10x token savings.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "url": {
        "type": "string",
        "description": "URL to fetch"
      },
      "budget": {
        "type": "integer",
        "description": "Maximum output tokens. SOM will be truncated to fit. Default: no limit."
      },
      "javascript": {
        "type": "boolean",
        "description": "Enable JavaScript execution for dynamic/SPA pages. Default: true."
      }
    },
    "required": ["url"]
  }
}
```

**Response**: SOM JSON with title, url, regions array, and meta stats.

### 2. `extract_text`

Fetch a URL and return clean text only (no structure).

**When to use**: When you just need the readable text content, not the page structure. Good for summarization, search, Q&A over page content.

```json
{
  "name": "extract_text",
  "description": "Fetch a web page and return only the clean, readable text content. No HTML, no structure - just the text a human would read.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "url": {
        "type": "string",
        "description": "URL to fetch"
      },
      "max_chars": {
        "type": "integer",
        "description": "Maximum characters to return. Default: no limit."
      }
    },
    "required": ["url"]
  }
}
```

**Response**: Plain text string.

### 3. `open_page`

Open a page in a persistent browser session for multi-step interaction.

**When to use**: When you need to interact with a page (click buttons, fill forms, navigate). Creates a session that persists across calls.

```json
{
  "name": "open_page",
  "description": "Open a web page in a persistent browser session. Returns a session ID and the initial SOM. Use with click, type, and evaluate for multi-step interactions.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "url": {
        "type": "string",
        "description": "URL to open"
      }
    },
    "required": ["url"]
  }
}
```

**Response**:
```json
{
  "session_id": "abc123",
  "title": "Page Title",
  "url": "https://resolved.url",
  "regions": [...]
}
```

### 4. `click`

Click an interactive element in an open session.

```json
{
  "name": "click",
  "description": "Click an element on the page by its SOM element ID. Returns the updated page SOM after the click.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "session_id": {
        "type": "string",
        "description": "Session ID from open_page"
      },
      "element_id": {
        "type": "string",
        "description": "Element ID from SOM (e.g. 'e5')"
      }
    },
    "required": ["session_id", "element_id"]
  }
}
```

**Response**: Updated SOM after click.

### 5. `type_text`

Type text into an input field.

```json
{
  "name": "type_text",
  "description": "Type text into an input element on the page. Returns the updated page SOM.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "session_id": {
        "type": "string",
        "description": "Session ID from open_page"
      },
      "element_id": {
        "type": "string",
        "description": "Element ID of the input field from SOM"
      },
      "text": {
        "type": "string",
        "description": "Text to type"
      },
      "submit": {
        "type": "boolean",
        "description": "Press Enter after typing. Default: false."
      }
    },
    "required": ["session_id", "element_id", "text"]
  }
}
```

### 6. `evaluate`

Run JavaScript on the page and return the result.

**When to use**: When SOM doesn't capture what you need, or you need to run custom extraction logic.

```json
{
  "name": "evaluate",
  "description": "Execute JavaScript in the page context and return the result. Use for custom data extraction or page manipulation.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "session_id": {
        "type": "string",
        "description": "Session ID from open_page"
      },
      "expression": {
        "type": "string",
        "description": "JavaScript expression to evaluate. Return value is serialized to JSON."
      }
    },
    "required": ["session_id", "expression"]
  }
}
```

### 7. `screenshot`

Capture a screenshot of the current page.

```json
{
  "name": "screenshot",
  "description": "Take a screenshot of the page. Returns a base64-encoded PNG image.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "session_id": {
        "type": "string",
        "description": "Session ID from open_page"
      },
      "full_page": {
        "type": "boolean",
        "description": "Capture full scrollable page. Default: false (viewport only)."
      }
    },
    "required": ["session_id"]
  }
}
```

**Response**: Base64-encoded PNG as an MCP image content block.

### 8. `close_page`

Close a browser session.

```json
{
  "name": "close_page",
  "description": "Close a browser session and free resources.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "session_id": {
        "type": "string",
        "description": "Session ID to close"
      }
    },
    "required": ["session_id"]
  }
}
```

## Implementation Architecture

```
Agent (Claude, etc.)
  |
  | MCP JSON-RPC over stdio
  v
plasmate mcp (long-running process)
  |
  |-- fetch_page / extract_text: direct pipeline (HTML -> JS -> SOM)
  |-- open_page / click / type / evaluate: CDP session pool
  |      |
  |      v
  |   Internal CDP server (127.0.0.1:random_port)
  |      |
  |      v
  |   Page sessions (V8 + DOM per tab)
  |
  v
Network (reqwest)
```

### Stateless tools (fetch_page, extract_text)
- Run the standard Plasmate pipeline: fetch HTML, execute JS, compile SOM
- No session state, no CDP overhead
- Fastest path for one-shot reads

### Stateful tools (open_page, click, type, evaluate, screenshot)
- Spin up internal CDP server on first `open_page` call
- Each `open_page` creates a CDP target (browser tab)
- `click` and `type` dispatch CDP Input events
- `evaluate` runs Runtime.evaluate via CDP
- After each mutation, re-fetch page.content() and recompile SOM
- Sessions auto-close after 5 minutes of inactivity

### Session management
- Sessions stored in a HashMap<String, CdpSession>
- Session IDs are random UUIDs
- Max 10 concurrent sessions (configurable via --max-sessions)
- Idle timeout: 5 minutes (configurable via --session-timeout)

## CLI Flags

```
plasmate mcp [options]

Options:
  --max-sessions <n>      Max concurrent browser sessions (default: 10)
  --session-timeout <s>   Idle session timeout in seconds (default: 300)
  --no-javascript         Disable JS execution globally
  --budget <tokens>       Default token budget for fetch_page
```

## Error Handling

All errors return MCP error responses:

```json
{
  "isError": true,
  "content": [
    {
      "type": "text",
      "text": "Failed to fetch https://example.com: connection timed out"
    }
  ]
}
```

Error categories:
- Network errors (timeout, DNS, TLS)
- Session errors (invalid session_id, session expired)
- JavaScript errors (syntax error, runtime exception)
- Resource errors (max sessions reached)

## Future Extensions

### Resources (read-only data)
- `page://{session_id}` - current page SOM as a resource
- `page://{session_id}/text` - current page text

### Prompts (templates)
- `scrape` - guided multi-page scraping workflow
- `research` - open multiple pages, extract and compare

### Notifications
- `page_loaded` - when a page finishes loading
- `session_expired` - when a session times out

## Registry Listing

For Anthropic MCP Registry and Smithery:

```yaml
name: plasmate
description: Agent-native headless browser. Fetch web pages as structured Semantic Object Models with 10x token compression. Browse, interact, and extract data from any website.
category: web-browsing
tags:
  - browser
  - web-scraping
  - headless
  - semantic
  - dom
install:
  - curl -fsSL https://plasmate.app/install.sh | sh
  - cargo install plasmate
  - npm install -g plasmate
  - pip install plasmate
```

## Implementation Priority

1. **fetch_page** and **extract_text** - stateless, uses existing pipeline directly
2. **open_page** and **close_page** - session management scaffolding
3. **evaluate** - already works via CDP Runtime.evaluate
4. **click** and **type_text** - CDP Input dispatch (already implemented)
5. **screenshot** - requires layout/paint (not yet implemented; could return SOM as fallback)
