# Authenticated Browsing for AI Agents

Give your agent access to sites that require login, like X (Twitter), GitHub, LinkedIn, or any authenticated web app. Plasmate stores encrypted cookie profiles locally and injects them into requests so your agent can browse as you.

## How It Works

1. You log into a site normally in Chrome
2. The Plasmate browser extension captures the auth cookies
3. Plasmate encrypts and stores them locally on your machine
4. Your agent uses the stored profile to make authenticated requests

No passwords leave your machine. No tokens are sent to any server. Everything stays local.

---

## Prerequisites

- [Plasmate CLI installed](install)
- Google Chrome (or any Chromium-based browser)
- The Plasmate browser extension ([install from GitHub](https://github.com/plasmate-labs/plasmate-extension))

## Step 1: Install the Browser Extension

1. Download or clone the extension repo:
   ```bash
   git clone https://github.com/plasmate-labs/plasmate-extension.git
   ```
2. Open `chrome://extensions` in Chrome
3. Enable **Developer mode** (top right toggle)
4. Click **Load unpacked** and select the cloned folder
5. Pin the Plasmate extension icon to your toolbar

## Step 2: Start the Local Bridge

The bridge is a local HTTP server that the extension talks to. It only binds to `127.0.0.1` and never exposes anything to the network.

```bash
plasmate auth serve
```

You should see:

```
Auth bridge server listening on http://127.0.0.1:9271
```

Leave this running in a terminal tab.

## Step 3: Log Into the Target Site

Open Chrome and log into the site you want your agent to access. For this guide we will use **X (Twitter)** as the example:

1. Go to [x.com](https://x.com) and sign in normally
2. Browse around briefly to make sure the session is active

## Step 4: Push Cookies to Plasmate

1. Click the **Plasmate extension icon** in your toolbar
2. It auto-detects the current site and selects the relevant auth cookies
3. Click **Push to Plasmate**
4. A green confirmation appears. Done.

## Step 5: Verify

```bash
plasmate auth info x.com
```

You should see something like:

```
Profile: x.com
Cookies: 4
Created: 2026-03-26T12:00:00Z
Status:  encrypted
```

The cookie values are never printed. Only metadata is shown.

---

## Using the Profile

### Direct CLI

Fetch any authenticated page:

```bash
plasmate fetch https://x.com/home --profile x.com
```

The output is a SOM JSON document with the full authenticated page content, ready for your agent to consume.

### Via the Plasmate Server

Start the server with a profile loaded:

```bash
plasmate serve --protocol cdp --profile x.com
```

Now any agent that connects via CDP or AWP inherits the authenticated session.

---

## Agent-Specific Setup

### Claude Code (via MCP)

Add Plasmate as an MCP server in your Claude Code config (`~/.claude/mcp.json` or project `.mcp.json`):

```json
{
  "mcpServers": {
    "plasmate": {
      "command": "plasmate-mcp",
      "args": ["--profile", "x.com"]
    }
  }
}
```

Now Claude Code can call the `plasmate_fetch` tool and it will automatically use your X session:

```
Use plasmate_fetch to get https://x.com/home and summarize my timeline
```

### OpenClaw

If you are running OpenClaw, add Plasmate as a tool in your agent config:

```yaml
tools:
  plasmate:
    command: plasmate-mcp
    args: ["--profile", "x.com"]
```

Your agent can then call `plasmate_fetch` from any conversation to browse X as you.

### Cursor / Windsurf

Add to your MCP config (same format as Claude Code):

```json
{
  "mcpServers": {
    "plasmate": {
      "command": "plasmate-mcp",
      "args": ["--profile", "x.com"]
    }
  }
}
```

### Custom Agents (Python)

Use the Python SDK:

```python
from plasmate import Plasmate

browser = Plasmate(profile="x.com")
result = browser.fetch("https://x.com/home")
print(result["title"])
for region in result["regions"]:
    for el in region["elements"]:
        print(el.get("text", ""))
```

### Custom Agents (Node.js)

```javascript
import { Plasmate } from 'plasmate';

const browser = new Plasmate({ profile: 'x.com' });
const result = await browser.fetch('https://x.com/home');
console.log(result.title);
```

---

## Supported Sites

The extension auto-selects the right cookies for popular platforms:

| Site | Domain | Key Cookies |
|------|--------|-------------|
| X / Twitter | `x.com` | `auth_token`, `ct0` |
| GitHub | `github.com` | `user_session`, `__Host-user_session_same_site` |
| LinkedIn | `linkedin.com` | `li_at`, `JSESSIONID` |
| Reddit | `reddit.com` | `reddit_session`, `token_v2` |
| YouTube | `youtube.com` | `SID`, `HSID`, `SSID` |

For any other site, navigate there in Chrome, click the extension, and push. If you are unsure which cookies matter, push all of them.

## Managing Profiles

```bash
# List all stored profiles
plasmate auth list

# View profile metadata (never prints cookie values)
plasmate auth info github.com

# Delete a profile
plasmate auth revoke x.com
```

## Security Notes

- Cookie profiles are encrypted at rest using a machine-local key
- The bridge server binds to `127.0.0.1` only
- Cookie values are never logged, printed, or transmitted
- Profiles are stored in `~/.plasmate/auth/` (or `$PLASMATE_AUTH_DIR`)
- Use `plasmate auth revoke <domain>` to delete a profile at any time

---

## Next Steps

- [Build a Research Agent](tutorial-research-agent) that uses authenticated browsing
- [SOM Spec](som-spec) to understand the output format
- [MCP Integration](integration-mcp) for Claude Code and Cursor setup
