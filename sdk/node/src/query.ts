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
  label?: string;
  href?: string;
  name?: string;
  input_type?: string;
  value?: string;
  placeholder?: string;
  description?: string;
  checked?: boolean | string;
  required?: boolean;
  disabled?: boolean;
  blocked_reason?: 'disabled';
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
  return findInteractive(som).map((el) => {
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
    if (el.attrs?.input_type) item.input_type = el.attrs.input_type;
    if (el.attrs?.value) item.value = el.attrs.value;
    if (el.attrs?.placeholder) item.placeholder = el.attrs.placeholder;
    if (el.attrs?.description) item.description = el.attrs.description;
    if (el.attrs?.checked !== undefined) {
      item.checked = el.attrs.checked;
    } else if (el.attrs?.aria?.checked !== undefined) {
      item.checked = el.attrs.aria.checked;
    }
    if (el.attrs?.required !== undefined) item.required = el.attrs.required;
    if (el.attrs?.disabled !== undefined) {
      item.disabled = el.attrs.disabled;
      if (el.attrs.disabled) {
        item.enabled = false;
        item.blocked_reason = 'disabled';
      }
    }
    if (el.attrs?.group) item.group = el.attrs.group;
    return {
      ...item,
      cache_key: getActionPlanCacheKey(item),
    };
  });
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
