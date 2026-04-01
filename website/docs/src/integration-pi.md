# Pi / oh-my-pi Integration

Plasmate integrates with both [Pi](https://github.com/badlogic/pi-mono) (the original coding agent by @mariozechner) and [oh-my-pi](https://github.com/can1357/oh-my-pi) (the extended fork with MCP, LSP, browser, and subagent support) via MCP.

Both agents discover Plasmate's 13 tools automatically through their MCP configuration.

## Quick Setup

### oh-my-pi (recommended)

oh-my-pi has native MCP support via its `mcp/` discovery layer. Add Plasmate to your MCP configuration:

```json
{
  "servers": {
    "plasmate": {
      "command": "plasmate",
      "args": ["mcp"],
      "transport": "stdio"
    }
  }
}
```

Place this in your oh-my-pi MCP config file (typically `~/.config/oh-my-pi/mcp.json` or passed via `--mcp-config`).

Once configured, oh-my-pi automatically discovers all 13 Plasmate tools:

| Tool | What it does |
|---|---|
| `fetch_page` | Fetch a URL, return SOM (structured JSON). Stateless. |
| `extract_text` | Fetch a URL, return plain text only. Stateless. |
| `screenshot_page` | Capture a screenshot. |
| `open_page` | Open a URL in a persistent session. Returns session ID + SOM. |
| `navigate_to` | Navigate an existing session to a new URL. |
| `click` | Click an element by SOM element ID. Returns updated SOM. |
| `type_text` | Type into a form input or textarea by element ID. |
| `select_option` | Set a dropdown value by element ID + option value. |
| `scroll` | Scroll the viewport or a specific element into view. |
| `toggle` | Toggle a checkbox, radio button, or details/summary widget. |
| `clear` | Clear the value of a text input or textarea. |
| `evaluate` | Run JavaScript in the page context. |
| `close_page` | Close a session and free resources. |

### Pi (original)

Pi supports MCP tools through its `--mcp` flag or configuration:

```bash
pi --mcp plasmate="plasmate mcp" "Research the pricing on stripe.com/pricing"
```

Or add to your Pi config file (`~/.config/pi/config.toml`):

```toml
[mcp.plasmate]
command = "plasmate"
args = ["mcp"]
```

## Why Plasmate with Pi?

Pi and oh-my-pi both have built-in web browsing via their `web/` module, which typically uses headless Chrome or simple HTTP fetch + readability extraction. Plasmate replaces this with structured SOM output that uses 4x fewer tokens.

### Default Pi web browsing

```
User: What are the pricing tiers on stripe.com?

Pi uses web_search + fetch → raw HTML or markdown
→ ~30,000 tokens of page content in context
→ Model reasons over noisy input
```

### With Plasmate

```
User: What are the pricing tiers on stripe.com?

Pi calls fetch_page via MCP → SOM JSON
→ ~8,000 tokens of structured content
→ Model receives typed regions, elements, and affordances
```

The token savings compound across multi-page research sessions. A 10-page research task that consumes 300,000 tokens with raw HTML uses approximately 80,000 tokens with SOM.

## Multi-step Interaction Example

oh-my-pi's subagent and task orchestration layer works well with Plasmate's stateful session tools. A multi-step workflow:

```
1. open_page("https://example.com/login")
   → Returns SOM with form fields: email input (e3), password input (e5), submit button (e7)

2. type_text(session_id, "e3", "user@example.com")
   → Returns updated SOM

3. type_text(session_id, "e5", "password123")
   → Returns updated SOM

4. click(session_id, "e7")
   → Submits form, navigates to dashboard, returns new SOM

5. navigate_to(session_id, "https://example.com/settings")
   → Returns settings page SOM with all interactive elements
```

Each step returns the full updated SOM, so Pi always has current page state in context without re-fetching.

## CDP Fallback

If you need pixel-perfect rendering or browser features that SOM does not cover (video playback, canvas interactions, WebRTC), Plasmate also exposes a CDP server:

```bash
plasmate serve --port 9222
```

Pi and oh-my-pi can connect via Puppeteer:

```javascript
const browser = await puppeteer.connect({
  browserWSEndpoint: 'ws://127.0.0.1:9222'
});
```

Use MCP for structured browsing (research, data extraction, form filling). Use CDP when you need the full browser.

## Authenticated Browsing

Plasmate supports cookie-based auth profiles for browsing sites the user is logged into. This works transparently with Pi:

```bash
# Set up an auth profile (one-time)
plasmate auth create twitter

# Pi automatically uses the profile when fetching authenticated pages
pi "What are the trending topics on my Twitter timeline?"
# → Plasmate serves authenticated SOM via the stored cookie profile
```

See the [Authenticated Browsing guide](/guide-authenticated-browsing) for setup details.

## Performance

| Metric | Pi + raw fetch | Pi + Plasmate |
|---|---|---|
| Tokens per page | ~33,000 | ~8,300 |
| Latency (warm) | 200-500ms | 200-400ms |
| Multi-page (10 pages) | ~330K tokens | ~83K tokens |
| Memory | Depends on browser | ~30MB / 100 pages |
| Form filling | Requires Playwright/CDP | Native (type_text, select_option, toggle) |
