import type {
  Som,
  SomElement,
  SomRegion,
  ElementAction,
  ElementRole,
  SemanticHint,
} from './types.js';

/** Collect elements from a tree, including nested children. */
function collectElements(elements: SomElement[]): SomElement[] {
  const result: SomElement[] = [];
  for (const el of elements) {
    result.push(el);
    if (el.children) {
      result.push(...collectElements(el.children));
    }
    if (el.shadow) {
      result.push(...collectElements(el.shadow.elements));
    }
  }
  return result;
}

/** Flatten all elements from all regions into a single array. */
export function getAllElements(som: Som): SomElement[] {
  const result: SomElement[] = [];
  for (const region of som.regions) {
    result.push(...collectElements(region.elements));
  }
  return result;
}

/** Find all elements with a specific role. */
export function findByRole(som: Som, role: ElementRole): SomElement[] {
  return getAllElements(som).filter((el) => el.role === role);
}

/** Find an element by its SOM id. */
export function findById(som: Som, id: string): SomElement | undefined {
  return getAllElements(som).find((el) => el.id === id);
}

/** Find an element by its original HTML id. */
export function findByHtmlId(som: Som, htmlId: string): SomElement | undefined {
  return getAllElements(som).find((el) => el.html_id === htmlId);
}

/** Find elements containing text (case-insensitive substring by default). */
export function findByText(
  som: Som,
  text: string,
  options?: { exact?: boolean },
): SomElement[] {
  const all = getAllElements(som);
  if (options?.exact) {
    return all.filter((el) => el.text === text || el.label === text);
  }
  const lower = text.toLowerCase();
  return all.filter(
    (el) =>
      (el.text && el.text.toLowerCase().includes(lower)) ||
      (el.label && el.label.toLowerCase().includes(lower)),
  );
}

/** Find elements by accessible label. */
export function findByLabel(
  som: Som,
  label: string,
  options?: { exact?: boolean },
): SomElement[] {
  const all = getAllElements(som);
  if (options?.exact) {
    return all.filter((el) => el.label === label);
  }
  const lower = label.toLowerCase();
  return all.filter((el) => el.label?.toLowerCase().includes(lower));
}

/** Find all elements that expose a specific action. */
export function findByAction(som: Som, action: ElementAction): SomElement[] {
  return getAllElements(som).filter((el) => el.actions?.includes(action));
}

/** Find all elements tagged with a specific semantic hint. */
export function findByHint(som: Som, hint: SemanticHint): SomElement[] {
  return getAllElements(som).filter((el) => el.hints?.includes(hint));
}

/** Get all elements that have actions (clickable, typeable, etc.). */
export function getInteractiveElements(som: Som): SomElement[] {
  return getAllElements(som).filter((el) => el.actions && el.actions.length > 0);
}

export interface ActionPlanItem {
  id: string;
  cache_key: string;
  role: ElementRole;
  actions: ElementAction[];
  enabled: boolean;
  label?: string;
  html_id?: string;
  href?: string;
  target?: string;
  rel?: string;
  hreflang?: string;
  type?: string;
  referrerpolicy?: string;
  download?: boolean | string;
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
  source_role?: string;
  test_id?: string;
  spellcheck?: boolean | string;
  draggable?: boolean | string;
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
  aria_label?: string;
  labelledby?: string;
  describedby?: string;
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
  grabbed?: boolean;
  dropeffect?: string;
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
  byTestId: Record<string, ActionPlanItem>;
  byLabel: Record<string, ActionPlanItem>;
  byRole: Record<string, ActionPlanItem[]>;
  byAction: Record<string, ActionPlanItem[]>;
}

export type ActionTargetLookupKey = 'auto' | 'id' | 'cache_key' | 'html_id' | 'test_id' | 'label';

export interface ActionTargetLookupOptions {
  by?: ActionTargetLookupKey;
  enabledOnly?: boolean;
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
    for (const el of collectElements(region.elements)) {
      if (el.actions?.length) {
        formContextById.set(el.id, region);
      }
    }
  }

  return getInteractiveElements(som).map((el) => {
    const item: Omit<ActionPlanItem, 'cache_key'> = {
      id: el.id,
      role: el.role,
      actions: el.actions ?? [],
      enabled: true,
    };
    const label = el.label ?? el.text;
    if (label) item.label = label;
    if (el.html_id) item.html_id = el.html_id;
    if (el.attrs?.href) item.href = el.attrs.href;
    if (el.attrs?.target) item.target = el.attrs.target;
    if (el.attrs?.rel) item.rel = el.attrs.rel;
    if (el.attrs?.hreflang) item.hreflang = el.attrs.hreflang;
    if (el.attrs?.type) item.type = el.attrs.type;
    if (el.attrs?.referrerpolicy) item.referrerpolicy = el.attrs.referrerpolicy;
    if (el.attrs?.download !== undefined) item.download = el.attrs.download;
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
    if (el.attrs?.source_role) item.source_role = el.attrs.source_role;
    if (el.attrs?.test_id) item.test_id = el.attrs.test_id;
    if (el.attrs?.spellcheck !== undefined) item.spellcheck = el.attrs.spellcheck;
    if (el.attrs?.draggable !== undefined) item.draggable = el.attrs.draggable;
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
    if (el.attrs?.aria?.label !== undefined) item.aria_label = el.attrs.aria.label;
    if (el.attrs?.aria?.labelledby !== undefined) item.labelledby = el.attrs.aria.labelledby;
    if (el.attrs?.aria?.describedby !== undefined) item.describedby = el.attrs.aria.describedby;
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
    if (el.attrs?.aria?.grabbed !== undefined) item.grabbed = el.attrs.aria.grabbed;
    if (el.attrs?.aria?.dropeffect !== undefined) item.dropeffect = el.attrs.aria.dropeffect;
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

/** Return compact action targets that are currently safe to offer. */
export function getEnabledActionPlan(som: Som): ActionPlanItem[] {
  return getActionPlan(som).filter((item) => item.enabled !== false);
}

/** Return action targets indexed for replay and grouped by role/action. */
export function getActionPlanIndex(
  som: Som,
  options?: { enabledOnly?: boolean },
): ActionPlanIndex {
  const plan = options?.enabledOnly ? getEnabledActionPlan(som) : getActionPlan(som);
  const index: ActionPlanIndex = {
    byId: {},
    byCacheKey: {},
    byHtmlId: {},
    byTestId: {},
    byLabel: {},
    byRole: {},
    byAction: {},
  };
  for (const item of plan) {
    if (index.byId[item.id] === undefined) index.byId[item.id] = item;
    if (index.byCacheKey[item.cache_key] === undefined) index.byCacheKey[item.cache_key] = item;
    if (item.html_id && index.byHtmlId[item.html_id] === undefined) {
      index.byHtmlId[item.html_id] = item;
    }
    if (item.test_id && index.byTestId[item.test_id] === undefined) {
      index.byTestId[item.test_id] = item;
    }
    if (item.label && index.byLabel[item.label] === undefined) {
      index.byLabel[item.label] = item;
    }
    (index.byRole[item.role] ??= []).push(item);
    for (const action of item.actions) {
      (index.byAction[action] ??= []).push(item);
    }
  }
  return index;
}

/** Find a compact action target by id, cache key, HTML id, test id, label, or auto-detected replay id. */
export function findActionTarget(
  som: Som,
  value: string,
  options: ActionTargetLookupOptions = {},
): ActionPlanItem | undefined {
  const index = getActionPlanIndex(som, { enabledOnly: options.enabledOnly });
  const by = options.by ?? 'auto';
  if (by === 'id') return index.byId[value];
  if (by === 'cache_key') return index.byCacheKey[value];
  if (by === 'html_id') return index.byHtmlId[value];
  if (by === 'test_id') return index.byTestId[value];
  if (by === 'label') return index.byLabel[value];
  return (
    index.byId[value] ??
    index.byCacheKey[value] ??
    index.byHtmlId[value] ??
    index.byTestId[value]
  );
}

/** Find compact action targets by accessible label. */
export function findActionTargetsByLabel(
  som: Som,
  label: string,
  options?: { exact?: boolean; enabledOnly?: boolean },
): ActionPlanItem[] {
  const plan = options?.enabledOnly ? getEnabledActionPlan(som) : getActionPlan(som);
  if (options?.exact) {
    return plan.filter((item) => item.label === label);
  }
  const lower = label.toLowerCase();
  return plan.filter((item) => item.label?.toLowerCase().includes(lower));
}

/** Find compact action targets by exact SOM role. */
export function findActionTargetsByRole(
  som: Som,
  role: ElementRole,
  options?: { enabledOnly?: boolean },
): ActionPlanItem[] {
  return getActionPlanIndex(som, { enabledOnly: options?.enabledOnly }).byRole[role] ?? [];
}

/** Find compact action targets that expose the requested action. */
export function findActionTargetsByAction(
  som: Som,
  action: ElementAction,
  options?: { enabledOnly?: boolean },
): ActionPlanItem[] {
  return getActionPlanIndex(som, { enabledOnly: options?.enabledOnly }).byAction[action] ?? [];
}

/** Find a compact action target by its SOM element id. */
export function findActionTargetById(
  som: Som,
  id: string,
  options: Pick<ActionTargetLookupOptions, 'enabledOnly'> = {},
): ActionPlanItem | undefined {
  return findActionTarget(som, id, { ...options, by: 'id' });
}

/** Find a compact action target by its deterministic cache key. */
export function findActionTargetByCacheKey(
  som: Som,
  cacheKey: string,
  options: Pick<ActionTargetLookupOptions, 'enabledOnly'> = {},
): ActionPlanItem | undefined {
  return findActionTarget(som, cacheKey, { ...options, by: 'cache_key' });
}

/** Find a compact action target by its original HTML id. */
export function findActionTargetByHtmlId(
  som: Som,
  htmlId: string,
  options: Pick<ActionTargetLookupOptions, 'enabledOnly'> = {},
): ActionPlanItem | undefined {
  return findActionTarget(som, htmlId, { ...options, by: 'html_id' });
}

/** Find a compact action target by its test locator attribute. */
export function findActionTargetByTestId(
  som: Som,
  testId: string,
  options: Pick<ActionTargetLookupOptions, 'enabledOnly'> = {},
): ActionPlanItem | undefined {
  return findActionTarget(som, testId, { ...options, by: 'test_id' });
}

/** Find the first compact action target by exact accessible label. */
export function findActionTargetByLabel(
  som: Som,
  label: string,
  options: Pick<ActionTargetLookupOptions, 'enabledOnly'> = {},
): ActionPlanItem | undefined {
  return findActionTarget(som, label, { ...options, by: 'label' });
}

/** Extract all links with their text and URLs. */
export function getLinks(som: Som): Array<{ text: string; href: string; id: string }> {
  return findByRole(som, 'link')
    .filter((el) => el.attrs?.href)
    .map((el) => ({
      text: el.text ?? el.label ?? '',
      href: el.attrs!.href!,
      id: el.id,
    }));
}

/** Get all form regions. */
export function getForms(som: Som): SomRegion[] {
  return som.regions.filter((r) => r.role === 'form');
}

const INPUT_ROLES: ElementRole[] = ['text_input', 'textarea', 'select', 'checkbox', 'radio'];

/** Get all input-type elements. */
export function getInputs(som: Som): SomElement[] {
  return getAllElements(som).filter((el) => INPUT_ROLES.includes(el.role));
}

/** Extract heading hierarchy. */
export function getHeadings(som: Som): Array<{ level: number; text: string; id: string }> {
  return findByRole(som, 'heading').map((el) => ({
    level: el.attrs?.level ?? 1,
    text: el.text ?? '',
    id: el.id,
  }));
}

/** Extract all visible text content, joined with newlines. */
export function getText(som: Som): string {
  return getAllElements(som)
    .map((el) => el.text ?? el.label ?? '')
    .filter(Boolean)
    .join('\n');
}

/** Extract text grouped by region. */
export function getTextByRegion(
  som: Som,
): Array<{ region: string; role: string; text: string }> {
  return som.regions.map((r) => ({
    region: r.id,
    role: r.role,
    text: collectElements(r.elements)
      .map((el) => el.text ?? el.label ?? '')
      .filter(Boolean)
      .join('\n'),
  }));
}

/** Return html_bytes / som_bytes from meta. */
export function getCompressionRatio(som: Som): number {
  if (!som.meta?.som_bytes || som.meta.som_bytes === 0) return Number.POSITIVE_INFINITY;
  return som.meta.html_bytes / som.meta.som_bytes;
}

/** Convert SOM to a readable markdown representation. */
export function toMarkdown(som: Som): string {
  const lines: string[] = [];

  if (som.title) {
    lines.push(`# ${som.title}`);
    lines.push('');
  }

  for (const region of som.regions) {
    if (region.role === 'form') {
      const action = region.action ? ` (${region.method ?? 'POST'} ${region.action})` : '';
      lines.push(`### Form${action}`);
      lines.push('');
      for (const el of collectElements(region.elements)) {
        if (INPUT_ROLES.includes(el.role)) {
          const label = el.label ?? el.attrs?.placeholder ?? el.role;
          lines.push(`- **${label}** (${el.role})`);
        } else if (el.role === 'button') {
          lines.push(`- [${el.text ?? 'Button'}] (button)`);
        }
      }
      lines.push('');
      continue;
    }

    for (const el of collectElements(region.elements)) {
      switch (el.role) {
        case 'heading': {
          const level = el.attrs?.level ?? 1;
          lines.push(`${'#'.repeat(Math.min(level + 1, 6))} ${el.text ?? ''}`);
          lines.push('');
          break;
        }
        case 'paragraph':
          if (el.text) {
            lines.push(el.text);
            lines.push('');
          }
          break;
        case 'link':
          lines.push(`- [${el.text ?? el.label ?? ''}](${el.attrs?.href ?? '#'})`);
          break;
        case 'image':
          lines.push(`![${el.attrs?.alt ?? ''}](${el.attrs?.src ?? ''})`);
          lines.push('');
          break;
        case 'list': {
          const items = el.attrs?.items ?? [];
          for (const item of items) {
            lines.push(`- ${item.text}`);
          }
          if (items.length) lines.push('');
          break;
        }
        default:
          if (el.text) {
            lines.push(el.text);
            lines.push('');
          }
      }
    }
  }

  return lines.join('\n').trim() + '\n';
}

/** Generic filter across all elements. */
export function filter(som: Som, predicate: (el: SomElement) => boolean): SomElement[] {
  return getAllElements(som).filter(predicate);
}
