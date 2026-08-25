import { describe, it } from 'node:test';
import * as assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { chmodSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
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

describe('openPage payload', () => {
  it('builds session.som from the flat MCP open_page fields', async () => {
    const browser = new Plasmate({ binary: 'unused' });
    const client = browser as unknown as {
      callTool: (name: string, args: Record<string, unknown>) => Promise<unknown>;
    };
    client.callTool = async (name, args) => {
      assert.equal(name, 'open_page');
      assert.deepEqual(args, { url: 'fixture' });
      return {
        session_id: 's1',
        title: 'Example',
        url: 'https://example.test/',
        cache_restored: false,
        regions: [{ id: 'r_main', role: 'main', elements: [] }],
      };
    };

    const session = await browser.openPage('fixture');
    assert.equal(session.sessionId, 's1');
    assert.equal(session.som.title, 'Example');
    assert.equal(session.som.url, 'https://example.test/');
    assert.equal(session.som.regions.length, 1);
    assert.equal(session.som.regions[0]?.id, 'r_main');
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

  it('bounds oversized tool error diagnostics', async () => {
    const directory = mkdtempSync(join(tmpdir(), 'plasmate-node-sdk-'));
    const fixture = join(directory, 'mcp-fixture.js');
    writeFileSync(
      fixture,
      `#!/usr/bin/env node
import { createInterface } from 'node:readline';

const send = (value) => process.stdout.write(JSON.stringify(value) + '\\n');
const input = createInterface({ input: process.stdin });
const diagnostic = 'x'.repeat(5000);

input.on('line', (line) => {
  const request = JSON.parse(line);
  if (request.method === 'initialize') {
    send({ jsonrpc: '2.0', id: request.id, result: { protocolVersion: '2024-11-05' } });
  } else if (request.method === 'tools/call') {
    send({
      jsonrpc: '2.0',
      id: request.id,
      result: { isError: true, content: [{ type: 'text', text: diagnostic }] },
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
        return error instanceof Error && error.message === `${'x'.repeat(199)}…`;
      });
    } finally {
      browser.close();
      rmSync(directory, { recursive: true, force: true });
    }
  });
});

describe('package entry points', () => {
  it('loads the advertised ESM package entry', () => {
    const script = `
      const module = await import('plasmate');
      if (typeof module.Plasmate !== 'function' || typeof module.findByRole !== 'function') {
        throw new Error('missing ESM SDK exports');
      }
    `;

    assert.doesNotThrow(() => {
      execFileSync(process.execPath, ['--input-type=module', '-e', script], {
        cwd: resolve(__dirname, '..'),
        stdio: 'pipe',
      });
    });
  });
});
