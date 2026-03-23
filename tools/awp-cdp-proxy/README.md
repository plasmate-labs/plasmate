# AWP-to-CDP Proxy

A reference implementation of the [Agent Web Protocol (AWP)](https://plasmate.app/docs/awp) that works with any Chrome/Chromium browser via the Chrome DevTools Protocol (CDP).

## Why This Exists

If only Plasmate speaks AWP, it is a proprietary protocol. This proxy proves AWP is a real open standard by providing a **second, independent implementation** that any AWP client can connect to.

```
AWP Client  <-->  awp-cdp-proxy  <-->  Chrome (CDP)
   (agent)       (this tool)        (any browser)
```

Any agent that speaks AWP can use this proxy to control a real browser -- no Plasmate required.

## Quick Start

```bash
# 1. Start Chrome with remote debugging
google-chrome --remote-debugging-port=9222
# or on macOS:
/Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome --remote-debugging-port=9222

# 2. Install and start the proxy
pip install -e .
awp-cdp-proxy --cdp-url ws://localhost:9222

# 3. Connect any AWP client to ws://localhost:9333
```

## Supported AWP Methods (v0.1)

| Method | Description |
|---|---|
| `awp.hello` | Handshake and version negotiation |
| `session.create` | Create a browser session (opens a new Chrome tab) |
| `session.close` | Close a session (closes the tab) |
| `page.navigate` | Navigate to a URL, get SOM snapshot |
| `page.observe` | Return the current page's SOM |
| `page.act` | Execute actions: click, type, select, scroll |
| `page.extract` | Extract structured data from the SOM |

## Architecture

```
                     AWP-CDP Proxy
                    +-----------------+
AWP Client -------->| server.py       |   AWP WebSocket (port 9333)
                    |   |             |
                    |   v             |
                    | translator.py   |   AWP method -> CDP command mapping
                    |   |             |
                    |   v             |
                    | cdp_client.py   |   CDP WebSocket -> Chrome
                    |   |             |
                    |   v             |
                    | som_generator.py|   HTML -> SOM conversion
                    +-----------------+
                          |
                          v
                    Chrome / Chromium
                    (any CDP browser)
```

### Components

- **server.py** -- AWP WebSocket server. Accepts client connections, dispatches to translator.
- **translator.py** -- Core translation logic. Maps AWP methods to sequences of CDP commands.
- **cdp_client.py** -- Minimal CDP WebSocket client. Sends commands, receives responses/events.
- **som_generator.py** -- Converts raw HTML into SOM using Python's html.parser and the `som-parser` Pydantic types.

## This Proxy vs Plasmate Native AWP

| Feature | awp-cdp-proxy | Plasmate |
|---|---|---|
| Backend | Chrome via CDP | Custom Rust engine |
| JavaScript | Full (Chrome's V8) | Not in v0.1 |
| SOM quality | Good (HTML heuristics) | Best (html5ever + Rust) |
| Performance | Depends on Chrome | Optimized, no browser overhead |
| Dependencies | Chrome + Python | Single binary |
| Use case | Reference impl, testing | Production agent browsing |

The proxy is intentionally simpler. Its purpose is to prove AWP works as a standard, not to replace Plasmate.

## Configuration

```bash
awp-cdp-proxy --help
```

| Flag | Default | Description |
|---|---|---|
| `--cdp-url` | `ws://localhost:9222` | Chrome DevTools Protocol URL |
| `--host` | `127.0.0.1` | Host to bind the AWP server |
| `--port` | `9333` | Port for the AWP server |
| `--verbose` | off | Enable debug logging |

## Development

```bash
# Install with dev dependencies
pip install -e ".[dev]"

# Run tests
pytest -v

# Run with verbose logging
awp-cdp-proxy --verbose
```

## Example: Python Client

```python
import asyncio
import json
import websockets

async def main():
    async with websockets.connect("ws://localhost:9333") as ws:
        # Handshake
        await ws.send(json.dumps({
            "id": "1", "type": "request",
            "method": "awp.hello",
            "params": {"client_name": "demo", "awp_version": "0.1"}
        }))
        print(json.loads(await ws.recv()))

        # Create session
        await ws.send(json.dumps({
            "id": "2", "type": "request",
            "method": "session.create", "params": {}
        }))
        session = json.loads(await ws.recv())
        sid = session["result"]["session_id"]

        # Navigate
        await ws.send(json.dumps({
            "id": "3", "type": "request",
            "method": "page.navigate",
            "params": {"session_id": sid, "url": "https://example.com"}
        }))
        print(json.loads(await ws.recv()))

        # Observe (get SOM)
        await ws.send(json.dumps({
            "id": "4", "type": "request",
            "method": "page.observe",
            "params": {"session_id": sid}
        }))
        som = json.loads(await ws.recv())
        print(json.dumps(som["result"]["som"]["meta"], indent=2))

asyncio.run(main())
```

## Links

- [AWP Specification](https://plasmate.app/docs/awp)
- [AWP v0.1 MVP Spec](https://plasmate.app/docs/awp-mvp)
- [SOM Specification](https://plasmate.app/docs/som-spec)
- [Plasmate](https://plasmate.app)
- [som-parser (Python)](https://github.com/plasmate-labs/plasmate/tree/master/packages/som-parser-python)

## License

Apache-2.0
