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
  href?: string;
  name?: string;
  autocomplete?: string;
  inputmode?: string;
  enterkeyhint?: string;
  input_type?: string;
  value?: string;
  placeholder?: string;
  minlength?: number | string;
  maxlength?: number | string;
  pattern?: string;
  description?: string;
  checked?: boolean | string;
  expanded?: boolean;
  pressed?: boolean;
  selected?: boolean;
  current?: boolean | string;
  controls?: string;
  haspopup?: boolean | string;
  invalid?: boolean | string;
  aria_autocomplete?: string;
  active_descendant?: string;
  required?: boolean;
  readonly?: boolean;
  disabled?: boolean;
  blocked_reason?: 'disabled' | 'readonly';
  group?: string;
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

/** Return compact action targets for agent planning. */
export function getActionPlan(som: Som): ActionPlanItem[] {
  return getInteractiveElements(som).map((el) => {
    const item: Omit<ActionPlanItem, 'cache_key'> = {
      id: el.id,
      role: el.role,
      actions: el.actions ?? [],
      enabled: true,
    };
    const label = el.label ?? el.text;
    if (label) item.label = label;
    if (el.attrs?.href) item.href = el.attrs.href;
    if (el.attrs?.name) item.name = el.attrs.name;
    if (el.attrs?.autocomplete) item.autocomplete = el.attrs.autocomplete;
    if (el.attrs?.inputmode) item.inputmode = el.attrs.inputmode;
    if (el.attrs?.enterkeyhint) item.enterkeyhint = el.attrs.enterkeyhint;
    if (el.attrs?.input_type) item.input_type = el.attrs.input_type;
    if (el.attrs?.value) item.value = el.attrs.value;
    if (el.attrs?.placeholder) item.placeholder = el.attrs.placeholder;
    if (el.attrs?.minlength !== undefined) item.minlength = el.attrs.minlength;
    if (el.attrs?.maxlength !== undefined) item.maxlength = el.attrs.maxlength;
    if (el.attrs?.pattern) item.pattern = el.attrs.pattern;
    if (el.attrs?.description) item.description = el.attrs.description;
    if (el.attrs?.checked !== undefined) {
      item.checked = el.attrs.checked;
    } else if (el.attrs?.aria?.checked !== undefined) {
      item.checked = el.attrs.aria.checked;
    }
    if (el.attrs?.aria?.expanded !== undefined) item.expanded = el.attrs.aria.expanded;
    if (el.attrs?.aria?.pressed !== undefined) item.pressed = el.attrs.aria.pressed;
    if (el.attrs?.aria?.selected !== undefined) item.selected = el.attrs.aria.selected;
    if (el.attrs?.aria?.current !== undefined) item.current = el.attrs.aria.current;
    if (el.attrs?.aria?.controls !== undefined) item.controls = el.attrs.aria.controls;
    if (el.attrs?.aria?.haspopup !== undefined) item.haspopup = el.attrs.aria.haspopup;
    if (el.attrs?.aria?.invalid !== undefined) item.invalid = el.attrs.aria.invalid;
    if (el.attrs?.aria?.autocomplete !== undefined) item.aria_autocomplete = el.attrs.aria.autocomplete;
    if (el.attrs?.aria?.active_descendant !== undefined) item.active_descendant = el.attrs.aria.active_descendant;
    if (el.attrs?.required !== undefined) item.required = el.attrs.required;
    if (el.attrs?.readonly !== undefined) item.readonly = el.attrs.readonly;
    if (el.attrs?.disabled !== undefined) {
      item.disabled = el.attrs.disabled;
      if (el.attrs.disabled) {
        item.enabled = false;
        item.blocked_reason = 'disabled';
      }
    } else if (el.attrs?.readonly) {
      item.enabled = false;
      item.blocked_reason = 'readonly';
    }
    if (el.attrs?.group) item.group = el.attrs.group;
    return {
      ...item,
      cache_key: getActionPlanCacheKey(item),
    };
  });
}

/** Extract all links with their text and URLs. */
export function getLinks(som: Som): Array<{ text: string; href: string; id: string }> {
  return findByRole(som, 'link')
    .filter((el) => el.attrs?.href)
    .map((el) => ({
      text: el.text ?? '',
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
          lines.push(`- [${el.text ?? ''}](${el.attrs?.href ?? '#'})`);
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
