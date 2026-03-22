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

/**
 * Parse raw Plasmate CLI JSON output into a typed Som.
 * Handles cases where the CLI may emit extra text before or after the JSON.
 */
export function fromPlasmate(jsonOutput: string): Som {
  // Try direct parse first
  try {
    return parseSom(jsonOutput);
  } catch {
    // Fall back: look for the first { ... } block
    const start = jsonOutput.indexOf('{');
    const end = jsonOutput.lastIndexOf('}');
    if (start === -1 || end === -1 || end <= start) {
      throw new Error('No JSON object found in Plasmate output');
    }
    return parseSom(jsonOutput.slice(start, end + 1));
  }
}
