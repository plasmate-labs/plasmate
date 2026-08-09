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
  if (o.meta == null || typeof o.meta !== 'object') return false;
  return true;
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
