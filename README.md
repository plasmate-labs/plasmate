# Plasmate

Agent-native headless browser engine with Semantic Object Model (SOM).

Plasmate replaces CDP with AWP (Agent Wire Protocol), replaces DOM dumps with a
token-efficient Semantic Object Model, and is purpose-built for LLM agent consumption.

## Architecture

```
┌─────────────────────────────────────────────────┐
│                   CLI / Server                   │
│           (fetch, serve, bench modes)            │
├────────────────────┬────────────────────────────┤
│    AWP Server      │      Bench Harness         │
│  (WebSocket)       │   (URL list → report)      │
│  ┌──────────┐      │                            │
│  │ Handler  │      │                            │
│  │ Session  │      │                            │
│  │ Messages │      │                            │
│  └──────────┘      │                            │
├────────────────────┴────────────────────────────┤
│                SOM Compiler                      │
│  HTML → parse → strip → regions → elements →    │
│  element IDs → collapse wrappers → JSON output   │
│  ┌──────────┐ ┌────────────┐ ┌──────────────┐  │
│  │  Types   │ │ Heuristics │ │  Element IDs  │  │
│  └──────────┘ └────────────┘ └──────────────┘  │
├─────────────────────────────────────────────────┤
│              Network (reqwest)                    │
│         HTTP GET + cookie jar + redirects        │
└─────────────────────────────────────────────────┘
```

## Build

```bash
cargo build --release
```

## Usage

### Fetch a URL and output SOM

```bash
# Print SOM JSON to stdout
cargo run -- fetch https://news.ycombinator.com

# Write SOM to file
cargo run -- fetch https://example.com --output som.json
```

### Start AWP WebSocket server

```bash
cargo run -- serve --host 127.0.0.1 --port 9222
```

Then connect with the Python client:

```bash
pip install websockets
python examples/plasmate_client.py
```

### Run benchmarks

```bash
cargo run -- bench --urls bench/urls.txt --output report.md
```

## AWP Protocol (v0.1)

Connect via WebSocket to `ws://host:port/` and send JSON messages:

| Method | Purpose |
|---|---|
| `awp.hello` | Handshake |
| `session.create` | Create browsing session |
| `session.close` | Destroy session |
| `page.navigate` | Fetch URL, compile SOM |
| `page.observe` | Get current SOM snapshot |
| `page.act` | Click, type, select actions |
| `page.extract` | Query structured data from SOM |

## SOM (Semantic Object Model)

SOM compiles raw HTML into a compact, deterministic JSON representation:

- Strips scripts, styles, comments, SVGs, hidden elements
- Identifies semantic regions (navigation, main, aside, header, footer, forms)
- Captures interactive elements with stable IDs
- Typically **8-30x smaller** than raw HTML

## Tests

```bash
cargo test
```

## Docker

```bash
docker build -t plasmate .
docker run -p 9222:9222 plasmate
```

## License

Apache-2.0
