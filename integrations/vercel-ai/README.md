# @plasmate/ai

**Plasmate browser tools for the [Vercel AI SDK](https://sdk.vercel.ai)**

Plasmate is a headless browser MCP server that gives AI models structured access to the web via a [Set of Marks (SOM)](https://plasmate.dev/docs/som) representation. This package is a thin wrapper that connects Plasmate to the Vercel AI SDK in one function call.

## Install

```bash
npm install @plasmate/ai ai
```

You also need [Plasmate](https://plasmate.dev/docs/install) installed locally:

```bash
npm install -g plasmate
# or
brew install plasmate-labs/tap/plasmate
```

## Quick Start

```ts
import { createPlasmateTools } from '@plasmate/ai'
import { generateText } from 'ai'
import { openai } from '@ai-sdk/openai'

const { tools, close } = await createPlasmateTools()

const { text } = await generateText({
  model: openai('gpt-4o'),
  tools,
  maxSteps: 5,
  prompt: 'Summarize the top 3 stories on news.ycombinator.com',
})

console.log(text)

await close()
```

### Action availability guidance

```ts
import { createPlasmateTools, plasmateActionGuidance } from '@plasmate/ai'
import { generateText } from 'ai'
import { openai } from '@ai-sdk/openai'

const { tools, close } = await createPlasmateTools()

const { text } = await generateText({
  model: openai('gpt-4o'),
  tools,
  system: plasmateActionGuidance,
  maxSteps: 5,
  prompt: 'Open the settings page and update the required fields only.',
})
```

`plasmateActionGuidance` tells the model to honor SOM action targets with
`cache_key`, `html_id`, `enabled`, `blocked_reason`, `required`, `description`,
`placeholder`, `group`, `readonly`, `inert`, `value`, `checked`, `expanded`, `pressed`, and
`selected`, `current`, `controls`, `haspopup`, `name`, `accept`, `capture`,
`multiple`, `autocomplete`, `inputmode`, `enterkeyhint`, `autocapitalize`,
`dirname`, `spellcheck`, `target`, `rel`,
`download`, `form`, `list`, `popovertarget`, `popovertargetaction`,
`commandfor`, `command`, `button_type`, `formaction`, `formmethod`,
`formenctype`, `formtarget`, `formnovalidate`, `accesskey`, `aria_placeholder`, `aria_autocomplete`,
`active_descendant`, `errormessage`,
`keyshortcuts`, `roledescription`, `busy`, `live`, `atomic`, `relevant`,
`owns`, `flowto`, `details`, `multiline`, `multiselectable`, `orientation`,
`sort`, `level`, `posinset`, `setsize`, `valuemin`, `valuemax`, `valuenow`,
`valuetext`, `minlength`, `maxlength`, `min`, `max`, `step`,
`pattern`, and `invalid` fields before selecting or
reusing browser actions. Use
`extractPlasmateActionTargets()`, `preparePlasmateActionPlan()`, or
`formatPlasmateActionPlan()` when your app filters cached or extracted action
plans before passing them to the model.

## API

### `createPlasmateTools(options?)`

Spawns `plasmate mcp` as a stdio MCP server and returns tools ready for use with `generateText`, `streamText`, etc.

**Options:**

| Option    | Type     | Default       | Description                              |
|-----------|----------|---------------|------------------------------------------|
| `binary`  | `string` | `'plasmate'`  | Path to the plasmate binary (if not in PATH) |

**Returns:** `Promise<{ tools, close }>`

- `tools` — `Record<string, CoreTool>` ready to pass directly to `generateText` / `streamText`
- `close()` — Call this when done to terminate the MCP subprocess

### `plasmateActionGuidance`

A short system prompt string for Vercel AI SDK agents. Use it when browsing
forms or cached workflows so the model skips disabled SOM targets and prefers
required, read-only, described, and grouped controls.

### `isPlasmateActionTargetAvailable(target)`

Returns `false` for compact action targets with `enabled: false`,
`disabled: true`, `inert: true`, `readonly: true`, or any `blocked_reason`. Use this when trimming an action
menu before a Vercel AI SDK call.

### `normalizePlasmateActionTarget(target)`

Returns a copy of an action target with explicit `enabled` state. Targets with
`disabled: true`, `inert: true`, `readonly: true`, `enabled: false`, or any `blocked_reason` normalize to
`enabled: false` and keep or receive a `blocked_reason`.

### `getPlasmateActionTargetCacheKey(target)`

Returns a deterministic key for caching or comparing a compact action target.
The key is derived from stable action fields such as id, role, label, actions,
name, href, input type, group, and placeholder.

### `extractPlasmateActionTargets(som)`

Flattens a raw Plasmate SOM response into compact action targets. It traverses
nested `children` and `shadow.elements`, copies common action metadata from
`attrs` (`href`, `name`, `input_type`, `placeholder`, `description`, `required`,
`disabled`, `inert`, `readonly`, `html_id`, and `group`), and normalizes availability plus `cache_key` state.

### `preparePlasmateActionPlan(targets, options?)`

Normalizes a list of action targets, filters unavailable targets by default,
and optionally caps the returned menu with `maxTargets`. Pass
`includeUnavailable: true` when you want a trace or UI to show blocked targets.

### `indexPlasmateActionTargets(targets, options?)`

Returns normalized action targets indexed by `by_id`, `by_cache_key`,
`by_html_id`, `by_test_id`, and `by_label`, plus grouped `by_role` and
`by_action` buckets. Use this when cached Vercel AI workflows need to resolve
a saved target or scope a plan without scanning the full action menu.

### `findPlasmateActionTarget(targets, value, options?)`

Finds one action target by `id`, `cache_key`, `html_id`, `test_id`, or the
default auto lookup across the stable replay buckets. Pass `by: 'label'`
explicitly for recovery/debugging flows where the label is unique enough.

### `findPlasmateActionTargetsByLabel(targets, label, options?)`

Returns compact action targets whose accessible label matches text. Label
matching is substring-based by default; pass `exact: true` for exact matching.
Use label lookup for user-facing recovery and inspection, not unattended
replay, because page labels can collide.

### `findPlasmateActionTargetsByRole(targets, role, options?)`

Returns compact action targets for one SOM role, such as `button` or
`text_input`.

### `findPlasmateActionTargetsByAction(targets, action, options?)`

Returns compact action targets exposing one action, such as `click`, `type`, or
`select`.

```ts
const index = indexPlasmateActionTargets(actionTargets, {
  includeUnavailable: true,
})
const save = findPlasmateActionTarget(actionTargets, 'settings-save', {
  by: 'test_id',
  includeUnavailable: true,
})
const saveByLabel = findPlasmateActionTarget(actionTargets, 'Save', {
  by: 'label',
  includeUnavailable: true,
})
const planTargets = findPlasmateActionTargetsByLabel(actionTargets, 'plan')
const enabledClicks = findPlasmateActionTargetsByAction(actionTargets, 'click')
```

### `formatPlasmateActionPlan(targets, options?)`

Formats a prepared action menu as compact prompt text:

```ts
const actionTargets = extractPlasmateActionTargets(som)
const menu = formatPlasmateActionPlan(actionTargets, { maxTargets: 20 })

const { text } = await generateText({
  model: openai('gpt-4o'),
  tools,
  system: `${plasmateActionGuidance}\n\nAvailable actions:\n${menu}`,
  prompt: 'Update the billing plan if the selector is available.',
})
```

## Available Tools

| Tool            | Description                                                      |
|-----------------|------------------------------------------------------------------|
| `fetch_page`    | Fetch a URL and return a structured SOM representation           |
| `extract_text`  | Extract readable text content from the current page              |
| `extract_links` | Extract all links from the current page                          |
| `open_page`     | Open a URL in a headless browser session                         |
| `click`         | Click an element identified by its SOM marker                    |
| `type_text`     | Type text into a focused input element                           |
| `navigate_to`   | Navigate to a URL within an existing browser session             |
| `evaluate`      | Evaluate JavaScript in the context of the current page           |

## Usage Examples

### Basic web research

```ts
import { createPlasmateTools } from '@plasmate/ai'
import { generateText } from 'ai'
import { anthropic } from '@ai-sdk/anthropic'

const { tools, close } = await createPlasmateTools()

const { text } = await generateText({
  model: anthropic('claude-3-5-sonnet-20241022'),
  tools,
  maxSteps: 10,
  prompt: 'What are the latest AI news headlines on techcrunch.com?',
})

await close()
console.log(text)
```

### Custom binary path

```ts
const { tools, close } = await createPlasmateTools({
  binary: '/usr/local/bin/plasmate',
})
```

### Next.js App Router route

```ts
// app/api/browse/route.ts
import { createPlasmateTools } from '@plasmate/ai'
import { streamText } from 'ai'
import { openai } from '@ai-sdk/openai'

export async function POST(req: Request) {
  const { prompt } = await req.json()

  const { tools, close } = await createPlasmateTools()

  const result = streamText({
    model: openai('gpt-4o'),
    tools,
    maxSteps: 5,
    prompt,
    onFinish: async () => {
      await close()
    },
  })

  return result.toDataStreamResponse()
}
```

### Error handling

```ts
import { createPlasmateTools } from '@plasmate/ai'

try {
  const { tools, close } = await createPlasmateTools()
  // ... use tools
  await close()
} catch (err) {
  // createPlasmateTools throws if plasmate binary is not found
  // or the MCP server fails to start
  console.error('Plasmate error:', err)
}
```

## How It Works

`createPlasmateTools` uses the Vercel AI SDK's `experimental_createMCPClient` with a stdio transport to spawn `plasmate mcp` as a child process. Plasmate speaks the [Model Context Protocol (MCP)](https://modelcontextprotocol.io) over stdin/stdout, and the AI SDK converts the MCP tool definitions into `CoreTool` objects that models can call natively.

```
generateText ──► CoreTool ──► MCP Client ──► plasmate mcp (stdio) ──► headless browser
```

## Requirements

- Node.js 18+
- `plasmate` CLI installed and in PATH (or specify via `binary` option)
- `ai` >= 4.0.0 (peer dependency)

## License

MIT
