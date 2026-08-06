import { describe, it } from 'node:test';
import * as assert from 'node:assert/strict';
import { chmodSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { Plasmate } from './index';

type ToolCall = { name: string; args: Record<string, unknown> };

describe('read selectors', () => {
  it('forwards selector options through fetch and text reads', async () => {
    const browser = new Plasmate({ binary: 'unused' });
    const calls: ToolCall[] = [];
    const client = browser as unknown as {
      callTool: (name: string, args: Record<string, unknown>) => Promise<unknown>;
    };
    client.callTool = async (name, args) => {
      calls.push({ name, args });
      return name === 'extract_text' ? 'text' : {};
    };

    await browser.fetchPage('fixture', { selector: 'main' });
    await browser.som('fixture', { selector: 'interactive' });
    await browser.extractText('fixture', { selector: 'content' });

    assert.deepEqual(calls, [
      { name: 'fetch_page', args: { url: 'fixture', selector: 'main' } },
      { name: 'fetch_page', args: { url: 'fixture', selector: 'interactive' } },
      { name: 'extract_text', args: { url: 'fixture', selector: 'content' } },
    ]);
    browser.close();
  });
});

describe('protocol lifecycle', () => {
  it('uses a true notification for initialized', async () => {
    const directory = mkdtempSync(join(tmpdir(), 'plasmate-node-sdk-'));
    const fixture = join(directory, 'mcp-fixture.js');
    writeFileSync(
      fixture,
      `#!/usr/bin/env node
import { createInterface } from 'node:readline';

const send = (value) => process.stdout.write(JSON.stringify(value) + '\\n');
const input = createInterface({ input: process.stdin });

input.on('line', (line) => {
  const request = JSON.parse(line);
  if (request.method === 'initialize') {
    send({ jsonrpc: '2.0', id: request.id, result: { protocolVersion: '2024-11-05' } });
  } else if (request.method === 'notifications/initialized') {
    if (Object.hasOwn(request, 'id')) process.exit(42);
  } else if (request.method === 'tools/call') {
    send({
      jsonrpc: '2.0',
      id: request.id,
      result: { content: [{ type: 'text', text: JSON.stringify({ ok: true }) }] },
    });
  }
});
`,
      'utf8',
    );
    chmodSync(fixture, 0o755);

    const browser = new Plasmate({ binary: fixture });
    try {
      assert.deepEqual(await browser.fetchPage('fixture'), { ok: true });
    } finally {
      browser.close();
      rmSync(directory, { recursive: true, force: true });
    }
  });

  it('keeps a bounded message for empty tool errors', async () => {
    const directory = mkdtempSync(join(tmpdir(), 'plasmate-node-sdk-'));
    const fixture = join(directory, 'mcp-fixture.js');
    writeFileSync(
      fixture,
      `#!/usr/bin/env node
import { createInterface } from 'node:readline';

const send = (value) => process.stdout.write(JSON.stringify(value) + '\\n');
const input = createInterface({ input: process.stdin });

input.on('line', (line) => {
  const request = JSON.parse(line);
  if (request.method === 'initialize') {
    send({ jsonrpc: '2.0', id: request.id, result: { protocolVersion: '2024-11-05' } });
  } else if (request.method === 'tools/call') {
    send({
      jsonrpc: '2.0',
      id: request.id,
      result: { isError: true, content: [{ type: 'text', text: '' }] },
    });
  }
});
`,
      'utf8',
    );
    chmodSync(fixture, 0o755);

    const browser = new Plasmate({ binary: fixture });
    try {
      await assert.rejects(browser.fetchPage('fixture'), (error: unknown) => {
        return error instanceof Error && error.message === 'Unknown error';
      });
    } finally {
      browser.close();
      rmSync(directory, { recursive: true, force: true });
    }
  });
});
