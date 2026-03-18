# plasmate

Agent-native headless browser for Python. HTML in, Semantic Object Model out.

## Install

```bash
pip install plasmate
```

Requires the `plasmate` binary in your PATH:

```bash
curl -fsSL https://plasmate.app/install.sh | sh
```

## Quick Start

```python
from plasmate import Plasmate

browser = Plasmate()

# Fetch a page as a structured Semantic Object Model
som = browser.fetch_page("https://news.ycombinator.com")
print(f"{som['title']}: {len(som['regions'])} regions")

# Extract clean text only
text = browser.extract_text("https://example.com")
print(text)

# Interactive browsing
session = browser.open_page("https://example.com")
print(session["session_id"], session["som"]["title"])

title = browser.evaluate(session["session_id"], "document.title")
print(title)

browser.close_page(session["session_id"])
browser.close()
```

### Async

```python
from plasmate import AsyncPlasmate

async with AsyncPlasmate() as browser:
    som = await browser.fetch_page("https://example.com")
    print(som["title"])
```

### Context Manager

```python
with Plasmate() as browser:
    som = browser.fetch_page("https://example.com")
    # Process closes automatically
```

## API

### `Plasmate(binary="plasmate", timeout=30)`

| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `binary` | `str` | `"plasmate"` | Path to the plasmate binary |
| `timeout` | `float` | `30` | Response timeout in seconds |

### Stateless (one-shot)

- **`fetch_page(url, *, budget=None, javascript=True)`** - Returns SOM dict
- **`extract_text(url, *, max_chars=None)`** - Returns clean text string

### Stateful (interactive sessions)

- **`open_page(url)`** - Returns dict with `session_id` and `som`
- **`evaluate(session_id, expression)`** - Run JS, get result
- **`click(session_id, element_id)`** - Click element, get updated SOM
- **`close_page(session_id)`** - Close session

### Lifecycle

- **`close()`** - Shut down the plasmate process

## How It Works

The SDK spawns `plasmate mcp` as a child process and communicates via JSON-RPC 2.0 over stdio. The plasmate binary handles HTML parsing, JavaScript execution (V8), and SOM compilation in Rust.

## License

Apache-2.0
