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
  html_id?: string
  cache_key?: string
  role?: string
  label?: string
  text?: string
  actions?: string[]
  enabled?: boolean
  disabled?: boolean
  inert?: boolean
  blocked_reason?: string
  required?: boolean
  readonly?: boolean
  description?: string
  autocomplete?: string
  inputmode?: string
  enterkeyhint?: string
  autocapitalize?: string
  dirname?: string
  dir?: string
  lang?: string
  form?: string
  form_action?: string
  form_method?: string
  form_target?: string
  form_enctype?: string
  form_novalidate?: boolean
  form_accept_charset?: string
  form_autocomplete?: string
  list?: string
  popovertarget?: string
  popovertargetaction?: string
  commandfor?: string
  command?: string
  popover?: string
  button_type?: string
  formaction?: string
  formmethod?: string
  formenctype?: string
  formtarget?: string
  formnovalidate?: boolean
  accesskey?: string
  title?: string
  aria_label?: string
  aria_description?: string
  labelledby?: string
  describedby?: string
  spellcheck?: boolean | string
  minlength?: number | string
  maxlength?: number | string
  min?: number | string
  max?: number | string
  step?: string
  pattern?: string
  checked?: boolean | string
  expanded?: boolean
  pressed?: boolean
  selected?: boolean
  multiline?: boolean
  multiselectable?: boolean
  current?: boolean | string
  controls?: string
  haspopup?: boolean | string
  invalid?: boolean | string
  aria_placeholder?: string
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
  orientation?: string
  sort?: string
  level?: string
  posinset?: string
  setsize?: string
  valuemin?: string
  valuemax?: string
  valuenow?: string
  valuetext?: string
  href?: string
  target?: string
  rel?: string
  download?: boolean | string
  alt?: string
  src?: string
  input_type?: string
  value?: string
  name?: string
  accept?: string
  capture?: boolean | string
  multiple?: boolean
  selected_values?: string[]
  size?: number | string
  placeholder?: string
  group?: string
  test_id?: string
  data_action?: string
  data_state?: string
  [key: string]: unknown
}

/**
 * Minimal SOM element shape needed to derive Vercel AI action menus.
 */
export interface PlasmateSomElement {
  id?: string
  html_id?: string
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
    action?: string
    method?: string
    target?: string
    enctype?: string
    novalidate?: boolean
    accept_charset?: string
    autocomplete?: string
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
 * Replay lookup tables for a normalized compact action plan.
 */
export interface PlasmateActionPlanIndex {
  by_id: Record<string, PlasmateActionTarget>
  by_cache_key: Record<string, PlasmateActionTarget>
  by_html_id: Record<string, PlasmateActionTarget>
}

/**
 * Compact counts and fingerprints for replay drift checks.
 */
export interface PlasmateActionPlanSummary {
  fingerprint: string
  enabled_fingerprint: string
  total: number
  enabled: number
  disabled: number
  with_cache_key: number
  unique_cache_keys: number
  duplicate_cache_keys: string[]
  with_html_id: number
  by_role: Record<string, number>
  blocked_reasons: Record<string, number>
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
  'cache_key, html_id, test_id, data_action, data_state, required, readonly, inert, value, target, rel, download, alt, src, name, accept, capture, multiple, selected_values, size, autocomplete, inputmode, enterkeyhint, autocapitalize, dirname, dir, lang, spellcheck, form, form_action, form_method, form_target, form_enctype, form_novalidate, form_accept_charset, form_autocomplete, button_type, formaction, formmethod, formenctype, formtarget, formnovalidate, list, popovertarget, popovertargetaction, commandfor, command, accesskey, title, aria_label, aria_description, labelledby, describedby, aria_placeholder, aria_autocomplete, active_descendant, errormessage, keyshortcuts, roledescription, busy, live, atomic, relevant, owns, flowto, details, multiline, multiselectable, orientation, sort, level, posinset, setsize, valuemin, valuemax, valuenow, valuetext, pattern, minlength, maxlength, min, max, step, invalid, description, placeholder, group, current, controls, and haspopup fields when choosing or reusing form controls.'

function compactString(value: unknown): string | undefined {
  return typeof value === 'string' && value.length > 0 ? value : undefined
}

function stableActionTargetParts(target: PlasmateActionTarget) {
  const parts = [
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

  for (const value of [compactString(target.test_id), compactString(target.data_action)]) {
    if (value) parts.push(value)
  }

  return parts
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
    target.inert !== true &&
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
      : { blocked_reason: target.blocked_reason ?? (target.inert ? 'inert' : target.readonly ? 'readonly' : 'disabled') }),
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
  key: 'href' | 'target' | 'rel' | 'alt' | 'src' | 'input_type' | 'value' | 'name' | 'accept' | 'placeholder' | 'description' | 'group' | 'test_id' | 'data_action' | 'data_state' | 'autocomplete' | 'inputmode' | 'enterkeyhint' | 'autocapitalize' | 'dirname' | 'dir' | 'lang' | 'form' | 'list' | 'popovertarget' | 'popovertargetaction' | 'commandfor' | 'command' | 'popover' | 'button_type' | 'formaction' | 'formmethod' | 'formenctype' | 'formtarget' | 'accesskey' | 'title' | 'aria_label' | 'aria_description' | 'labelledby' | 'describedby' | 'pattern' | 'step'
) {
  if (typeof attrs[key] === 'string' && attrs[key].length > 0) {
    item[key] = attrs[key]
  }
}

function copyStringOrBooleanAttr(
  item: PlasmateActionTarget,
  attrs: Record<string, unknown>,
  key: 'download' | 'capture' | 'spellcheck'
) {
  if (typeof attrs[key] === 'string' || typeof attrs[key] === 'boolean') {
    item[key] = attrs[key]
  }
}

function copyStringOrNumberAttr(
  item: PlasmateActionTarget,
  attrs: Record<string, unknown>,
  key: 'minlength' | 'maxlength' | 'min' | 'max' | 'size'
) {
  if (typeof attrs[key] === 'string' || typeof attrs[key] === 'number') {
    item[key] = attrs[key]
  }
}

function copyFormContext(
  target: PlasmateActionTarget,
  region: NonNullable<PlasmateSom['regions']>[number]
) {
  if (typeof region.action === 'string' && region.action.length > 0) target.form_action = region.action
  if (typeof region.method === 'string' && region.method.length > 0) target.form_method = region.method
  if (typeof region.target === 'string' && region.target.length > 0) target.form_target = region.target
  if (typeof region.enctype === 'string' && region.enctype.length > 0) target.form_enctype = region.enctype
  if (typeof region.novalidate === 'boolean') target.form_novalidate = region.novalidate
  if (typeof region.accept_charset === 'string' && region.accept_charset.length > 0) target.form_accept_charset = region.accept_charset
  if (typeof region.autocomplete === 'string' && region.autocomplete.length > 0) target.form_autocomplete = region.autocomplete
}

/**
 * Extract compact action targets from a raw Plasmate SOM response.
 */
export function extractPlasmateActionTargets(
  som: PlasmateSom
): PlasmateActionTarget[] {
  return (som.regions ?? [])
    .flatMap((region) =>
      collectSomElements(region.elements ?? []).map((element) => ({ element, region }))
    )
    .filter(({ element }) => element.actions?.length)
    .map(({ element, region }) => {
      const attrs = element.attrs ?? {}
      const target: PlasmateActionTarget = {
        id: element.id,
        role: element.role,
        actions: element.actions,
        enabled: true,
      }
      const label = element.label ?? element.text

      if (label) target.label = label
      if (element.html_id) target.html_id = element.html_id
      copyFormContext(target, region)
      copyStringAttr(target, attrs, 'href')
      copyStringAttr(target, attrs, 'target')
      copyStringAttr(target, attrs, 'rel')
      copyStringOrBooleanAttr(target, attrs, 'download')
      copyStringAttr(target, attrs, 'alt')
      copyStringAttr(target, attrs, 'src')
      copyStringAttr(target, attrs, 'input_type')
      copyStringAttr(target, attrs, 'value')
      copyStringAttr(target, attrs, 'name')
      copyStringAttr(target, attrs, 'accept')
      copyStringOrBooleanAttr(target, attrs, 'capture')
      if (Array.isArray(attrs.selected_values)) {
        target.selected_values = attrs.selected_values.filter((value): value is string => typeof value === 'string')
      }
      copyStringOrNumberAttr(target, attrs, 'size')
      copyStringAttr(target, attrs, 'autocomplete')
      copyStringAttr(target, attrs, 'inputmode')
      copyStringAttr(target, attrs, 'enterkeyhint')
      copyStringAttr(target, attrs, 'autocapitalize')
      copyStringAttr(target, attrs, 'dirname')
      copyStringAttr(target, attrs, 'dir')
      copyStringAttr(target, attrs, 'lang')
      copyStringAttr(target, attrs, 'form')
      copyStringAttr(target, attrs, 'list')
      copyStringAttr(target, attrs, 'popovertarget')
      copyStringAttr(target, attrs, 'popovertargetaction')
      copyStringAttr(target, attrs, 'commandfor')
      copyStringAttr(target, attrs, 'command')
      copyStringAttr(target, attrs, 'popover')
      copyStringAttr(target, attrs, 'button_type')
      copyStringAttr(target, attrs, 'formaction')
      copyStringAttr(target, attrs, 'formmethod')
      copyStringAttr(target, attrs, 'formenctype')
      copyStringAttr(target, attrs, 'formtarget')
      if (typeof attrs.formnovalidate === 'boolean') target.formnovalidate = attrs.formnovalidate
      copyStringAttr(target, attrs, 'accesskey')
      copyStringAttr(target, attrs, 'title')
      copyStringAttr(target, attrs, 'aria_label')
      copyStringAttr(target, attrs, 'aria_description')
      copyStringAttr(target, attrs, 'labelledby')
      copyStringAttr(target, attrs, 'describedby')
      copyStringOrBooleanAttr(target, attrs, 'spellcheck')
      copyStringAttr(target, attrs, 'placeholder')
      copyStringOrNumberAttr(target, attrs, 'minlength')
      copyStringOrNumberAttr(target, attrs, 'maxlength')
      copyStringOrNumberAttr(target, attrs, 'min')
      copyStringOrNumberAttr(target, attrs, 'max')
      copyStringAttr(target, attrs, 'step')
      copyStringAttr(target, attrs, 'pattern')
      copyStringAttr(target, attrs, 'description')
      copyStringAttr(target, attrs, 'group')
      copyStringAttr(target, attrs, 'test_id')
      copyStringAttr(target, attrs, 'data_action')
      copyStringAttr(target, attrs, 'data_state')

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
        for (const stateKey of ['expanded', 'pressed', 'selected', 'multiline', 'multiselectable'] as const) {
          const stateValue = (aria as Record<string, unknown>)[stateKey]
          if (typeof stateValue === 'boolean') {
            target[stateKey] = stateValue
          }
        }
        const ariaReadonly = (aria as Record<string, unknown>).readonly
        if (typeof ariaReadonly === 'boolean' && typeof attrs.readonly !== 'boolean') {
          target.readonly = ariaReadonly
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
        const ariaPlaceholder = (aria as Record<string, unknown>).placeholder
        if (typeof ariaPlaceholder === 'string' && ariaPlaceholder.length > 0) {
          target.aria_placeholder = ariaPlaceholder
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
        for (const stateKey of [
          'orientation',
          'sort',
          'level',
          'posinset',
          'setsize',
          'valuemin',
          'valuemax',
          'valuenow',
          'valuetext',
        ] as const) {
          const stateValue = (aria as Record<string, unknown>)[stateKey]
          if (typeof stateValue === 'string' && stateValue.length > 0) {
            target[stateKey] = stateValue
          }
        }
      }

      if (typeof attrs.required === 'boolean') {
        target.required = attrs.required
      }
      if (typeof attrs.multiple === 'boolean') {
        target.multiple = attrs.multiple
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
      }
      if (typeof attrs.inert === 'boolean') {
        target.inert = attrs.inert
        if (attrs.inert) {
          target.enabled = false
          target.blocked_reason = 'inert'
        }
      } else if (target.enabled !== false && attrs.readonly === true) {
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
 * Index action targets by SOM id, deterministic cache key, and source HTML id.
 *
 * The index includes unavailable targets by default so replay validation can
 * distinguish missing targets from currently blocked targets.
 */
export function getPlasmateActionPlanIndex(
  targets: readonly PlasmateActionTarget[],
  options: PreparePlasmateActionPlanOptions = {}
): PlasmateActionPlanIndex {
  const plan = preparePlasmateActionPlan(targets, {
    ...options,
    includeUnavailable: options.includeUnavailable ?? true,
  })
  const index: PlasmateActionPlanIndex = {
    by_id: {},
    by_cache_key: {},
    by_html_id: {},
  }

  for (const target of plan) {
    if (target.id && !index.by_id[target.id]) {
      index.by_id[target.id] = target
    }
    if (target.cache_key && !index.by_cache_key[target.cache_key]) {
      index.by_cache_key[target.cache_key] = target
    }
    if (target.html_id && !index.by_html_id[target.html_id]) {
      index.by_html_id[target.html_id] = target
    }
  }

  return index
}

/**
 * Return a deterministic fingerprint for a compact Vercel AI action plan.
 *
 * The default includes blocked targets so apps can detect whether the current
 * page still matches a cached plan before resolving individual action ids.
 */
export function getPlasmateActionPlanFingerprint(
  targets: readonly PlasmateActionTarget[],
  options: PreparePlasmateActionPlanOptions = {}
): string {
  const plan = preparePlasmateActionPlan(targets, {
    ...options,
    includeUnavailable: options.includeUnavailable ?? true,
  })
  const rows = plan
    .map((target) => [
      target.cache_key,
      target.enabled !== false,
      target.blocked_reason ?? null,
    ])
    .sort((left, right) => String(left[0]).localeCompare(String(right[0])))

  return `plasmate-plan:v1:${fnv1a32(JSON.stringify(rows))}`
}

/**
 * Return compact action-plan counts and fingerprints for replay validation.
 */
export function getPlasmateActionPlanSummary(
  targets: readonly PlasmateActionTarget[]
): PlasmateActionPlanSummary {
  const plan = preparePlasmateActionPlan(targets, { includeUnavailable: true })
  const byRole: Record<string, number> = {}
  const blockedReasons: Record<string, number> = {}
  const cacheKeyCounts: Record<string, number> = {}
  let enabled = 0
  let withCacheKey = 0
  let withHtmlId = 0

  for (const target of plan) {
    const role = target.role ?? 'target'
    byRole[role] = (byRole[role] ?? 0) + 1
    if (target.cache_key) {
      withCacheKey += 1
      cacheKeyCounts[target.cache_key] = (cacheKeyCounts[target.cache_key] ?? 0) + 1
    }
    if (target.html_id) {
      withHtmlId += 1
    }

    if (target.enabled === false) {
      const reason = target.blocked_reason ?? 'unknown'
      blockedReasons[reason] = (blockedReasons[reason] ?? 0) + 1
    } else {
      enabled += 1
    }
  }
  const duplicateCacheKeys = Object.entries(cacheKeyCounts)
    .filter(([, count]) => count > 1)
    .map(([cacheKey]) => cacheKey)
    .sort()

  return {
    fingerprint: getPlasmateActionPlanFingerprint(plan),
    enabled_fingerprint: getPlasmateActionPlanFingerprint(plan, {
      includeUnavailable: false,
    }),
    total: plan.length,
    enabled,
    disabled: plan.length - enabled,
    with_cache_key: withCacheKey,
    unique_cache_keys: Object.keys(cacheKeyCounts).length,
    duplicate_cache_keys: duplicateCacheKeys,
    with_html_id: withHtmlId,
    by_role: Object.fromEntries(Object.entries(byRole).sort()),
    blocked_reasons: Object.fromEntries(Object.entries(blockedReasons).sort()),
  }
}

/**
 * Resolve an action target by stable SOM id from the current action plan.
 */
export function findPlasmateActionTargetById(
  targets: readonly PlasmateActionTarget[],
  id: string,
  options: PreparePlasmateActionPlanOptions = {}
): PlasmateActionTarget | undefined {
  return getPlasmateActionPlanIndex(targets, options).by_id[id]
}

/**
 * Resolve an action target by deterministic cache key from the current plan.
 */
export function findPlasmateActionTargetByCacheKey(
  targets: readonly PlasmateActionTarget[],
  cacheKey: string,
  options: PreparePlasmateActionPlanOptions = {}
): PlasmateActionTarget | undefined {
  return getPlasmateActionPlanIndex(targets, options).by_cache_key[cacheKey]
}

/**
 * Resolve an action target by original source DOM id from the current plan.
 */
export function findPlasmateActionTargetByHtmlId(
  targets: readonly PlasmateActionTarget[],
  htmlId: string,
  options: PreparePlasmateActionPlanOptions = {}
): PlasmateActionTarget | undefined {
  return getPlasmateActionPlanIndex(targets, options).by_html_id[htmlId]
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
      const htmlId = target.html_id ? ` [html_id=${target.html_id}]` : ''
      const cacheKey = target.cache_key ? ` [cache_key=${target.cache_key}]` : ''
      const testId = target.test_id ? ` [test_id=${target.test_id}]` : ''
      const dataAction = target.data_action ? ` [data_action=${target.data_action}]` : ''
      const dataState = target.data_state ? ` [data_state=${target.data_state}]` : ''
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
      const inert = target.inert ? ' [inert]' : ''
      const linkTarget = target.target ? ` [target=${target.target}]` : ''
      const rel = target.rel ? ` [rel=${target.rel}]` : ''
      const download =
        typeof target.download !== 'undefined' ? ` [download=${target.download}]` : ''
      const alt = target.alt ? ` [alt=${target.alt}]` : ''
      const src = target.src ? ` [src=${target.src}]` : ''
      const inputType = target.input_type ? ` [type=${target.input_type}]` : ''
      const value = target.value ? ` [value=${target.value}]` : ''
      const nameAttr = target.name ? ` [name=${target.name}]` : ''
      const accept = target.accept ? ` [accept=${target.accept}]` : ''
      const capture =
        typeof target.capture !== 'undefined' ? ` [capture=${target.capture}]` : ''
      const multiple =
        typeof target.multiple !== 'undefined' ? ` [multiple=${target.multiple}]` : ''
      const selectedValues =
        target.selected_values?.length ? ` [selected_values=${target.selected_values.join(',')}]` : ''
      const size =
        typeof target.size !== 'undefined' ? ` [size=${target.size}]` : ''
      const autocomplete = target.autocomplete
        ? ` [autocomplete=${target.autocomplete}]`
        : ''
      const inputmode = target.inputmode
        ? ` [inputmode=${target.inputmode}]`
        : ''
      const enterkeyhint = target.enterkeyhint
        ? ` [enterkeyhint=${target.enterkeyhint}]`
        : ''
      const autocapitalize = target.autocapitalize
        ? ` [autocapitalize=${target.autocapitalize}]`
        : ''
      const dirname = target.dirname ? ` [dirname=${target.dirname}]` : ''
      const dir = target.dir ? ` [dir=${target.dir}]` : ''
      const lang = target.lang ? ` [lang=${target.lang}]` : ''
      const form = target.form ? ` [form=${target.form}]` : ''
      const formAction = target.form_action ? ` [form_action=${target.form_action}]` : ''
      const formMethod = target.form_method ? ` [form_method=${target.form_method}]` : ''
      const formTarget = target.form_target ? ` [form_target=${target.form_target}]` : ''
      const formEnctype = target.form_enctype ? ` [form_enctype=${target.form_enctype}]` : ''
      const formNoValidate =
        typeof target.form_novalidate !== 'undefined' ? ` [form_novalidate=${target.form_novalidate}]` : ''
      const formAcceptCharset = target.form_accept_charset
        ? ` [form_accept_charset=${target.form_accept_charset}]`
        : ''
      const formAutocomplete = target.form_autocomplete
        ? ` [form_autocomplete=${target.form_autocomplete}]`
        : ''
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
      const buttonType = target.button_type ? ` [button_type=${target.button_type}]` : ''
      const submitFormAction = target.formaction ? ` [formaction=${target.formaction}]` : ''
      const submitFormMethod = target.formmethod ? ` [formmethod=${target.formmethod}]` : ''
      const submitFormEnctype = target.formenctype ? ` [formenctype=${target.formenctype}]` : ''
      const submitFormTarget = target.formtarget ? ` [formtarget=${target.formtarget}]` : ''
      const submitNoValidate =
        typeof target.formnovalidate !== 'undefined' ? ` [formnovalidate=${target.formnovalidate}]` : ''
      const accesskey = target.accesskey
        ? ` [accesskey=${target.accesskey}]`
        : ''
      const title = target.title ? ` [title=${target.title}]` : ''
      const ariaLabel = target.aria_label ? ` [aria_label=${target.aria_label}]` : ''
      const ariaDescription = target.aria_description ? ` [aria_description=${target.aria_description}]` : ''
      const labelledby = target.labelledby ? ` [labelledby=${target.labelledby}]` : ''
      const describedby = target.describedby ? ` [describedby=${target.describedby}]` : ''
      const spellcheck =
        typeof target.spellcheck !== 'undefined' ? ` [spellcheck=${target.spellcheck}]` : ''
      const placeholder = target.placeholder
        ? ` [placeholder=${target.placeholder}]`
        : ''
      const minlength =
        typeof target.minlength !== 'undefined' ? ` [minlength=${target.minlength}]` : ''
      const maxlength =
        typeof target.maxlength !== 'undefined' ? ` [maxlength=${target.maxlength}]` : ''
      const min = typeof target.min !== 'undefined' ? ` [min=${target.min}]` : ''
      const max = typeof target.max !== 'undefined' ? ` [max=${target.max}]` : ''
      const step = target.step ? ` [step=${target.step}]` : ''
      const pattern = target.pattern ? ` [pattern=${target.pattern}]` : ''
      const checked =
        typeof target.checked !== 'undefined' ? ` [checked=${target.checked}]` : ''
      const expanded =
        typeof target.expanded !== 'undefined' ? ` [expanded=${target.expanded}]` : ''
      const pressed =
        typeof target.pressed !== 'undefined' ? ` [pressed=${target.pressed}]` : ''
      const selected =
        typeof target.selected !== 'undefined' ? ` [selected=${target.selected}]` : ''
      const multiline =
        typeof target.multiline !== 'undefined' ? ` [multiline=${target.multiline}]` : ''
      const multiselectable =
        typeof target.multiselectable !== 'undefined' ? ` [multiselectable=${target.multiselectable}]` : ''
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
      const ariaPlaceholder = target.aria_placeholder
        ? ` [aria_placeholder=${target.aria_placeholder}]`
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
      const orientation = target.orientation ? ` [orientation=${target.orientation}]` : ''
      const sort = target.sort ? ` [sort=${target.sort}]` : ''
      const level = target.level ? ` [level=${target.level}]` : ''
      const posinset = target.posinset ? ` [posinset=${target.posinset}]` : ''
      const setsize = target.setsize ? ` [setsize=${target.setsize}]` : ''
      const valuemin = target.valuemin ? ` [valuemin=${target.valuemin}]` : ''
      const valuemax = target.valuemax ? ` [valuemax=${target.valuemax}]` : ''
      const valuenow = target.valuenow ? ` [valuenow=${target.valuenow}]` : ''
      const valuetext = target.valuetext ? ` [valuetext=${target.valuetext}]` : ''
      const group = target.group ? ` [group=${target.group}]` : ''
      const description = target.description
        ? ` [description=${target.description}]`
        : ''

      return `${id}${role}${name ? ` "${name}"` : ''}${actions}${state}${cacheKey}${htmlId}${testId}${dataAction}${dataState}${blockedReason}${required}${readonly}${inert}${linkTarget}${rel}${download}${alt}${src}${inputType}${value}${nameAttr}${accept}${capture}${multiple}${selectedValues}${size}${autocomplete}${inputmode}${enterkeyhint}${autocapitalize}${dirname}${dir}${lang}${form}${formAction}${formMethod}${formTarget}${formEnctype}${formNoValidate}${formAcceptCharset}${formAutocomplete}${list}${popovertarget}${popovertargetaction}${commandfor}${command}${popover}${buttonType}${submitFormAction}${submitFormMethod}${submitFormEnctype}${submitFormTarget}${submitNoValidate}${accesskey}${title}${ariaLabel}${ariaDescription}${labelledby}${describedby}${spellcheck}${placeholder}${minlength}${maxlength}${min}${max}${step}${pattern}${checked}${expanded}${pressed}${selected}${multiline}${multiselectable}${current}${controls}${haspopup}${invalid}${ariaPlaceholder}${ariaAutocomplete}${activeDescendant}${errorMessage}${keyshortcuts}${roledescription}${busy}${live}${atomic}${relevant}${owns}${flowto}${details}${orientation}${sort}${level}${posinset}${setsize}${valuemin}${valuemax}${valuenow}${valuetext}${group}${description}`
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
