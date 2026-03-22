import type { Som, SomElement, SomRegion, ElementRole } from './types.js';

/** Collect elements from a tree, including nested children. */
function collectElements(elements: SomElement[]): SomElement[] {
  const result: SomElement[] = [];
  for (const el of elements) {
    result.push(el);
    if (el.children) {
      result.push(...collectElements(el.children));
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

/** Get all elements that have actions (clickable, typeable, etc.). */
export function getInteractiveElements(som: Som): SomElement[] {
  return getAllElements(som).filter((el) => el.actions && el.actions.length > 0);
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
  if (!som.meta?.som_bytes || som.meta.som_bytes === 0) return 0;
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
