# Quick Start

Get Plasmate running and produce your first SOM output in under 60 seconds.

## Install

### From the install script

```sh
curl -fsSL https://plasmate.app/install.sh | sh
```

### From GitHub Releases

Download the binary for your platform from [GitHub Releases](https://github.com/plasmate-labs/plasmate/releases):

- `plasmate-aarch64-macos` (Apple Silicon)
- `plasmate-x86_64-macos` (Intel Mac)
- `plasmate-x86_64-linux` (Linux x86_64)
- `plasmate-aarch64-linux` (Linux ARM64)

Make it executable and move it to your PATH:

```sh
chmod +x plasmate-aarch64-macos
mv plasmate-aarch64-macos /usr/local/bin/plasmate
```

### From Docker

```sh
docker run -p 9222:9222 plasmate/browser
```

### From Source

```sh
git clone https://github.com/plasmate-labs/plasmate.git
cd plasmate
cargo build --release
./target/release/plasmate --help
```

## Usage

### One-shot fetch (CLI)

Fetch a page and output its SOM representation:

```sh
plasmate fetch https://news.ycombinator.com
```

This prints the Semantic Object Model to stdout: title, regions, elements, compression ratio, and timing.

### Persistent server (AWP)

Start the AWP WebSocket server:

```sh
plasmate serve --protocol awp
```

Connect on `ws://127.0.0.1:9222`. The server accepts AWP messages (JSON over WebSocket).

### Persistent server (CDP compatibility)

Start with Chrome DevTools Protocol compatibility for Puppeteer and Playwright:

```sh
plasmate serve --protocol cdp
```

Connect with Puppeteer:

```js
import puppeteer from 'puppeteer-core';

const browser = await puppeteer.connect({
  browserWSEndpoint: 'ws://127.0.0.1:9222/devtools/browser/plasmate',
  protocolTimeout: 10000,
});

const page = await browser.newPage();
await page.goto('https://news.ycombinator.com');
console.log(await page.title());
```

### Benchmark mode

Run the built-in throughput benchmark against a set of local pages:

```sh
plasmate throughput-bench --base-url http://localhost:8765 --pages 100
```

## AWP Protocol (Quick Reference)

AWP v0.1 has 7 methods:

| Method | Description |
|--------|-------------|
| `awp.hello` | Handshake and capability negotiation |
| `session.create` | Create a new browsing session |
| `session.close` | Close and clean up a session |
| `page.navigate` | Navigate to a URL, returns SOM |
| `page.observe` | Get the current SOM snapshot |
| `page.act` | Perform an action (click, type) |
| `page.extract` | Extract structured data, interactive elements, or specific fields |

### Example AWP session (Python pseudocode)

```python
import asyncio, websockets, json

async def main():
    async with websockets.connect("ws://127.0.0.1:9222") as ws:
        # Handshake
        await ws.send(json.dumps({
            "id": 1,
            "method": "awp.hello",
            "params": {"client": "my-agent", "version": "0.1"}
        }))
        print(await ws.recv())

        # Create session
        await ws.send(json.dumps({
            "id": 2,
            "method": "session.create",
            "params": {"user_agent": "MyAgent/1.0"}
        }))
        resp = json.loads(await ws.recv())
        session_id = resp["result"]["session_id"]

        # Navigate
        await ws.send(json.dumps({
            "id": 3,
            "method": "page.navigate",
            "params": {"session_id": session_id, "url": "https://news.ycombinator.com"}
        }))
        nav = json.loads(await ws.recv())
        print(f"Title: {nav['result']['title']}")
        print(f"Regions: {len(nav['result']['som']['regions'])}")

asyncio.run(main())
```

## SOM Output Structure

A SOM snapshot contains:

```json
{
  "version": "0.1",
  "url": "https://news.ycombinator.com",
  "title": "Hacker News",
  "meta": {
    "html_bytes": 34430,
    "som_bytes": 5200,
    "element_count": 45,
    "compression_ratio": 6.6
  },
  "regions": [
    {
      "id": "r_navigation",
      "role": "Navigation",
      "elements": [
        {
          "id": "e_a1b2c3d4e5f6",
          "role": "link",
          "name": "Hacker News",
          "href": "https://news.ycombinator.com"
        }
      ]
    },
    {
      "id": "r_main",
      "role": "Main",
      "elements": [...]
    }
  ]
}
```

Element IDs are deterministic: `e_` + first 12 hex chars of `sha256(origin|role|name|dom_path)`.

## What's Next

- Read the full [Product Spec](/docs/spec) for architecture details
- Read the [AWP Protocol](/docs/awp) specification
- Check the [Roadmap](/docs/roadmap) for what's coming in v0.2
