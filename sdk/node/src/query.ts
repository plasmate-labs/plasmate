/**
 * SOM query helpers for searching and traversing Semantic Object Model documents.
 */

import type { Som, SomRegion, SomElement, RegionRole, ElementRole, ElementAction } from './types';

/** Find all regions matching a given role. */
export function findByRole(som: Som, role: RegionRole): SomRegion[] {
  return som.regions.filter((r) => r.role === role);
}

/** Find an element by its stable ID across all regions. Returns undefined if not found. */
export function findById(som: Som, id: string): SomElement | undefined {
  for (const region of som.regions) {
    const found = findElementById(region.elements, id);
    if (found) return found;
  }
  return undefined;
}

/** Find an element by its original HTML id. */
export function findByHtmlId(som: Som, htmlId: string): SomElement | undefined {
  return flatElements(som).find((el) => el.html_id === htmlId);
}

function findElementById(elements: SomElement[], id: string): SomElement | undefined {
  for (const el of elements) {
    if (el.id === id) return el;
    if (el.children) {
      const found = findElementById(el.children, id);
      if (found) return found;
    }
    if (el.shadow) {
      const found = findElementById(el.shadow.elements, id);
      if (found) return found;
    }
  }
  return undefined;
}

/** Find all elements whose role matches the given element role. */
export function findByTag(som: Som, tag: ElementRole): SomElement[] {
  return flatElements(som).filter((el) => el.role === tag);
}

/** Return all interactive elements (those with a non-empty `actions` array). */
export function findInteractive(som: Som): SomElement[] {
  return flatElements(som).filter((el) => el.actions && el.actions.length > 0);
}

export interface ActionPlanItem {
  id: string;
  cache_key: string;
  role: ElementRole;
  actions: ElementAction[];
  enabled: boolean;
  html_id?: string;
  label?: string;
  href?: string;
  target?: string;
  rel?: string;
  download?: boolean | string;
  alt?: string;
  src?: string;
  name?: string;
  accept?: string;
  capture?: boolean | string;
  multiple?: boolean;
  selected_values?: string[];
  size?: number | string;
  autocomplete?: string;
  inputmode?: string;
  enterkeyhint?: string;
  autocapitalize?: string;
  dirname?: string;
  dir?: string;
  lang?: string;
  form?: string;
  form_action?: string;
  form_method?: string;
  form_target?: string;
  form_enctype?: string;
  form_novalidate?: boolean;
  form_accept_charset?: string;
  form_autocomplete?: string;
  list?: string;
  popovertarget?: string;
  popovertargetaction?: string;
  commandfor?: string;
  command?: string;
  popover?: string;
  button_type?: string;
  formaction?: string;
  formmethod?: string;
  formenctype?: string;
  formtarget?: string;
  formnovalidate?: boolean;
  accesskey?: string;
  title?: string;
  aria_label?: string;
  aria_description?: string;
  labelledby?: string;
  describedby?: string;
  spellcheck?: boolean | string;
  input_type?: string;
  value?: string;
  placeholder?: string;
  minlength?: number | string;
  maxlength?: number | string;
  min?: number | string;
  max?: number | string;
  step?: string;
  pattern?: string;
  description?: string;
  checked?: boolean | string;
  expanded?: boolean;
  pressed?: boolean;
  selected?: boolean;
  multiline?: boolean;
  multiselectable?: boolean;
  current?: boolean | string;
  controls?: string;
  haspopup?: boolean | string;
  invalid?: boolean | string;
  aria_placeholder?: string;
  aria_autocomplete?: string;
  active_descendant?: string;
  errormessage?: string;
  keyshortcuts?: string;
  roledescription?: string;
  busy?: boolean;
  live?: string;
  atomic?: boolean;
  relevant?: string;
  owns?: string;
  flowto?: string;
  details?: string;
  orientation?: string;
  sort?: string;
  level?: string;
  posinset?: string;
  setsize?: string;
  valuemin?: string;
  valuemax?: string;
  valuenow?: string;
  valuetext?: string;
  required?: boolean;
  readonly?: boolean;
  disabled?: boolean;
  inert?: boolean;
  blocked_reason?: 'disabled' | 'readonly' | 'inert';
  group?: string;
}

export interface ActionPlanIndex {
  byId: Record<string, ActionPlanItem>;
  byCacheKey: Record<string, ActionPlanItem>;
  byHtmlId: Record<string, ActionPlanItem>;
}

export interface ActionPlanSummary {
  fingerprint: string;
  enabledFingerprint: string;
  total: number;
  enabled: number;
  disabled: number;
  byRole: Record<string, number>;
  blockedReasons: Record<string, number>;
}

function compactString(value: unknown): string | undefined {
  return typeof value === 'string' && value.length > 0 ? value : undefined;
}

function stableActionPlanParts(item: Omit<ActionPlanItem, 'cache_key'> | ActionPlanItem) {
  return [
    compactString(item.id),
    compactString(item.role),
    compactString(item.label),
    [...item.actions].sort().join(',') || undefined,
    compactString(item.name),
    compactString(item.href),
    compactString(item.input_type),
    compactString(item.group),
    compactString(item.placeholder),
  ];
}

function fnv1a32(input: string): string {
  let hash = 0x811c9dc5;
  for (let index = 0; index < input.length; index += 1) {
    hash ^= input.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193);
  }
  return (hash >>> 0).toString(16).padStart(8, '0');
}

/** Return a deterministic key for caching or comparing an action target. */
export function getActionPlanCacheKey(item: Omit<ActionPlanItem, 'cache_key'> | ActionPlanItem): string {
  return `plasmate-action:v1:${fnv1a32(JSON.stringify(stableActionPlanParts(item)))}`;
}

function copyFormContext(item: Omit<ActionPlanItem, 'cache_key'>, region: SomRegion): void {
  if (region.action) item.form_action = region.action;
  if (region.method) item.form_method = region.method;
  if (region.target) item.form_target = region.target;
  if (region.enctype) item.form_enctype = region.enctype;
  if (region.novalidate !== undefined) item.form_novalidate = region.novalidate;
  if (region.accept_charset) item.form_accept_charset = region.accept_charset;
  if (region.autocomplete) item.form_autocomplete = region.autocomplete;
}

/** Return compact action targets for agent planning. */
export function getActionPlan(som: Som): ActionPlanItem[] {
  const formContextById = new Map<string, SomRegion>();
  for (const region of som.regions) {
    const regionElements: SomElement[] = [];
    collectElements(region.elements, regionElements);
    for (const el of regionElements) {
      if (el.actions?.length) {
        formContextById.set(el.id, region);
      }
    }
  }

  return findInteractive(som).map((el) => {
    const item: Omit<ActionPlanItem, 'cache_key'> = {
      id: el.id,
      role: el.role,
      actions: el.actions ?? [],
      enabled: true,
    };
    if (el.html_id) item.html_id = el.html_id;
    const label = el.label ?? el.text;
    if (label) item.label = label;
    if (el.attrs?.href) item.href = el.attrs.href;
    if (el.attrs?.target) item.target = el.attrs.target;
    if (el.attrs?.rel) item.rel = el.attrs.rel;
    if (el.attrs?.download !== undefined) item.download = el.attrs.download;
    if (el.attrs?.alt) item.alt = el.attrs.alt;
    if (el.attrs?.src) item.src = el.attrs.src;
    if (el.attrs?.name) item.name = el.attrs.name;
    if (el.attrs?.accept) item.accept = el.attrs.accept;
    if (el.attrs?.capture !== undefined) item.capture = el.attrs.capture;
    if (el.attrs?.multiple !== undefined) item.multiple = el.attrs.multiple;
    if (el.attrs?.selected_values?.length) item.selected_values = el.attrs.selected_values;
    if (el.attrs?.size !== undefined) item.size = el.attrs.size;
    if (el.attrs?.autocomplete) item.autocomplete = el.attrs.autocomplete;
    if (el.attrs?.inputmode) item.inputmode = el.attrs.inputmode;
    if (el.attrs?.enterkeyhint) item.enterkeyhint = el.attrs.enterkeyhint;
    if (el.attrs?.autocapitalize) item.autocapitalize = el.attrs.autocapitalize;
    if (el.attrs?.dirname) item.dirname = el.attrs.dirname;
    if (el.attrs?.dir) item.dir = el.attrs.dir;
    if (el.attrs?.lang) item.lang = el.attrs.lang;
    if (el.attrs?.form) item.form = el.attrs.form;
    if (el.attrs?.list) item.list = el.attrs.list;
    if (el.attrs?.popovertarget) item.popovertarget = el.attrs.popovertarget;
    if (el.attrs?.popovertargetaction) item.popovertargetaction = el.attrs.popovertargetaction;
    if (el.attrs?.commandfor) item.commandfor = el.attrs.commandfor;
    if (el.attrs?.command) item.command = el.attrs.command;
    if (el.attrs?.popover) item.popover = el.attrs.popover;
    if (el.attrs?.button_type) item.button_type = el.attrs.button_type;
    if (el.attrs?.formaction) item.formaction = el.attrs.formaction;
    if (el.attrs?.formmethod) item.formmethod = el.attrs.formmethod;
    if (el.attrs?.formenctype) item.formenctype = el.attrs.formenctype;
    if (el.attrs?.formtarget) item.formtarget = el.attrs.formtarget;
    if (el.attrs?.formnovalidate !== undefined) item.formnovalidate = el.attrs.formnovalidate;
    if (el.attrs?.accesskey) item.accesskey = el.attrs.accesskey;
    if (el.attrs?.title) item.title = el.attrs.title;
    if (el.attrs?.aria_label) item.aria_label = el.attrs.aria_label;
    if (el.attrs?.aria_description) item.aria_description = el.attrs.aria_description;
    if (el.attrs?.labelledby) item.labelledby = el.attrs.labelledby;
    if (el.attrs?.describedby) item.describedby = el.attrs.describedby;
    if (el.attrs?.spellcheck !== undefined) item.spellcheck = el.attrs.spellcheck;
    if (el.attrs?.input_type) item.input_type = el.attrs.input_type;
    if (el.attrs?.value) item.value = el.attrs.value;
    if (el.attrs?.placeholder) item.placeholder = el.attrs.placeholder;
    if (el.attrs?.minlength !== undefined) item.minlength = el.attrs.minlength;
    if (el.attrs?.maxlength !== undefined) item.maxlength = el.attrs.maxlength;
    if (el.attrs?.min !== undefined) item.min = el.attrs.min;
    if (el.attrs?.max !== undefined) item.max = el.attrs.max;
    if (el.attrs?.step) item.step = el.attrs.step;
    if (el.attrs?.pattern) item.pattern = el.attrs.pattern;
    if (el.attrs?.description) item.description = el.attrs.description;
    if (el.attrs?.checked !== undefined) {
      item.checked = el.attrs.checked;
    } else if (el.attrs?.aria?.checked !== undefined) {
      item.checked = el.attrs.aria.checked;
    }
    if (el.attrs?.aria?.expanded !== undefined) item.expanded = el.attrs.aria.expanded;
    if (el.attrs?.aria?.readonly !== undefined && el.attrs?.readonly === undefined) item.readonly = el.attrs.aria.readonly;
    if (el.attrs?.aria?.pressed !== undefined) item.pressed = el.attrs.aria.pressed;
    if (el.attrs?.aria?.selected !== undefined) item.selected = el.attrs.aria.selected;
    if (el.attrs?.aria?.current !== undefined) item.current = el.attrs.aria.current;
    if (el.attrs?.aria?.controls !== undefined) item.controls = el.attrs.aria.controls;
    if (el.attrs?.aria?.haspopup !== undefined) item.haspopup = el.attrs.aria.haspopup;
    if (el.attrs?.aria?.invalid !== undefined) item.invalid = el.attrs.aria.invalid;
    if (el.attrs?.aria?.placeholder !== undefined) item.aria_placeholder = el.attrs.aria.placeholder;
    if (el.attrs?.aria?.autocomplete !== undefined) item.aria_autocomplete = el.attrs.aria.autocomplete;
    if (el.attrs?.aria?.active_descendant !== undefined) item.active_descendant = el.attrs.aria.active_descendant;
    if (el.attrs?.aria?.errormessage !== undefined) item.errormessage = el.attrs.aria.errormessage;
    if (el.attrs?.aria?.keyshortcuts !== undefined) item.keyshortcuts = el.attrs.aria.keyshortcuts;
    if (el.attrs?.aria?.roledescription !== undefined) item.roledescription = el.attrs.aria.roledescription;
    if (el.attrs?.aria?.busy !== undefined) item.busy = el.attrs.aria.busy;
    if (el.attrs?.aria?.live !== undefined) item.live = el.attrs.aria.live;
    if (el.attrs?.aria?.atomic !== undefined) item.atomic = el.attrs.aria.atomic;
    if (el.attrs?.aria?.relevant !== undefined) item.relevant = el.attrs.aria.relevant;
    if (el.attrs?.aria?.owns !== undefined) item.owns = el.attrs.aria.owns;
    if (el.attrs?.aria?.flowto !== undefined) item.flowto = el.attrs.aria.flowto;
    if (el.attrs?.aria?.details !== undefined) item.details = el.attrs.aria.details;
    if (el.attrs?.aria?.multiline !== undefined) item.multiline = el.attrs.aria.multiline;
    if (el.attrs?.aria?.multiselectable !== undefined) item.multiselectable = el.attrs.aria.multiselectable;
    if (el.attrs?.aria?.orientation !== undefined) item.orientation = el.attrs.aria.orientation;
    if (el.attrs?.aria?.sort !== undefined) item.sort = el.attrs.aria.sort;
    if (el.attrs?.aria?.level !== undefined) item.level = el.attrs.aria.level;
    if (el.attrs?.aria?.posinset !== undefined) item.posinset = el.attrs.aria.posinset;
    if (el.attrs?.aria?.setsize !== undefined) item.setsize = el.attrs.aria.setsize;
    if (el.attrs?.aria?.valuemin !== undefined) item.valuemin = el.attrs.aria.valuemin;
    if (el.attrs?.aria?.valuemax !== undefined) item.valuemax = el.attrs.aria.valuemax;
    if (el.attrs?.aria?.valuenow !== undefined) item.valuenow = el.attrs.aria.valuenow;
    if (el.attrs?.aria?.valuetext !== undefined) item.valuetext = el.attrs.aria.valuetext;
    if (el.attrs?.required !== undefined) item.required = el.attrs.required;
    if (el.attrs?.readonly !== undefined) item.readonly = el.attrs.readonly;
    if (el.attrs?.disabled !== undefined) {
      item.disabled = el.attrs.disabled;
      if (el.attrs.disabled) {
        item.enabled = false;
        item.blocked_reason = 'disabled';
      }
    }
    if (el.attrs?.inert !== undefined) {
      item.inert = el.attrs.inert;
      if (el.attrs.inert) {
        item.enabled = false;
        item.blocked_reason = 'inert';
      }
    } else if (item.readonly && item.enabled !== false) {
      item.enabled = false;
      item.blocked_reason = 'readonly';
    }
    if (el.attrs?.group) item.group = el.attrs.group;
    const formContext = formContextById.get(el.id);
    if (formContext) copyFormContext(item, formContext);
    return {
      ...item,
      cache_key: getActionPlanCacheKey(item),
    };
  });
}

/** Find a compact action target by its deterministic cache key. */
export function findActionTargetByCacheKey(som: Som, cacheKey: string): ActionPlanItem | undefined {
  return getActionPlan(som).find((item) => item.cache_key === cacheKey);
}

/** Return compact action targets that are currently available. */
export function getEnabledActionPlan(som: Som): ActionPlanItem[] {
  return getActionPlan(som).filter((item) => item.enabled !== false);
}

/** Return action targets indexed by SOM id, cache key, and original HTML id. */
export function getActionPlanIndex(
  som: Som,
  options?: { enabledOnly?: boolean },
): ActionPlanIndex {
  const plan = options?.enabledOnly ? getEnabledActionPlan(som) : getActionPlan(som);
  const index: ActionPlanIndex = {
    byId: {},
    byCacheKey: {},
    byHtmlId: {},
  };
  for (const item of plan) {
    if (index.byId[item.id] === undefined) index.byId[item.id] = item;
    if (index.byCacheKey[item.cache_key] === undefined) index.byCacheKey[item.cache_key] = item;
    if (item.html_id && index.byHtmlId[item.html_id] === undefined) {
      index.byHtmlId[item.html_id] = item;
    }
  }
  return index;
}

/** Return a deterministic fingerprint for the current compact action plan. */
export function getActionPlanFingerprint(
  som: Som,
  options?: { enabledOnly?: boolean },
): string {
  const plan = options?.enabledOnly ? getEnabledActionPlan(som) : getActionPlan(som);
  const rows = plan
    .map((item) => [item.cache_key, item.enabled !== false, item.blocked_reason ?? null])
    .sort((left, right) => String(left[0]).localeCompare(String(right[0])));
  return `plasmate-plan:v1:${fnv1a32(JSON.stringify(rows))}`;
}

/** Return compact action-plan counts and fingerprints for replay validation. */
export function getActionPlanSummary(som: Som): ActionPlanSummary {
  const plan = getActionPlan(som);
  const byRole: Record<string, number> = {};
  const blockedReasons: Record<string, number> = {};
  let enabled = 0;
  for (const item of plan) {
    byRole[item.role] = (byRole[item.role] ?? 0) + 1;
    if (item.enabled === false) {
      const reason = item.blocked_reason ?? 'unknown';
      blockedReasons[reason] = (blockedReasons[reason] ?? 0) + 1;
    } else {
      enabled += 1;
    }
  }
  return {
    fingerprint: getActionPlanFingerprint(som),
    enabledFingerprint: getActionPlanFingerprint(som, { enabledOnly: true }),
    total: plan.length,
    enabled,
    disabled: plan.length - enabled,
    byRole: Object.fromEntries(Object.entries(byRole).sort()),
    blockedReasons: Object.fromEntries(Object.entries(blockedReasons).sort()),
  };
}

/** Find a compact action target by its SOM element id. */
export function findActionTargetById(som: Som, id: string): ActionPlanItem | undefined {
  return getActionPlan(som).find((item) => item.id === id);
}

/** Find a compact action target by its original HTML id. */
export function findActionTargetByHtmlId(som: Som, htmlId: string): ActionPlanItem | undefined {
  return getActionPlan(som).find((item) => item.html_id === htmlId);
}

/** Find all elements containing the given text (case-insensitive substring match). */
export function findByText(som: Som, text: string): SomElement[] {
  const lower = text.toLowerCase();
  return flatElements(som).filter(
    (el) => el.text != null && el.text.toLowerCase().includes(lower),
  );
}

/** Flatten all elements from all regions into a single array, recursively including children. */
export function flatElements(som: Som): SomElement[] {
  const result: SomElement[] = [];
  for (const region of som.regions) {
    collectElements(region.elements, result);
  }
  return result;
}

function collectElements(elements: SomElement[], result: SomElement[]): void {
  for (const el of elements) {
    result.push(el);
    if (el.children) {
      collectElements(el.children, result);
    }
    if (el.shadow) {
      collectElements(el.shadow.elements, result);
    }
  }
}

/**
 * Estimate the token count for a SOM document.
 * Uses the heuristic of ~4 bytes per token based on `meta.som_bytes`.
 */
export function getTokenEstimate(som: Som): number {
  return Math.ceil(som.meta.som_bytes / 4);
}
