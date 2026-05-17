# Using Plasmate with Claude Desktop

Add Plasmate to your Claude Desktop MCP configuration for token-efficient web browsing.

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
| `fetch_page` | Fetch a URL and return the Semantic Object Model (SOM) - 17x fewer tokens than raw HTML |
| `extract_text` | Get clean, readable text from any web page |
| `extract_links` | Get all outbound URLs from a page (deduplicated) |
| `cache_status` | Inspect MCP SOM cache hits, misses, selector/effective-HTML entries, and avoided HTML work |
| `session_status` | Inspect active MCP browser sessions, loaded URLs, raw/effective HTML, SOM/node inventory, capacity, age, and idle time |
| `open_page` | Open a persistent browser session; reports `cache_restored` on validated page-state cache hits |
| `click` | Click elements on an open page |
| `type_text` | Type into form fields |
| `navigate_to` | Navigate to a new URL in an open session; reports `cache_restored` on validated page-state cache hits |
| `scroll` | Scroll the page |
| `screenshot` | Take a screenshot |
| `evaluate` | Run JavaScript on the page |

### Tips

**Use `selector` to cut tokens further:**
Ask Claude: "Fetch stripe.com/docs but only the main content, not the nav"
Claude will call: `fetch_page(url="https://stripe.com/docs", selector="main")`

**Available selectors:** `main`, `nav`, `header`, `footer`, `aside`, `content`, `form`, `dialog`, element roles such as `button`/`link`/`text_input`, action surfaces such as `interactive` and `action:click`, text/label selectors such as `text:Upgrade` or `label:Email`, test locators such as `test_id:save-action` or `[data-testid=save-action]`, or any HTML id like `#my-section`.

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
