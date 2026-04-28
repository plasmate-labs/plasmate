/**
 * Plasmate - Agent-native headless browser SDK for Node.js
 *
 * Communicates with the `plasmate mcp` process over stdio using JSON-RPC 2.0.
 *
 * @example
 * ```typescript
 * import { Plasmate } from 'plasmate';
 *
 * const browser = new Plasmate();
 *
 * // One-shot: fetch a page as SOM
 * const som = await browser.fetchPage('https://example.com');
 * console.log(som.title, som.regions.length);
 *
 * // Interactive: open, click, evaluate, close
 * const session = await browser.openPage('https://news.ycombinator.com');
 * const titles = await browser.evaluate(session.sessionId, 'document.title');
 * await browser.closePage(session.sessionId);
 *
 * // Clean up
 * browser.close();
 * ```
 */

import { ChildProcess, spawn } from 'child_process';
import { createInterface, Interface } from 'readline';
import { EventEmitter } from 'events';

// ---- SOM types (from specs/som-schema.json) ----

export type {
  Som,
  SomRegion,
  SomElement,
  SomElementAttrs,
  SomMeta,
  StructuredData,
  RegionRole,
  ElementRole,
  ElementAction,
  SemanticHint,
  SelectOption,
  ListItem,
  LinkElement,
  ShadowRoot,
} from './types';

// ---- SOM query helpers ----

export {
  findByRole,
  findById,
  findByTag,
  findInteractive,
  findByText,
  flatElements,
  getTokenEstimate,
} from './query';

import type { Som } from './types';

export interface PageSession {
  sessionId: string;
  som: Som;
}

export interface PlasmateOptions {
  /** Path to the plasmate binary. Default: "plasmate" (found in PATH) */
  binary?: string;
  /** Maximum time to wait for a response, in milliseconds. Default: 30000 */
  timeout?: number;
}

// ---- JSON-RPC ----

interface JsonRpcRequest {
  jsonrpc: '2.0';
  id: number;
  method: string;
  params?: unknown;
}

interface JsonRpcResponse {
  jsonrpc: '2.0';
  id: number | null;
  result?: unknown;
  error?: { code: number; message: string; data?: unknown };
}

// ---- Client ----

export class Plasmate extends EventEmitter {
  private process: ChildProcess | null = null;
  private readline: Interface | null = null;
  private nextId = 1;
  private pending = new Map<number, {
    resolve: (value: unknown) => void;
    reject: (error: Error) => void;
    timer: ReturnType<typeof setTimeout>;
  }>();
  private initialized = false;
  private initPromise: Promise<void> | null = null;
  private binary: string;
  private timeout: number;

  constructor(options: PlasmateOptions = {}) {
    super();
    this.binary = options.binary ?? 'plasmate';
    this.timeout = options.timeout ?? 30000;
  }

  // ---- Lifecycle ----

  private async ensureStarted(): Promise<void> {
    if (this.initialized) return;
    if (this.initPromise) return this.initPromise;

    this.initPromise = this.start();
    return this.initPromise;
  }

  private async start(): Promise<void> {
    this.process = spawn(this.binary, ['mcp'], {
      stdio: ['pipe', 'pipe', 'pipe'],
    });

    this.process.on('error', (err) => {
      this.emit('error', err);
    });

    this.process.on('exit', (code) => {
      this.initialized = false;
      this.initPromise = null;
      // Reject all pending requests
      for (const [, entry] of this.pending) {
        clearTimeout(entry.timer);
        entry.reject(new Error(`Plasmate process exited with code ${code}`));
      }
      this.pending.clear();
    });

    // Capture stderr for debugging
    this.process.stderr?.on('data', (data: Buffer) => {
      this.emit('log', data.toString());
    });

    // Parse stdout as newline-delimited JSON
    this.readline = createInterface({ input: this.process.stdout! });
    this.readline.on('line', (line: string) => {
      if (!line.trim()) return;
      try {
        const response: JsonRpcResponse = JSON.parse(line);
        if (response.id != null && this.pending.has(response.id)) {
          const entry = this.pending.get(response.id)!;
          this.pending.delete(response.id);
          clearTimeout(entry.timer);
          if (response.error) {
            entry.reject(new Error(response.error.message));
          } else {
            entry.resolve(response.result);
          }
        }
      } catch {
        // Ignore non-JSON lines (e.g. tracing output)
      }
    });

    // Send initialize
    await this.rpc('initialize', {
      protocolVersion: '2024-11-05',
      capabilities: {},
      clientInfo: { name: 'plasmate-node-sdk', version: '0.2.0' },
    });

    // Send initialized notification (no response expected)
    this.send({
      jsonrpc: '2.0',
      id: this.nextId++,
      method: 'notifications/initialized',
    });

    this.initialized = true;
  }

  /** Shut down the plasmate process. */
  close(): void {
    if (this.process) {
      this.process.kill();
      this.process = null;
    }
    this.readline?.close();
    this.readline = null;
    this.initialized = false;
    this.initPromise = null;
  }

  // ---- Transport ----

  private send(request: JsonRpcRequest): void {
    if (!this.process?.stdin?.writable) {
      throw new Error('Plasmate process is not running');
    }
    this.process.stdin.write(JSON.stringify(request) + '\n');
  }

  private rpc(method: string, params?: unknown): Promise<unknown> {
    return new Promise((resolve, reject) => {
      const id = this.nextId++;
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(`Timeout waiting for response to ${method} (${this.timeout}ms)`));
      }, this.timeout);

      this.pending.set(id, { resolve, reject, timer });
      this.send({ jsonrpc: '2.0', id, method, params });
    });
  }

  private async callTool(name: string, args: Record<string, unknown>): Promise<unknown> {
    await this.ensureStarted();
    const result = await this.rpc('tools/call', { name, arguments: args }) as {
      content?: Array<{ type: string; text: string }>;
      isError?: boolean;
    };

    if (result.isError) {
      const msg = result.content?.[0]?.text ?? 'Unknown error';
      throw new Error(msg);
    }

    const text = result.content?.[0]?.text;
    if (!text) return null;

    // Try to parse as JSON, fall back to raw text
    try {
      return JSON.parse(text);
    } catch {
      return text;
    }
  }

  // ---- Stateless Tools ----

  /**
   * Convenience alias for `fetchPage` — fetch a page and return its typed SOM.
   *
   * @param url - URL to fetch
   * @param options.budget - Maximum output tokens (SOM will be truncated)
   * @param options.javascript - Enable JS execution (default: true)
   */
  async som(url: string, options?: {
    budget?: number;
    javascript?: boolean;
  }): Promise<Som> {
    return this.fetchPage(url, options);
  }

  /**
   * Fetch a page and return its Semantic Object Model.
   *
   * @param url - URL to fetch
   * @param options.budget - Maximum output tokens (SOM will be truncated)
   * @param options.javascript - Enable JS execution (default: true)
   */
  async fetchPage(url: string, options?: {
    budget?: number;
    javascript?: boolean;
  }): Promise<Som> {
    const args: Record<string, unknown> = { url };
    if (options?.budget != null) args.budget = options.budget;
    if (options?.javascript != null) args.javascript = options.javascript;
    return await this.callTool('fetch_page', args) as Som;
  }

  /**
   * Fetch a page and return clean, readable text only.
   *
   * @param url - URL to fetch
   * @param options.maxChars - Maximum characters to return
   */
  async extractText(url: string, options?: {
    maxChars?: number;
  }): Promise<string> {
    const args: Record<string, unknown> = { url };
    if (options?.maxChars != null) args.max_chars = options.maxChars;
    return await this.callTool('extract_text', args) as string;
  }

  // ---- Stateful Tools ----

  /**
   * Open a page in a persistent browser session.
   * Returns a session ID and the initial SOM.
   *
   * @param url - URL to open
   */
  async openPage(url: string): Promise<PageSession> {
    const result = await this.callTool('open_page', { url }) as {
      session_id: string;
      som: Som;
    };
    return { sessionId: result.session_id, som: result.som };
  }

  /**
   * Execute JavaScript in the page context.
   *
   * @param sessionId - Session ID from openPage
   * @param expression - JavaScript expression to evaluate
   */
  async evaluate(sessionId: string, expression: string): Promise<unknown> {
    return await this.callTool('evaluate', {
      session_id: sessionId,
      expression,
    });
  }

  /**
   * Click an element by its SOM element ID.
   * Returns the updated SOM after the click.
   *
   * @param sessionId - Session ID from openPage
   * @param elementId - Element ID from SOM (e.g. 'e5')
   */
  async click(sessionId: string, elementId: string): Promise<Som> {
    return await this.callTool('click', {
      session_id: sessionId,
      element_id: elementId,
    }) as Som;
  }

  /**
   * Close a browser session and free resources.
   *
   * @param sessionId - Session ID to close
   */
  async closePage(sessionId: string): Promise<void> {
    await this.callTool('close_page', { session_id: sessionId });
  }
}

export default Plasmate;
