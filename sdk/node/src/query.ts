/**
 * SOM query helpers for searching and traversing Semantic Object Model documents.
 */

import type { Som, SomRegion, SomElement, RegionRole, ElementRole } from './types';

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
  }
}

/**
 * Estimate the token count for a SOM document.
 * Uses the heuristic of ~4 bytes per token based on `meta.som_bytes`.
 */
export function getTokenEstimate(som: Som): number {
  return Math.ceil(som.meta.som_bytes / 4);
}
