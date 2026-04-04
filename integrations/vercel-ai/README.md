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
