# som-parser

Parse and query [SOM (Semantic Object Model)](https://plasmate.app/docs/som-spec) data in JavaScript and TypeScript. SOM is a structured JSON format that represents web pages as semantic elements (links, buttons, headings, forms, etc.) instead of raw HTML, making it easy for AI agents and automation tools to understand and interact with web content.

## Install

```bash
npm install som-parser
```

## Quick Start

### Parse Plasmate output

```typescript
import { fromPlasmate, parseSom } from 'som-parser';

// From Plasmate CLI output
const som = fromPlasmate(plasmateJsonOutput);

// From a JSON string or object
const som2 = parseSom('{"som_version": "0.1", ...}');
```

### Find all links on a page

```typescript
import { getLinks } from 'som-parser';

const links = getLinks(som);
// [{ text: 'Home', href: '/', id: 'e_1' }, ...]
```

### Get interactive elements

```typescript
import { getInteractiveElements } from 'som-parser';

const interactive = getInteractiveElements(som);
// All elements with actions (clickable, typeable, etc.)
```

### Convert to markdown

```typescript
import { toMarkdown } from 'som-parser';

const md = toMarkdown(som);
// # Page Title
// ## Welcome
// - [Home](/)
// This is a paragraph...
```

### Get compression ratio

```typescript
import { getCompressionRatio } from 'som-parser';

const ratio = getCompressionRatio(som);
// e.g. 6.25 (HTML was 6.25x larger than the SOM representation)
```

## API Reference

### Parser

| Function | Description |
|----------|-------------|
| `parseSom(input: string \| object): Som` | Parse JSON string or object into a typed Som. Throws on invalid input. |
| `isValidSom(input: unknown): input is Som` | Type guard to check if a value conforms to the SOM schema. |
| `fromPlasmate(jsonOutput: string): Som` | Parse raw Plasmate CLI output, handling extra text around the JSON. |

### Query Utilities

| Function | Description |
|----------|-------------|
| `getAllElements(som): SomElement[]` | Flatten all elements from all regions into a single array. |
| `findByRole(som, role): SomElement[]` | Find elements by role (e.g., `'link'`, `'button'`, `'heading'`). |
| `findById(som, id): SomElement \| undefined` | Find a single element by its SOM id. |
| `findByText(som, text, options?): SomElement[]` | Find elements by text content. Case-insensitive substring by default; pass `{ exact: true }` for exact match. |
| `getInteractiveElements(som): SomElement[]` | Get all elements that have actions. |
| `getLinks(som): Array<{ text, href, id }>` | Extract all links with text, URL, and id. |
| `getForms(som): SomRegion[]` | Get all form regions. |
| `getInputs(som): SomElement[]` | Get all input elements (text_input, textarea, select, checkbox, radio). |
| `getHeadings(som): Array<{ level, text, id }>` | Extract heading hierarchy with levels. |
| `getText(som): string` | Extract all visible text content, joined with newlines. |
| `getTextByRegion(som): Array<{ region, role, text }>` | Extract text grouped by region. |
| `getCompressionRatio(som): number` | Return html_bytes / som_bytes from meta. |
| `toMarkdown(som): string` | Convert SOM to a readable markdown representation. |
| `filter(som, predicate): SomElement[]` | Generic filter across all elements. |

### Types

All SOM types are exported: `Som`, `SomElement`, `SomRegion`, `SomMeta`, `SomElementAttrs`, `RegionRole`, `ElementRole`, `ElementAction`, `SemanticHint`, and more.

## Links

- [SOM Spec](https://plasmate.app/docs/som-spec)
- [Plasmate](https://github.com/plasmate-labs/plasmate)

## License

Apache-2.0
