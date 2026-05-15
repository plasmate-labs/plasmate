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

### Plan agent actions

```typescript
import {
  findActionTargetByCacheKey,
  findActionTargetByHtmlId,
  findActionTargetById,
  findByAction,
  getEnabledActionPlan,
} from 'som-parser';

const plan = getEnabledActionPlan(som);
// Compact action targets with id, cache_key, role, actions, enabled, labels, link target/rel/download cues, graphical submitter alt/src cues, form/list context, form submission metadata, submitter override cues, select selected_values/size context, popover/command relation cues, text-entry/input hints, validation/range cues, and ARIA owns/flowto/details plus orientation/sort/value state.
const cached = findActionTargetByCacheKey(som, plan[0].cache_key);
const sameTarget = findActionTargetById(som, plan[0].id);
const domTarget = findActionTargetByHtmlId(som, 'save-settings');

const clickable = findByAction(som, 'click');
// Elements that can be clicked.
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
| `findByHtmlId(som, htmlId): SomElement \| undefined` | Find a single element by its original HTML id. |
| `findByText(som, text, options?): SomElement[]` | Find elements by text content. Case-insensitive substring by default; pass `{ exact: true }` for exact match. |
| `findByAction(som, action): SomElement[]` | Find elements that expose a specific action. |
| `findByHint(som, hint): SomElement[]` | Find elements tagged with a semantic hint. |
| `getActionPlan(som): ActionPlanItem[]` | Return compact action targets with cache keys, availability, original DOM-id bridge cues, link target/rel/download cues, graphical submitter alt/src cues, form/list and form submission context, submitter override cues, select selected_values/size context, popover/command relation cues, title/label/description ID relationships, ARIA source text plus locale/direction cues, text-entry/input-affordance cues, validation/range constraints, ARIA live-region cues, ARIA owns/flowto/details relationships, ARIA widget affordances, orientation/sort/value state, and set-position cues for agent planning. |
| `getEnabledActionPlan(som): ActionPlanItem[]` | Return compact action targets whose `enabled` field is not false. |
| `getActionPlanCacheKey(item): string` | Return a deterministic key for caching or comparing an action target. |
| `findActionTargetByCacheKey(som, cacheKey): ActionPlanItem \| undefined` | Resolve a cached action target from the current SOM action plan. |
| `findActionTargetById(som, id): ActionPlanItem \| undefined` | Resolve an action target by stable SOM id. |
| `findActionTargetByHtmlId(som, htmlId): ActionPlanItem \| undefined` | Resolve an action target by original HTML id. |
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
