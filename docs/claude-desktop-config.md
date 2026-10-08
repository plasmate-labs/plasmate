# Using Plasmate with Claude Desktop

Add Plasmate to your Claude Desktop MCP configuration for structured web
browsing with page-dependent output size.

## Quick Setup

### 1. Install Plasmate

```bash
# Pick one:
cargo install plasmate       # Rust (fastest)
npm install -g plasmate      # Node
pip install plasmate          # Python
brew install plasmate         # macOS
```

### 2. Add to Claude Desktop config

**macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows:** `%APPDATA%\Claude\claude_desktop_config.json`

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

### 3. Restart Claude Desktop

Claude now has access to these tools:

| Tool | What it does |
|------|-------------|
| `fetch_page` | Fetch a URL and return the Semantic Object Model (SOM); output size is page- and selector-dependent |
| `extract_text` | Get clean, readable text from any web page |
| `extract_links` | Get all outbound URLs from a page (deduplicated) |
| `ard_discover` | Inspect bounded static ARD tool-discovery signals without invoking page tools |
| `crawl_policy` | Check a URL's robots.txt policy without changing fetch behavior |
| `inspect_page` | Return bounded SOM with an optional deterministic visual fallback |
| `cache_status` | Inspect MCP SOM cache hits, misses, selector/effective-HTML entries, and avoided HTML work |
| `session_status` | Inspect active MCP browser sessions, loaded URLs, raw/effective HTML, SOM/node inventory, disabled/readonly interactive counts, capacity, age, and idle time |
| `trace_status` | Inspect whether a session's bounded, memory-only action trace is enabled and retained |
| `trace_export` | Export redacted `plasmate.trace.v1` events for one live session |
| `trace_clear` | Clear retained trace events while preserving the monotonic sequence |
| `replay_validate` | Validate a retained action against current session state without executing it |
| `open_page` | Open a persistent browser session; reports `cache_restored` on validated page-state cache hits |
| `evaluate` | Run JavaScript on the page |
| `click` | Click elements on an open page |
| `type_text` | Type into form fields |
| `select_option` | Select an option from a dropdown or native radio group |
| `navigate_to` | Navigate to a new URL in an open session; reports `cache_restored` on validated page-state cache hits |
| `scroll` | Scroll the page |
| `toggle` | Toggle a checkbox, radio, or details element |
| `clear` | Clear an input or textarea |
| `close_page` | End a persistent browser session |
| `screenshot_page` | Take a screenshot |
| `get_cookies` | Read cookies from a browser session |
| `set_cookies` | Add or update cookies in a browser session |
| `clear_cookies` | Remove cookies from a browser session |

### Tips

**Use `selector` to cut tokens further:**
Ask Claude: "Fetch stripe.com/docs but only the main content, not the nav"
Claude will call: `fetch_page(url="https://stripe.com/docs", selector="main")`

**Available selectors:** `main`, `nav`, `header`, `footer`, `aside`, `content`, `form`, `dialog`, `h1` through `h6`, `interactive`, `action:click`, `action:type`, `action:clear`, `action:select`, `action:toggle`, `action:submit`, or any HTML id like `#my-section`.

## Using with Cursor

Same config — add to Cursor's MCP settings:

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
