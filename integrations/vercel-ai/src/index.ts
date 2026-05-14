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
  cache_key?: string
  role?: string
  label?: string
  text?: string
  actions?: string[]
  enabled?: boolean
  disabled?: boolean
  blocked_reason?: string
  required?: boolean
  readonly?: boolean
  description?: string
  autocomplete?: string
  inputmode?: string
  enterkeyhint?: string
  form?: string
  list?: string
  popovertarget?: string
  popovertargetaction?: string
  commandfor?: string
  command?: string
  popover?: string
  accesskey?: string
  minlength?: number | string
  maxlength?: number | string
  pattern?: string
  checked?: boolean | string
  expanded?: boolean
  pressed?: boolean
  selected?: boolean
  current?: boolean | string
  controls?: string
  haspopup?: boolean | string
  invalid?: boolean | string
  aria_autocomplete?: string
  active_descendant?: string
  errormessage?: string
  keyshortcuts?: string
  roledescription?: string
  busy?: boolean
  live?: string
  atomic?: boolean
  relevant?: string
  owns?: string
  flowto?: string
  details?: string
  href?: string
  input_type?: string
  value?: string
  name?: string
  placeholder?: string
  group?: string
  [key: string]: unknown
}

/**
 * Minimal SOM element shape needed to derive Vercel AI action menus.
 */
export interface PlasmateSomElement {
  id?: string
  role?: string
  label?: string
  text?: string
  actions?: string[]
  attrs?: Record<string, unknown>
  children?: PlasmateSomElement[]
  shadow?: {
    elements?: PlasmateSomElement[]
  }
}

/**
 * Minimal SOM shape accepted by extractPlasmateActionTargets().
 */
export interface PlasmateSom {
  regions?: Array<{
    elements?: PlasmateSomElement[]
  }>
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
  'cache_key, required, readonly, value, autocomplete, inputmode, enterkeyhint, form, list, popovertarget, popovertargetaction, commandfor, command, accesskey, aria_autocomplete, active_descendant, errormessage, keyshortcuts, roledescription, busy, live, atomic, relevant, owns, flowto, details, pattern, minlength, maxlength, invalid, description, placeholder, group, current, controls, and haspopup fields when choosing or reusing form controls.'

function compactString(value: unknown): string | undefined {
  return typeof value === 'string' && value.length > 0 ? value : undefined
}

function stableActionTargetParts(target: PlasmateActionTarget) {
  return [
    compactString(target.id),
    compactString(target.role),
    compactString(target.label ?? target.text),
    [...(target.actions ?? [])].sort().join(',') || undefined,
    compactString(target.name),
    compactString(target.href),
    compactString(target.input_type),
    compactString(target.group),
    compactString(target.placeholder),
  ]
}

function fnv1a32(input: string): string {
  let hash = 0x811c9dc5

  for (let index = 0; index < input.length; index += 1) {
    hash ^= input.charCodeAt(index)
    hash = Math.imul(hash, 0x01000193)
  }

  return (hash >>> 0).toString(16).padStart(8, '0')
}

/**
 * Return a deterministic key for caching or comparing action targets.
 */
export function getPlasmateActionTargetCacheKey(
  target: PlasmateActionTarget
): string {
  return `plasmate-action:v1:${fnv1a32(JSON.stringify(stableActionTargetParts(target)))}`
}

/**
 * Return whether a compact Plasmate action target is safe to offer for action.
 */
export function isPlasmateActionTargetAvailable(
  target: PlasmateActionTarget
): boolean {
  return (
    target.enabled !== false &&
    target.disabled !== true &&
    target.readonly !== true &&
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
    cache_key:
      target.cache_key ?? getPlasmateActionTargetCacheKey(target),
    enabled,
    ...(enabled
      ? {}
      : { blocked_reason: target.blocked_reason ?? (target.readonly ? 'readonly' : 'disabled') }),
  }
}

function collectSomElements(elements: readonly PlasmateSomElement[] = []) {
  const collected: PlasmateSomElement[] = []

  for (const element of elements) {
    collected.push(element)

    if (element.children?.length) {
      collected.push(...collectSomElements(element.children))
    }

    if (element.shadow?.elements?.length) {
      collected.push(...collectSomElements(element.shadow.elements))
    }
  }

  return collected
}

function copyStringAttr(
  item: PlasmateActionTarget,
  attrs: Record<string, unknown>,
  key: 'href' | 'input_type' | 'value' | 'name' | 'placeholder' | 'description' | 'group' | 'autocomplete' | 'inputmode' | 'enterkeyhint' | 'form' | 'list' | 'popovertarget' | 'popovertargetaction' | 'commandfor' | 'command' | 'popover' | 'accesskey' | 'pattern'
) {
  if (typeof attrs[key] === 'string' && attrs[key].length > 0) {
    item[key] = attrs[key]
  }
}

function copyStringOrNumberAttr(
  item: PlasmateActionTarget,
  attrs: Record<string, unknown>,
  key: 'minlength' | 'maxlength'
) {
  if (typeof attrs[key] === 'string' || typeof attrs[key] === 'number') {
    item[key] = attrs[key]
  }
}

/**
 * Extract compact action targets from a raw Plasmate SOM response.
 */
export function extractPlasmateActionTargets(
  som: PlasmateSom
): PlasmateActionTarget[] {
  return (som.regions ?? [])
    .flatMap((region) => collectSomElements(region.elements ?? []))
    .filter((element) => element.actions?.length)
    .map((element) => {
      const attrs = element.attrs ?? {}
      const target: PlasmateActionTarget = {
        id: element.id,
        role: element.role,
        actions: element.actions,
        enabled: true,
      }
      const label = element.label ?? element.text

      if (label) target.label = label
      copyStringAttr(target, attrs, 'href')
      copyStringAttr(target, attrs, 'input_type')
      copyStringAttr(target, attrs, 'value')
      copyStringAttr(target, attrs, 'name')
      copyStringAttr(target, attrs, 'autocomplete')
      copyStringAttr(target, attrs, 'inputmode')
      copyStringAttr(target, attrs, 'enterkeyhint')
      copyStringAttr(target, attrs, 'form')
      copyStringAttr(target, attrs, 'list')
      copyStringAttr(target, attrs, 'popovertarget')
      copyStringAttr(target, attrs, 'popovertargetaction')
      copyStringAttr(target, attrs, 'commandfor')
      copyStringAttr(target, attrs, 'command')
      copyStringAttr(target, attrs, 'popover')
      copyStringAttr(target, attrs, 'accesskey')
      copyStringAttr(target, attrs, 'placeholder')
      copyStringOrNumberAttr(target, attrs, 'minlength')
      copyStringOrNumberAttr(target, attrs, 'maxlength')
      copyStringAttr(target, attrs, 'pattern')
      copyStringAttr(target, attrs, 'description')
      copyStringAttr(target, attrs, 'group')

      if (typeof attrs.checked === 'boolean') {
        target.checked = attrs.checked
      } else {
        const aria = attrs.aria
        if (
          aria &&
          typeof aria === 'object' &&
          'checked' in aria
        ) {
          const checked = (aria as Record<string, unknown>).checked
          if (typeof checked === 'boolean' || typeof checked === 'string') {
            target.checked = checked
          }
        }
      }
      const aria = attrs.aria
      if (aria && typeof aria === 'object') {
        for (const stateKey of ['expanded', 'pressed', 'selected'] as const) {
          const stateValue = (aria as Record<string, unknown>)[stateKey]
          if (typeof stateValue === 'boolean') {
            target[stateKey] = stateValue
          }
        }
        const current = (aria as Record<string, unknown>).current
        if (typeof current === 'boolean' || typeof current === 'string') {
          target.current = current
        }
        const controls = (aria as Record<string, unknown>).controls
        if (typeof controls === 'string' && controls.length > 0) {
          target.controls = controls
        }
        const haspopup = (aria as Record<string, unknown>).haspopup
        if (typeof haspopup === 'boolean' || typeof haspopup === 'string') {
          target.haspopup = haspopup
        }
        const invalid = (aria as Record<string, unknown>).invalid
        if (typeof invalid === 'boolean' || typeof invalid === 'string') {
          target.invalid = invalid
        }
        const ariaAutocomplete = (aria as Record<string, unknown>).autocomplete
        if (typeof ariaAutocomplete === 'string' && ariaAutocomplete.length > 0) {
          target.aria_autocomplete = ariaAutocomplete
        }
        const activeDescendant = (aria as Record<string, unknown>).active_descendant
        if (typeof activeDescendant === 'string' && activeDescendant.length > 0) {
          target.active_descendant = activeDescendant
        }
        const errorMessage = (aria as Record<string, unknown>).errormessage
        if (typeof errorMessage === 'string' && errorMessage.length > 0) {
          target.errormessage = errorMessage
        }
        const keyshortcuts = (aria as Record<string, unknown>).keyshortcuts
        if (typeof keyshortcuts === 'string' && keyshortcuts.length > 0) {
          target.keyshortcuts = keyshortcuts
        }
        const roledescription = (aria as Record<string, unknown>).roledescription
        if (typeof roledescription === 'string' && roledescription.length > 0) {
          target.roledescription = roledescription
        }
        const busy = (aria as Record<string, unknown>).busy
        if (typeof busy === 'boolean') {
          target.busy = busy
        }
        const live = (aria as Record<string, unknown>).live
        if (typeof live === 'string' && live.length > 0) {
          target.live = live
        }
        const atomic = (aria as Record<string, unknown>).atomic
        if (typeof atomic === 'boolean') {
          target.atomic = atomic
        }
        const relevant = (aria as Record<string, unknown>).relevant
        if (typeof relevant === 'string' && relevant.length > 0) {
          target.relevant = relevant
        }
        const owns = (aria as Record<string, unknown>).owns
        if (typeof owns === 'string' && owns.length > 0) {
          target.owns = owns
        }
        const flowto = (aria as Record<string, unknown>).flowto
        if (typeof flowto === 'string' && flowto.length > 0) {
          target.flowto = flowto
        }
        const details = (aria as Record<string, unknown>).details
        if (typeof details === 'string' && details.length > 0) {
          target.details = details
        }
      }

      if (typeof attrs.required === 'boolean') {
        target.required = attrs.required
      }

      if (typeof attrs.readonly === 'boolean') {
        target.readonly = attrs.readonly
      }

      if (typeof attrs.disabled === 'boolean') {
        target.disabled = attrs.disabled
        if (attrs.disabled) {
          target.enabled = false
          target.blocked_reason = 'disabled'
        }
      } else if (attrs.readonly === true) {
        target.enabled = false
        target.blocked_reason = 'readonly'
      }

      return normalizePlasmateActionTarget(target)
    })
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
      const cacheKey = target.cache_key ? ` [cache_key=${target.cache_key}]` : ''
      const role = target.role ?? 'target'
      const name = target.label ?? target.text ?? ''
      const actions = target.actions?.length
        ? ` (${target.actions.join(',')})`
        : ''
      const state = target.enabled === false ? ' [blocked]' : ' [enabled]'
      const blockedReason = target.blocked_reason
        ? ` [blocked_reason=${target.blocked_reason}]`
        : ''
      const required = target.required ? ' [required]' : ''
      const readonly = target.readonly ? ' [readonly]' : ''
      const inputType = target.input_type ? ` [type=${target.input_type}]` : ''
      const value = target.value ? ` [value=${target.value}]` : ''
      const autocomplete = target.autocomplete
        ? ` [autocomplete=${target.autocomplete}]`
        : ''
      const inputmode = target.inputmode
        ? ` [inputmode=${target.inputmode}]`
        : ''
      const enterkeyhint = target.enterkeyhint
        ? ` [enterkeyhint=${target.enterkeyhint}]`
        : ''
      const form = target.form ? ` [form=${target.form}]` : ''
      const list = target.list ? ` [list=${target.list}]` : ''
      const popovertarget = target.popovertarget
        ? ` [popovertarget=${target.popovertarget}]`
        : ''
      const popovertargetaction = target.popovertargetaction
        ? ` [popovertargetaction=${target.popovertargetaction}]`
        : ''
      const commandfor = target.commandfor
        ? ` [commandfor=${target.commandfor}]`
        : ''
      const command = target.command ? ` [command=${target.command}]` : ''
      const popover = target.popover ? ` [popover=${target.popover}]` : ''
      const accesskey = target.accesskey
        ? ` [accesskey=${target.accesskey}]`
        : ''
      const placeholder = target.placeholder
        ? ` [placeholder=${target.placeholder}]`
        : ''
      const minlength =
        typeof target.minlength !== 'undefined' ? ` [minlength=${target.minlength}]` : ''
      const maxlength =
        typeof target.maxlength !== 'undefined' ? ` [maxlength=${target.maxlength}]` : ''
      const pattern = target.pattern ? ` [pattern=${target.pattern}]` : ''
      const checked =
        typeof target.checked !== 'undefined' ? ` [checked=${target.checked}]` : ''
      const expanded =
        typeof target.expanded !== 'undefined' ? ` [expanded=${target.expanded}]` : ''
      const pressed =
        typeof target.pressed !== 'undefined' ? ` [pressed=${target.pressed}]` : ''
      const selected =
        typeof target.selected !== 'undefined' ? ` [selected=${target.selected}]` : ''
      const current =
        typeof target.current !== 'undefined' ? ` [current=${target.current}]` : ''
      const controls = target.controls ? ` [controls=${target.controls}]` : ''
      const haspopup =
        typeof target.haspopup !== 'undefined' ? ` [haspopup=${target.haspopup}]` : ''
      const invalid =
        typeof target.invalid !== 'undefined' ? ` [invalid=${target.invalid}]` : ''
      const ariaAutocomplete = target.aria_autocomplete
        ? ` [aria_autocomplete=${target.aria_autocomplete}]`
        : ''
      const activeDescendant = target.active_descendant
        ? ` [active_descendant=${target.active_descendant}]`
        : ''
      const errorMessage = target.errormessage
        ? ` [errormessage=${target.errormessage}]`
        : ''
      const keyshortcuts = target.keyshortcuts
        ? ` [keyshortcuts=${target.keyshortcuts}]`
        : ''
      const roledescription = target.roledescription
        ? ` [roledescription=${target.roledescription}]`
        : ''
      const busy =
        typeof target.busy !== 'undefined' ? ` [busy=${target.busy}]` : ''
      const live = target.live ? ` [live=${target.live}]` : ''
      const atomic =
        typeof target.atomic !== 'undefined' ? ` [atomic=${target.atomic}]` : ''
      const relevant = target.relevant ? ` [relevant=${target.relevant}]` : ''
      const owns = target.owns ? ` [owns=${target.owns}]` : ''
      const flowto = target.flowto ? ` [flowto=${target.flowto}]` : ''
      const details = target.details ? ` [details=${target.details}]` : ''
      const group = target.group ? ` [group=${target.group}]` : ''
      const description = target.description
        ? ` [description=${target.description}]`
        : ''

      return `${id}${role}${name ? ` "${name}"` : ''}${actions}${state}${cacheKey}${blockedReason}${required}${readonly}${inputType}${value}${autocomplete}${inputmode}${enterkeyhint}${form}${list}${popovertarget}${popovertargetaction}${commandfor}${command}${popover}${accesskey}${placeholder}${minlength}${maxlength}${pattern}${checked}${expanded}${pressed}${selected}${current}${controls}${haspopup}${invalid}${ariaAutocomplete}${activeDescendant}${errorMessage}${keyshortcuts}${roledescription}${busy}${live}${atomic}${relevant}${owns}${flowto}${details}${group}${description}`
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
