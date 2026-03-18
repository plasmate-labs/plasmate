# plasmate

Agent-native headless browser for Node.js. HTML in, Semantic Object Model out.

## Install

```bash
npm install plasmate
```

Requires the `plasmate` binary in your PATH:

```bash
curl -fsSL https://plasmate.app/install.sh | sh
```

## Quick Start

```typescript
import { Plasmate } from 'plasmate';

const browser = new Plasmate();

// Fetch a page as a structured Semantic Object Model
const som = await browser.fetchPage('https://news.ycombinator.com');
console.log(`${som.title}: ${som.regions.length} regions`);

// Extract clean text only
const text = await browser.extractText('https://example.com');
console.log(text);

// Interactive browsing
const session = await browser.openPage('https://example.com');
console.log(session.sessionId, session.som.title);

const title = await browser.evaluate(session.sessionId, 'document.title');
console.log(title);

await browser.closePage(session.sessionId);

// Clean up
browser.close();
```

## API

### `new Plasmate(options?)`

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `binary` | `string` | `"plasmate"` | Path to the plasmate binary |
| `timeout` | `number` | `30000` | Response timeout in milliseconds |

### Stateless (one-shot)

- **`fetchPage(url, options?)`** - Returns SOM JSON
- **`extractText(url, options?)`** - Returns clean text

### Stateful (interactive sessions)

- **`openPage(url)`** - Returns `{ sessionId, som }`
- **`evaluate(sessionId, expression)`** - Run JS, get result
- **`click(sessionId, elementId)`** - Click element, get updated SOM
- **`closePage(sessionId)`** - Close session

### Lifecycle

- **`close()`** - Shut down the plasmate process

## How It Works

The SDK spawns `plasmate mcp` as a child process and communicates via JSON-RPC 2.0 over stdio. The plasmate binary handles HTML parsing, JavaScript execution (V8), and SOM compilation in Rust.

## License

Apache-2.0
