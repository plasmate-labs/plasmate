# OpenClaw Integration

[OpenClaw](https://openclaw.ai) integration for Plasmate — install as a skill to give any OpenClaw agent a fast, token-efficient browser engine.

Skill repo: [`plasmate-labs/skill-openclaw`](https://github.com/plasmate-labs/skill-openclaw)

## Installation

### 1. Install Plasmate

```bash
curl -fsSL https://plasmate.app/install.sh | sh
```

### 2. Install the skill

```bash
clawhub install plasmate
```

Or manually copy `integrations/openclaw/SKILL.md` to `~/.openclaw/skills/plasmate/SKILL.md`.

### 3. Install the `pf` wrapper

```bash
cp integrations/openclaw/scripts/pf /usr/local/bin/pf
chmod +x /usr/local/bin/pf
```

## Quick Start

Replace `web_fetch` calls with `pf`:

```bash
# Before
web_fetch https://docs.stripe.com/api

# After — ~96% fewer tokens, stats logged automatically
pf https://docs.stripe.com/api
```

`pf` wraps `plasmate fetch`, prints timing + token savings to stderr, and appends a stat entry to `~/.plasmate/fetch-stats.jsonl`.

## Token Savings

Real-world benchmark (SOM vs raw HTML, 12 sites):

| Site | Plasmate | Raw HTML | Savings |
|---|---|---|---|
| Vercel docs | 2,206 tok | 556,464 tok | **99.6%** |
| Stripe API | 12,699 tok | 301,604 tok | **95.8%** |
| Next.js docs | 15,350 tok | 198,307 tok | **92.3%** |
| Stack Overflow | 41,699 tok | 289,090 tok | **85.6%** |
| Wikipedia | 25,448 tok | 147,538 tok | **82.8%** |

**1.56M tokens saved across 10 test fetches.** Plasmate is most effective on SPAs and content-heavy pages.

## MCP Integration

For multi-step browsing, run Plasmate as an MCP server:

```bash
plasmate mcp
```

Add to your agent's MCP config:

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

Available MCP tools: `fetch_page`, `extract_text`, `screenshot_page`, `open_page`, `navigate_to`, `click`, `type_text`, `select_option`, `scroll`, `evaluate`, `close_page`.

## CDP Mode (Puppeteer-compatible)

Run Plasmate as a CDP server to replace Chrome in existing Puppeteer/Playwright workflows:

```bash
plasmate serve --protocol cdp --port 9222
export BROWSER_WS_ENDPOINT="ws://127.0.0.1:9222"
```

## Viewing Fetch Stats

The `pf` wrapper logs every fetch:

```bash
python3 - << 'EOF'
import json, os
log = os.path.expanduser("~/.plasmate/fetch-stats.jsonl")
entries = [json.loads(l) for l in open(log) if l.strip()]
n = len(entries)
saved = sum(e.get("tokens_saved_est", 0) for e in entries)
print(f"{n} fetches | {saved:,} tokens saved")
EOF
```

## Further Reading

- [MCP Integration](./integration-mcp.md) — detailed MCP tool reference
- [AWP Protocol](./awp-protocol.md) — native agent protocol
- [Authenticated Browsing](./auth.md) — cookie profiles for logged-in sites
