---
name: plasmate
description: Browse the web via Plasmate, a fast headless browser engine for agents. Compiles HTML into a Semantic Object Model (SOM) — 50x faster than Chrome, 10x fewer tokens. Supports MCP, AWP (Agent Web Protocol), and CDP compatibility. Optional authenticated browsing uses locally encrypted cookie profiles that never leave the user's machine.
homepage: https://plasmate.app
metadata:
  {
    "openclaw":
      {
        "emoji": "⚡",
        "source": "https://github.com/plasmate-labs/plasmate",
        "license": "Apache-2.0",
        "privacy": "https://plasmate.app/privacy",
        "requires": { "bins": ["plasmate"] },
        "install":
          [
            {
              "id": "curl",
              "kind": "shell",
              "command": "curl -fsSL https://plasmate.app/install.sh | sh",
              "bins": ["plasmate"],
              "label": "Install Plasmate (shell script)",
            },
            {
              "id": "cargo",
              "kind": "shell",
              "command": "cargo install plasmate",
              "bins": ["plasmate"],
              "label": "Install Plasmate (cargo)",
            },
          ],
      },
  }
---

# Plasmate — Browser Engine for Agents

Plasmate compiles HTML into a Semantic Object Model (SOM): 50x faster than Chrome, 10x fewer tokens.

- **Docs**: https://docs.plasmate.app
- **Source**: https://github.com/plasmate-labs/plasmate (Apache 2.0, fully auditable)
- **Privacy**: https://plasmate.app/privacy

## Security and Privacy

Plasmate is open source (Apache 2.0) and runs entirely on the user's machine. No data is sent to Plasmate Labs or any third party.

- **No telemetry, analytics, or cloud services** — everything runs locally
- **Auth profiles are encrypted** with AES-256-GCM before being written to disk
- **Encryption key** at `~/.plasmate/master.key` (owner-only permissions, chmod 0600)
- **Cookie bridge listens only on localhost** (127.0.0.1:9271) — never network-accessible
- **Cookie sharing is always user-initiated** — agent cannot extract cookies on its own
- **Users can revoke any stored profile** at any time with `plasmate auth revoke <domain>`

## Install

```bash
# Install script (inspect source: https://plasmate.app/install.sh)
curl -fsSL https://plasmate.app/install.sh | sh

# Or build from source
git clone https://github.com/plasmate-labs/plasmate && cd plasmate && cargo build --release
```

## Quick Start

### `pf` — recommended fetch wrapper

The `pf` script wraps `plasmate fetch` with automatic token-savings logging. Copy it to your PATH from `integrations/openclaw/scripts/pf`:

```bash
cp integrations/openclaw/scripts/pf /usr/local/bin/pf
chmod +x /usr/local/bin/pf
```

Then use `pf` anywhere you would use `web_fetch`:

```bash
pf https://example.com          # returns SOM, logs stats to ~/.plasmate/fetch-stats.jsonl
pf https://docs.python.org/3/   # ~85% token savings vs raw HTML
```

Stats are appended to `~/.plasmate/fetch-stats.jsonl` on every call — useful for measuring real-world savings over time.

**Override the stats log path:**

```bash
PF_STATS_LOG=/path/to/stats.jsonl pf https://example.com
```

### Direct fetch (one-shot, no server)

```bash
plasmate fetch <url>
```

Returns SOM JSON: regions, interactive elements with stable IDs, extracted content.

### MCP Server (recommended for agent frameworks)

```bash
plasmate mcp
```

Starts an MCP server over stdio. Configure it in your agent's MCP settings:

```json
{
  "servers": {
    "plasmate": {
      "command": "plasmate",
      "args": ["mcp"],
      "transport": "stdio",
      "description": "Plasmate browser engine — SOM output, ~90% token savings vs Chrome"
    }
  }
}
```

Available MCP tools: `fetch_page`, `extract_text`, `screenshot_page`, `open_page`, `navigate_to`, `click`, `type_text`, `select_option`, `scroll`, `evaluate`, `close_page`.

### CDP Server (Puppeteer/Playwright compatible)

```bash
plasmate serve --protocol cdp --port 9222
```

Point any Puppeteer/Playwright client at `ws://127.0.0.1:9222` — Plasmate handles the request instead of Chrome.

### AWP Server (native protocol)

```bash
plasmate serve --protocol awp --port 9222

# Or both AWP + CDP on the same port
plasmate serve --protocol both --port 9222
```

## Protocols

| Protocol | Use when |
|---|---|
| **MCP** (stdio) | Native agent framework integration — cleaner than CDP for new code |
| **AWP** (native) | Performance-critical; direct protocol access |
| **CDP** (bridge) | Existing Puppeteer/Playwright code that needs reuse |

## AWP Usage (Python)

Use the included `awp-browse.py` helper for AWP interactions:

```bash
# Navigate and get SOM snapshot
python3 scripts/awp-browse.py navigate "https://example.com"

# Click an interactive element by ref ID
python3 scripts/awp-browse.py click "https://example.com" --ref "e12"

# Type into a field
python3 scripts/awp-browse.py type "https://example.com" --ref "e5" --text "search query"

# Extract structured data (JSON-LD, OpenGraph, tables)
python3 scripts/awp-browse.py extract "https://example.com"

# Scroll
python3 scripts/awp-browse.py scroll "https://example.com" --direction down
```

## Performance

| Metric | Plasmate | Chrome headless |
|---|---|---|
| Per page | ~4ms | ~252ms |
| Memory (100 pages) | ~30MB | ~20GB |
| Token output | SOM (10–800x smaller) | Raw HTML |

Real-world token savings from a 12-site benchmark:

| Site type | Savings |
|---|---|
| Complex SPA (Stripe, Vercel docs) | 95–99% |
| General docs (Next.js, GitHub) | 80–92% |
| Content/news sites | 68–85% |
| Simple/minimal HTML (HN, Python docs) | -15–0% (use raw fetch here) |

**When Plasmate helps most:** SPAs, heavy docs sites, content-rich pages.
**When raw fetch is better:** Already-minimal HTML (Hacker News, simple static pages).

## Authenticated Browsing (Optional)

Plasmate can optionally use stored cookie profiles to browse sites the user is logged into. Entirely opt-in and user-controlled.

### How it works

1. User logs into a site normally in their browser
2. User opens the Plasmate extension and clicks "Push to Plasmate"
3. Cookies are sent to the local bridge server on localhost only
4. Plasmate encrypts and stores them on disk (AES-256-GCM)
5. Agent fetches that site with the stored profile

### Using stored profiles

```bash
plasmate fetch --profile x.com https://x.com/home
plasmate serve --profile github.com --port 9222
```

### Managing profiles

```bash
plasmate auth list          # list stored profiles with expiry status
plasmate auth info x.com    # detailed info for a domain
plasmate auth revoke x.com  # delete a stored profile
```

### Agent-guided auth flow

When a user asks you to browse a site that requires login and no profile exists:

```bash
# Start the bridge (listens on 127.0.0.1:9271)
plasmate auth serve &

# Wait for user to share cookies (blocks up to 120s — no polling needed)
curl -s "http://127.0.0.1:9271/api/wait?domain=x.com&timeout=120"

# Browse authenticated
plasmate fetch --profile x.com https://x.com/notifications
```

Tell the user:
> "I need access to your [site] account. Please install the Plasmate extension, go to [site] while logged in, and click 'Push to Plasmate' in the toolbar."

## SOM Output Structure

SOM is structured JSON — not raw HTML.

- **regions**: Semantic page areas (nav, main, article, sidebar)
- **interactive**: Clickable/typeable elements with stable ref IDs (e.g., `e1`, `e12`)
- **content**: Text content organized by region
- **structured_data**: JSON-LD, OpenGraph, microdata extracted automatically

Use ref IDs from `interactive` elements for click/type/select actions.

## When to Use Plasmate vs Browser Tool

- **Plasmate (`pf` / `plasmate fetch` / MCP)**: Speed-critical scraping, batch processing, token-sensitive extraction, structured data, authenticated browsing, any URL where token count matters
- **Browser tool (Chrome)**: Visual rendering needed, screenshots, complex JS SPAs that Plasmate can't handle, pixel-level interaction
