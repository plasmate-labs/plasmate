# Install Plasmate in 30 Seconds

One command. Your AI gets a better browser.

## Claude Code

```bash
claude mcp add plasmate -- npx plasmate-mcp
```

Done. Your Claude Code agent now has 13 web browsing tools.

## Cursor

```bash
npx plasmate-mcp --install cursor
```

This auto-detects your Cursor config path and adds Plasmate. Restart Cursor.

## Claude Desktop

```bash
npx plasmate-mcp --install claude-desktop
```

This adds the MCP config to `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or the equivalent on Linux/Windows. Restart Claude Desktop.

## Any MCP Client

If your client supports MCP stdio servers, add this to your config:

```json
{
  "mcpServers": {
    "plasmate": {
      "command": "npx",
      "args": ["plasmate-mcp"]
    }
  }
}
```

## Verify It Works

Ask your AI: "What are the top stories on Hacker News?"

It should use `fetch_page` and return structured content instead of raw HTML. If you see region names like `navigation` and `main` in the response, Plasmate is working.

## What Your AI Gets

| Tool | What it does |
|------|-------------|
| `fetch_page` | Fetch any URL as structured JSON (17x fewer tokens than HTML) |
| `extract_text` | Plain text only (for reading, not interacting) |
| `extract_links` | All links with region context |
| `open_page` | Start an interactive session |
| `navigate_to` | Go to a new URL in an existing session |
| `click` | Click buttons and links |
| `type_text` | Fill form fields |
| `select_option` | Choose dropdown options |
| `scroll` | Scroll the page |
| `toggle` | Check/uncheck boxes, expand details |
| `clear` | Clear input fields |
| `evaluate` | Run JavaScript |
| `close_page` | End session |

## What Changes for the User

Nothing visible changes. Your AI just gets better at reading the web:

- **Faster**: 17x fewer tokens means faster responses
- **Cheaper**: 75% less token cost on web pages
- **More accurate**: Structured input reduces hallucination
- **Interactive**: Can fill forms, click buttons, navigate multi-step workflows
