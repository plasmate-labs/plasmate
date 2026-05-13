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
 * Compact action target shape emitted by Plasmate SOM action-plan helpers.
 */
export interface PlasmateActionTarget {
  id?: string
  role?: string
  label?: string
  text?: string
  actions?: string[]
  enabled?: boolean
  disabled?: boolean
  blocked_reason?: string
  required?: boolean
  description?: string
  placeholder?: string
  group?: string
  [key: string]: unknown
}

/**
 * Options for preparing a compact action plan before prompting a model.
 */
export interface PreparePlasmateActionPlanOptions {
  /**
   * Keep unavailable targets in the returned menu.
   * Defaults to false so prompts only include targets that can be acted on.
   */
  includeUnavailable?: boolean

  /**
   * Maximum number of action targets to return after filtering.
   */
  maxTargets?: number
}

/**
 * System prompt guidance for Vercel AI SDK agents using Plasmate tools.
 *
 * Plasmate SOM responses expose action targets with stable element ids and
 * availability fields. Add this to your system prompt when your agent will
 * browse forms or reuse action plans across steps.
 */
export const plasmateActionGuidance =
  'Use Plasmate SOM element ids for browser actions. Treat action targets ' +
  'with enabled=false or blocked_reason as unavailable, and prefer ' +
  'required, description, placeholder, and group fields when choosing form controls.'

/**
 * Return whether a compact Plasmate action target is safe to offer for action.
 */
export function isPlasmateActionTargetAvailable(
  target: PlasmateActionTarget
): boolean {
  return (
    target.enabled !== false &&
    target.disabled !== true &&
    !target.blocked_reason
  )
}

/**
 * Normalize a compact Plasmate action target without mutating the caller value.
 */
export function normalizePlasmateActionTarget(
  target: PlasmateActionTarget
): PlasmateActionTarget {
  const enabled = isPlasmateActionTargetAvailable(target)

  return {
    ...target,
    enabled,
    ...(enabled ? {} : { blocked_reason: target.blocked_reason ?? 'disabled' }),
  }
}

/**
 * Prepare a compact action menu for Vercel AI SDK prompts or cached workflows.
 */
export function preparePlasmateActionPlan(
  targets: readonly PlasmateActionTarget[],
  options: PreparePlasmateActionPlanOptions = {}
): PlasmateActionTarget[] {
  const prepared = targets
    .map(normalizePlasmateActionTarget)
    .filter(
      (target) =>
        options.includeUnavailable || isPlasmateActionTargetAvailable(target)
    )

  if (typeof options.maxTargets === 'number' && options.maxTargets >= 0) {
    return prepared.slice(0, options.maxTargets)
  }

  return prepared
}

/**
 * Format a compact action menu for a model prompt or trace log.
 */
export function formatPlasmateActionPlan(
  targets: readonly PlasmateActionTarget[],
  options: PreparePlasmateActionPlanOptions = {}
): string {
  return preparePlasmateActionPlan(targets, options)
    .map((target) => {
      const id = target.id ? `[${target.id}] ` : ''
      const role = target.role ?? 'target'
      const name = target.label ?? target.text ?? ''
      const actions = target.actions?.length
        ? ` (${target.actions.join(',')})`
        : ''
      const state = target.enabled === false ? ' [blocked]' : ' [enabled]'
      const required = target.required ? ' [required]' : ''
      const group = target.group ? ` [group=${target.group}]` : ''
      const description = target.description
        ? ` [description=${target.description}]`
        : ''

      return `${id}${role}${name ? ` "${name}"` : ''}${actions}${state}${required}${group}${description}`
    })
    .join('\n')
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
