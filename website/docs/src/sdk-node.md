# Node.js SDK

The official Node.js SDK for Plasmate, with full TypeScript types and SOM query helpers.

## Installation

```sh
npm install plasmate
```

Requires Node.js 18+ and the `plasmate` binary on your PATH.

## Quick Start

```typescript
import { Plasmate } from 'plasmate';

const client = new Plasmate();

// Fetch a page and get SOM
const som = await client.som('https://example.com');
console.log(som.title);       // "Example Domain"
console.log(som.regions);     // semantic regions
console.log(som.meta);        // compression stats

client.close();
```

## TypeScript Types

### `Som`

The top-level SOM document.

```typescript
interface Som {
  som_version: string;
  url: string;
  title: string;
  lang: string;
  regions: SomRegion[];
  meta: SomMeta;
  structured_data?: StructuredData;
}
```

### `SomRegion`

A semantic region of the page (navigation, main content, sidebar, etc.).

```typescript
interface SomRegion {
  id: string;
  role: RegionRole;   // 'navigation' | 'main' | 'aside' | 'header' | 'footer' | 'form' | 'dialog' | 'content'
  label?: string;
  action?: string;
  method?: string;
  elements: SomElement[];
}
```

### `SomElement`

An individual element within a region.

```typescript
interface SomElement {
  id: string;
  role: ElementRole;  // 'link' | 'button' | 'text_input' | 'textarea' | 'select' | 'checkbox' | 'radio' | 'heading' | 'image' | 'list' | 'table' | 'paragraph' | 'section' | 'separator'
  text?: string;
  label?: string;
  actions?: ElementAction[];
  attrs?: SomElementAttrs;
  children?: SomElement[];
  hints?: SemanticHint[];
}
```

### `SomMeta`

Compression and element statistics.

```typescript
interface SomMeta {
  html_bytes: number;
  som_bytes: number;
  element_count: number;
  interactive_count: number;
}
```

## Query Helpers

All query helpers accept a `Som` object and return matching elements or regions.

### `findByRole(som, role)`

Find all regions matching a given role.

```typescript
import { findByRole } from 'plasmate';

const navRegions = findByRole(som, 'navigation');
```

### `findById(som, id)`

Find a single element by its stable ID.

```typescript
import { findById } from 'plasmate';

const el = findById(som, 'login-btn');
if (el) console.log(el.text);
```

### `findByTag(som, tag)`

Find elements matching a tag/role string.

```typescript
import { findByTag } from 'plasmate';

const links = findByTag(som, 'link');
```

### `findInteractive(som)`

Return all interactive elements (links, buttons, inputs, selects, textareas, checkboxes, radios).

```typescript
import { findInteractive } from 'plasmate';

const interactive = findInteractive(som);
console.log(`${interactive.length} interactive elements`);
```

### `findByText(som, text)`

Find elements whose text content contains the given string (case-insensitive).

```typescript
import { findByText } from 'plasmate';

const matches = findByText(som, 'Sign in');
```

### `flatElements(som)`

Flatten all elements across all regions into a single array.

```typescript
import { flatElements } from 'plasmate';

const all = flatElements(som);
```

### `getTokenEstimate(som)`

Estimate the LLM token count for the SOM (heuristic: ~4 bytes per token).

```typescript
import { getTokenEstimate } from 'plasmate';

const tokens = getTokenEstimate(som);
console.log(`~${tokens} tokens`);
```

## Client API

### Constructor

```typescript
const client = new Plasmate({
  binary: 'plasmate',   // path to plasmate binary (default: 'plasmate')
  timeout: 30000,       // timeout in ms (default: 30000)
});
```

### Stateless Methods

```typescript
// Fetch a page and return SOM
const som = await client.som(url, options?);

// Alias for som()
const som = await client.fetchPage(url, options?);

// Extract plain text from a page
const text = await client.extractText(url, options?);
```

### Stateful Sessions

```typescript
// Open a persistent page session
const { sessionId, som } = await client.openPage(url);

// Execute JavaScript in the session
const result = await client.evaluate(sessionId, 'document.title');

// Click an element and get updated SOM
const updatedSom = await client.click(sessionId, 'login-btn');

// Close the session
await client.closePage(sessionId);
```

### Cleanup

```typescript
client.close();
```

The client communicates with the `plasmate mcp` subprocess over JSON-RPC 2.0 on stdio. It auto-starts the process on first call.
