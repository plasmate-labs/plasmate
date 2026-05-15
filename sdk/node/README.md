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

- **`som(url, options?)`** - Convenience alias for `fetchPage`, returns typed `Som`
- **`fetchPage(url, options?)`** - Returns SOM JSON
- **`extractText(url, options?)`** - Returns clean text

### Stateful (interactive sessions)

- **`openPage(url)`** - Returns `{ sessionId, som }`
- **`evaluate(sessionId, expression)`** - Run JS, get result
- **`click(sessionId, elementId)`** - Click element, get updated SOM
- **`closePage(sessionId)`** - Close session

### Lifecycle

- **`close()`** - Shut down the plasmate process

## SOM Types

The SDK exports full TypeScript types matching the [SOM Specification v1.0](../../specs/som-schema.json):

```typescript
import type { Som, SomRegion, SomElement, SomMeta, StructuredData } from 'plasmate';
import type { RegionRole, ElementRole, SemanticHint, ElementAction } from 'plasmate';
```

Key types:

- **`Som`** - Root document: `som_version`, `url`, `title`, `lang`, `regions`, `meta`, `structured_data?`
- **`SomRegion`** - Semantic region with `id`, `role` (navigation, main, aside, header, footer, form, dialog, content), form submission metadata, and `elements`
- **`SomElement`** - Element with `id`, `role` (link, button, text\_input, etc.), `text?`, `actions?`, `attrs?`, `children?`, `hints?`
- **`SomMeta`** - Metadata: `html_bytes`, `som_bytes`, `element_count`, `interactive_count`
- **`StructuredData`** - Extracted JSON-LD, OpenGraph, Twitter Cards, meta tags, and links

## Query Helpers

The SDK includes query helpers for searching and traversing SOM documents:

```typescript
import {
  findActionTargetByCacheKey, findActionTargetByHtmlId, findActionTargetById,
  findByRole, findById, findByHtmlId, findByTag, findInteractive,
  findByText, flatElements, getActionPlan, getActionPlanFingerprint,
  getActionPlanIndex, getActionPlanSummary, getEnabledActionPlan,
  getTokenEstimate,
} from 'plasmate';

const browser = new Plasmate();
const som = await browser.som('https://example.com');

// Find the main content region
const [main] = findByRole(som, 'main');

// Look up an element by its stable ID
const el = findById(som, 'e5');

// Bridge back to a live DOM id when the source page provided one
const sourceEl = findByHtmlId(som, 'login-button');

// Find all links
const links = findByTag(som, 'link');

// Get all interactive elements (those with actions)
const interactive = findInteractive(som);

// Get compact action targets for cached agent workflows
const actionPlan = getEnabledActionPlan(som);
console.log(actionPlan.map((target) => [target.id, target.cache_key, target.actions, target.expanded]));
const cachedTarget = findActionTargetByCacheKey(som, actionPlan[0].cache_key);
const sameTarget = findActionTargetById(som, actionPlan[0].id);
const domTarget = findActionTargetByHtmlId(som, 'login-button');
const targetIndex = getActionPlanIndex(som, { enabledOnly: true });
const readyTarget = targetIndex.byCacheKey[actionPlan[0].cache_key];
const planFingerprint = getActionPlanFingerprint(som, { enabledOnly: true });
const planSummary = getActionPlanSummary(som);
console.log(
  planFingerprint,
  planSummary.enabled,
  planSummary.uniqueCacheKeys,
  planSummary.duplicateCacheKeys,
);

// Search by visible text or control label (case-insensitive by default)
const matches = findByText(som, 'sign in');
const exactLabel = findByText(som, 'Email', { exact: true });

// Flatten all elements across all regions
const all = flatElements(som);

// Estimate token count
const tokens = getTokenEstimate(som);
console.log(`~${tokens} tokens`);

browser.close();
```

| Function | Returns | Description |
|----------|---------|-------------|
| `findByRole(som, role)` | `SomRegion[]` | Find regions by role |
| `findById(som, id)` | `SomElement \| undefined` | Find element by stable SOM ID |
| `findByHtmlId(som, htmlId)` | `SomElement \| undefined` | Find element by original HTML ID |
| `findByTag(som, tag)` | `SomElement[]` | Find elements by element role |
| `findInteractive(som)` | `SomElement[]` | All elements with actions |
| `getActionPlan(som)` | `ActionPlanItem[]` | Compact action targets with cache keys, availability, original `html_id` bridge cues, link target/rel/download cues, graphical submitter alt/src cues, form/list and form submission context, submitter override cues, select selected_values/size context, popover/command relation cues, title/label/description ID relationships, ARIA source text plus locale/direction cues, text-entry/input-affordance cues, validation/range constraints, ARIA live-region cues, ARIA owns/flowto/details relationships, ARIA widget affordances, orientation/sort/value state, and set-position cues |
| `getEnabledActionPlan(som)` | `ActionPlanItem[]` | Compact action targets whose `enabled` field is not false |
| `getActionPlanCacheKey(item)` | `string` | Deterministic key for caching or comparing action targets |
| `getActionPlanIndex(som, { enabledOnly })` | `ActionPlanIndex` | Index compact action targets by `byId`, `byCacheKey`, and `byHtmlId` for replay validation |
| `getActionPlanFingerprint(som, { enabledOnly })` | `string` | Deterministic plan-level fingerprint for replay drift checks |
| `getActionPlanSummary(som)` | `ActionPlanSummary` | Action-plan fingerprints plus total/enabled/disabled, role, blocked-reason, cache-key coverage, duplicate cache-key, and `html_id` coverage counts |
| `findActionTargetByCacheKey(som, cacheKey)` | `ActionPlanItem \| undefined` | Resolve a cached action target from the current SOM action plan |
| `findActionTargetById(som, id)` | `ActionPlanItem \| undefined` | Resolve an action target by stable SOM id |
| `findActionTargetByHtmlId(som, htmlId)` | `ActionPlanItem \| undefined` | Resolve an action target by original HTML id |
| `findByText(som, text, { exact })` | `SomElement[]` | Search visible text and labels; default is case-insensitive substring, exact is case-sensitive |
| `flatElements(som)` | `SomElement[]` | Flatten all elements |
| `getTokenEstimate(som)` | `number` | Estimate token count (~4 bytes/token) |

## How It Works

The SDK spawns `plasmate mcp` as a child process and communicates via JSON-RPC 2.0 over stdio. The plasmate binary handles HTML parsing, JavaScript execution (V8), and SOM compilation in Rust.

## License

Apache-2.0
