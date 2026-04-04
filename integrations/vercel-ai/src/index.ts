import { experimental_createMCPClient } from 'ai'
import { Experimental_StdioMCPTransport } from '@ai-sdk/mcp/mcp-stdio'

/**
 * Options for createPlasmateTools
 */
export interface CreatePlasmateToolsOptions {
  /**
   * Path to the plasmate binary.
   * Defaults to 'plasmate' (resolved from PATH).
   */
  binary?: string
}

/**
 * The result returned by createPlasmateTools.
 * tools: ready-to-use tools for generateText / streamText
 * close: call this when done to shut down the MCP subprocess
 */
export interface PlasmateTools {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  tools: Record<string, any>
  close: () => Promise<void>
}

/**
 * Create Plasmate browser tools for use with the Vercel AI SDK.
 *
 * Spawns `plasmate mcp` as a stdio MCP server and returns the tool set
 * ready for use with `generateText`, `streamText`, etc.
 *
 * @example
 * ```ts
 * import { createPlasmateTools } from '@plasmate/ai'
 * import { generateText } from 'ai'
 * import { openai } from '@ai-sdk/openai'
 *
 * const { tools, close } = await createPlasmateTools()
 *
 * const { text } = await generateText({
 *   model: openai('gpt-4o'),
 *   tools,
 *   maxSteps: 5,
 *   prompt: 'Summarize the top 3 stories on news.ycombinator.com',
 * })
 *
 * await close()
 * ```
 */
export async function createPlasmateTools(
  options: CreatePlasmateToolsOptions = {}
): Promise<PlasmateTools> {
  const binary = options.binary ?? 'plasmate'

  let transport: Experimental_StdioMCPTransport

  try {
    transport = new Experimental_StdioMCPTransport({
      command: binary,
      args: ['mcp'],
    })
  } catch (err) {
    throw new Error(
      `Failed to create Plasmate MCP transport (binary: "${binary}").\n` +
        `Make sure plasmate is installed: https://plasmate.dev/docs/install\n` +
        `Original error: ${err instanceof Error ? err.message : String(err)}`
    )
  }

  let client: Awaited<ReturnType<typeof experimental_createMCPClient>>

  try {
    client = await experimental_createMCPClient({ transport })
  } catch (err) {
    throw new Error(
      `Failed to start Plasmate MCP server (binary: "${binary}").\n` +
        `Make sure plasmate is installed: https://plasmate.dev/docs/install\n` +
        `Original error: ${err instanceof Error ? err.message : String(err)}`
    )
  }

  const tools = await client.tools()

  return {
    tools,
    close: () => client.close(),
  }
}
