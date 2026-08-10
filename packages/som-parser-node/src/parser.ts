import type { Som } from './types.js';

/**
 * Parse a JSON string or plain object into a typed Som.
 * Throws if the input is not valid SOM.
 */
export function parseSom(input: string | object): Som {
  const obj: unknown = typeof input === 'string' ? JSON.parse(input) : input;
  if (!isValidSom(obj)) {
    throw new Error('Invalid SOM: missing required fields (som_version, url, title, regions, meta)');
  }
  return obj;
}

/**
 * Type guard that checks whether an unknown value conforms to the SOM schema.
 */
export function isValidSom(input: unknown): input is Som {
  if (input == null || typeof input !== 'object') return false;
  const o = input as Record<string, unknown>;
  if (typeof o.som_version !== 'string') return false;
  if (typeof o.url !== 'string') return false;
  if (typeof o.title !== 'string') return false;
  if (!Array.isArray(o.regions)) return false;
  if (!isValidMeta(o.meta)) return false;
  if (!o.regions.every(isValidRegion)) return false;
  return true;
}

function isRecord(input: unknown): input is Record<string, unknown> {
  return input !== null && typeof input === 'object' && !Array.isArray(input);
}

function isNonNegativeInteger(input: unknown): input is number {
  return typeof input === 'number' && Number.isInteger(input) && input >= 0;
}

function isValidMeta(input: unknown): boolean {
  if (!isRecord(input)) return false;
  return (
    isNonNegativeInteger(input.html_bytes) &&
    isNonNegativeInteger(input.som_bytes) &&
    isNonNegativeInteger(input.element_count) &&
    isNonNegativeInteger(input.interactive_count)
  );
}

function isValidElementTree(input: unknown): boolean {
  if (!isRecord(input) || typeof input.id !== 'string' || typeof input.role !== 'string') {
    return false;
  }

  if (
    input.actions !== undefined &&
    (!Array.isArray(input.actions) || !input.actions.every((action) => typeof action === 'string'))
  ) {
    return false;
  }
  if (
    input.hints !== undefined &&
    (!Array.isArray(input.hints) || !input.hints.every((hint) => typeof hint === 'string'))
  ) {
    return false;
  }
  if (
    input.children !== undefined &&
    (!Array.isArray(input.children) || !input.children.every(isValidElementTree))
  ) {
    return false;
  }

  if (input.shadow !== undefined) {
    if (
      !isRecord(input.shadow) ||
      typeof input.shadow.mode !== 'string' ||
      !Array.isArray(input.shadow.elements) ||
      !input.shadow.elements.every(isValidElementTree)
    ) {
      return false;
    }
  }

  return true;
}

function isValidRegion(input: unknown): boolean {
  return (
    isRecord(input) &&
    typeof input.id === 'string' &&
    typeof input.role === 'string' &&
    Array.isArray(input.elements) &&
    input.elements.every(isValidElementTree)
  );
}

function extractJsonObjects(text: string): unknown[] {
  const objects: unknown[] = [];

  for (let start = 0; start < text.length; start += 1) {
    if (text[start] !== '{') continue;

    let depth = 0;
    let inString = false;
    let escaped = false;
    for (let index = start; index < text.length; index += 1) {
      const character = text[index];
      if (inString) {
        if (escaped) {
          escaped = false;
        } else if (character === '\\') {
          escaped = true;
        } else if (character === '"') {
          inString = false;
        }
        continue;
      }

      if (character === '"') {
        inString = true;
      } else if (character === '{') {
        depth += 1;
      } else if (character === '}') {
        depth -= 1;
        if (depth === 0) {
          try {
            objects.push(JSON.parse(text.slice(start, index + 1)));
          } catch {
            // Keep scanning for a later complete JSON object.
          }
          start = index;
          break;
        }
      }
    }
  }

  return objects;
}

/**
 * Parse raw Plasmate CLI JSON output into a typed Som.
 * Handles cases where the CLI may emit extra text before or after the JSON.
 */
export function fromPlasmate(jsonOutput: string): Som {
  const unwrap = (value: unknown): Som => {
    if (
      value != null &&
      typeof value === 'object' &&
      'som' in value &&
      !('som_version' in value)
    ) {
      return parseSom((value as { som: unknown }).som as object);
    }
    return parseSom(value as object);
  };

  // Try direct parse first
  try {
    return unwrap(JSON.parse(jsonOutput));
  } catch {
    // Fall back: scan complete objects and ignore non-SOM progress records.
    const objects = extractJsonObjects(jsonOutput);
    if (objects.length === 0) {
      throw new Error('No JSON object found in Plasmate output');
    }

    let result: Som | undefined;
    let lastError: unknown;
    for (const value of objects) {
      try {
        result = unwrap(value);
      } catch (error) {
        lastError = error;
      }
    }
    if (result) return result;
    if (lastError !== undefined) throw lastError;
    throw new Error('Invalid SOM in Plasmate output');
  }
}
