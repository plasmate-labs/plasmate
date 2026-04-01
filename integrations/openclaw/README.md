# OpenClaw Integration

[OpenClaw](https://openclaw.ai) integration for [Plasmate](https://plasmate.app) — install as an OpenClaw skill to give your agent a fast, token-efficient browser engine backed by the Semantic Object Model (SOM).

## What this provides

- **`SKILL.md`** — OpenClaw skill file. Install once; your agent knows when and how to use Plasmate automatically.
- **`scripts/pf`** — Fetch wrapper that calls `plasmate fetch` and appends token-savings stats to a local JSONL log. Drop it on your PATH; replace `web_fetch` calls with `pf`.

## Installation

### 1. Install Plasmate

```bash
curl -fsSL https://plasmate.app/install.sh | sh
```

### 2. Install the OpenClaw skill

**Via ClawHub (recommended):**

```bash
clawhub install plasmate
```

**Manually:**

```bash
mkdir -p ~/.openclaw/skills/plasmate
cp SKILL.md ~/.openclaw/skills/plasmate/SKILL.md
cp -r scripts ~/.openclaw/skills/plasmate/scripts
```

### 3. Install the `pf` wrapper (optional but recommended)

```bash
cp scripts/pf /usr/local/bin/pf
chmod +x /usr/local/bin/pf
```

`pf` is a drop-in replacement for `web_fetch` that:
- Calls `plasmate fetch` under the hood
- Prints fetch time + estimated token savings to stderr
- Appends a JSON stat entry to `~/.plasmate/fetch-stats.jsonl` on every call

```bash
pf https://docs.stripe.com/api    # ~96% token savings vs raw HTML
pf https://github.com/rust-lang/rust
```

Override the stats log:

```bash
PF_STATS_LOG=/my/custom/path.jsonl pf https://example.com
```

## Token Savings

Real-world benchmark across 12 diverse sites (SOM vs raw HTML via curl):

| Site | Plasmate tokens | Raw HTML tokens | Savings |
|---|---|---|---|
| Vercel docs | 2,206 | 556,464 | **99.6%** |
| Stripe API docs | 12,699 | 301,604 | **95.8%** |
| Next.js docs | 15,350 | 198,307 | **92.3%** |
| Stack Overflow | 41,699 | 289,090 | **85.6%** |
| Wikipedia article | 25,448 | 147,538 | **82.8%** |
| GitHub repo | 17,869 | 91,994 | **80.6%** |
| TechCrunch | 32,798 | 104,446 | **68.6%** |
| MDN reference | 23,038 | 47,290 | **51.3%** |

**Total across 10 successful fetches: 1.56M tokens saved.**

Plasmate is most effective on SPAs, documentation sites, and content-heavy pages. For already-minimal HTML (Hacker News, simple static pages) raw fetch may be comparable.

## Usage in OpenClaw agents

Once the skill is installed, your agent will automatically prefer `pf` for web fetching. You can also invoke Plasmate tools directly:

```bash
# One-shot fetch (stateless)
pf https://example.com

# MCP server (for multi-step browsing)
plasmate mcp

# CDP server (Puppeteer/Playwright compatible)
plasmate serve --protocol cdp --port 9222

# With auth profile (for logged-in browsing)
pf --profile github.com https://github.com/notifications
```

## MCP Configuration

Plasmate's MCP server exposes 11 tools for stateful multi-step browsing:

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

Available tools: `fetch_page`, `extract_text`, `screenshot_page`, `open_page`, `navigate_to`, `click`, `type_text`, `select_option`, `scroll`, `evaluate`, `close_page`.

## Viewing stats

The `pf` wrapper logs every fetch to `~/.plasmate/fetch-stats.jsonl`. View a summary:

```bash
python3 - << 'EOF'
import json, os
log = os.path.expanduser("~/.plasmate/fetch-stats.jsonl")
entries = [json.loads(l) for l in open(log) if l.strip()]
n = len(entries)
saved = sum(e.get("tokens_saved_est", 0) for e in entries)
avg_ms = sum(e.get("plasmate_ms", 0) for e in entries) // n
print(f"{n} fetches | {saved:,} tokens saved | avg {avg_ms}ms per fetch")
EOF
```

## License

Apache 2.0 — same as Plasmate.
