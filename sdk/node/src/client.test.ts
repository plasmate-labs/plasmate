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
    await browser.extractText('fixture', { javascript: false, selector: 'content' });
    await browser.extractLinks('fixture', { javascript: false, maxLinks: 3, selector: 'main' });

    assert.deepEqual(calls, [
      { name: 'fetch_page', args: { url: 'fixture', selector: 'main' } },
      { name: 'fetch_page', args: { url: 'fixture', selector: 'interactive' } },
      { name: 'extract_text', args: { url: 'fixture', javascript: false, selector: 'content' } },
      { name: 'extract_links', args: { url: 'fixture', javascript: false, max_links: 3, selector: 'main' } },
    ]);
    browser.close();
  });
});

describe('extractText payload', () => {
  it('keeps JSON-looking and empty extract_text as strings', async () => {
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
    const name = request.params.name;
    const text = name === 'extract_text'
      ? (request.params.arguments.url === 'empty' ? '' : '{"headline":"ok"}')
      : JSON.stringify({ title: 'Example' });
    send({
      jsonrpc: '2.0',
      id: request.id,
      result: { content: [{ type: 'text', text }] },
    });
  }
});
`,
      'utf8',
    );
    chmodSync(fixture, 0o755);

    const browser = new Plasmate({ binary: fixture });
    try {
      assert.equal(await browser.extractText('json-page'), '{"headline":"ok"}');
      assert.equal(await browser.extractText('empty'), '');
      assert.deepEqual(await browser.fetchPage('fixture'), { title: 'Example' });
    } finally {
      browser.close();
      rmSync(directory, { recursive: true, force: true });
    }
  });

  it('keeps plain extract_links output as a string', async () => {
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
      result: { content: [{ type: 'text', text: 'https://example.test/a\\nhttps://example.test/b' }] },
    });
  }
});
`,
      'utf8',
    );
    chmodSync(fixture, 0o755);

    const browser = new Plasmate({ binary: fixture });
    try {
      assert.equal(
        await browser.extractLinks('fixture'),
        'https://example.test/a\nhttps://example.test/b',
      );
    } finally {
      browser.close();
      rmSync(directory, { recursive: true, force: true });
    }
  });

  it('uses structuredContent when a tool omits text content', async () => {
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
    const result = request.params.name === 'extract_text'
      ? 'structured text'
      : { title: 'Structured Example' };
    send({
      jsonrpc: '2.0',
      id: request.id,
      result: { content: [], structuredContent: { result } },
    });
  }
});
`,
      'utf8',
    );
    chmodSync(fixture, 0o755);

    const browser = new Plasmate({ binary: fixture });
    try {
      assert.equal(await browser.extractText('fixture'), 'structured text');
      assert.deepEqual(await browser.fetchPage('fixture'), { title: 'Structured Example' });
    } finally {
      browser.close();
      rmSync(directory, { recursive: true, force: true });
    }
  });

  it('combines text content blocks in order', async () => {
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
      result: {
        content: [
          { type: 'image', data: 'ignored' },
          { type: 'text', text: 'progress' },
          { type: 'text', text: JSON.stringify({ ok: true }) },
        ],
      },
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

  it('fails closed when a structured tool returns non-JSON text', async () => {
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
      result: { content: [{ type: 'text', text: 'not-json' }] },
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
        return error instanceof Error && error.message === 'Tool returned unparseable output';
      });
    } finally {
      browser.close();
      rmSync(directory, { recursive: true, force: true });
    }
  });

  it('uses the first text block when MCP content starts with another type', async () => {
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
      result: {
        content: [
          { type: 'image', data: 'ignored' },
          { type: 'text', text: JSON.stringify({ title: 'Example' }) },
        ],
      },
    });
  }
});
`,
      'utf8',
    );
    chmodSync(fixture, 0o755);

    const browser = new Plasmate({ binary: fixture });
    try {
      assert.deepEqual(await browser.fetchPage('fixture'), { title: 'Example' });
    } finally {
      browser.close();
      rmSync(directory, { recursive: true, force: true });
    }
  });
});

describe('evaluate payload', () => {
  it('returns the JS value instead of the MCP result envelope', async () => {
    const browser = new Plasmate({ binary: 'unused' });
    const client = browser as unknown as {
      callTool: (name: string, args: Record<string, unknown>) => Promise<unknown>;
    };
    client.callTool = async (name, args) => {
      assert.equal(name, 'evaluate');
      assert.deepEqual(args, { session_id: 's1', expression: 'document.title' });
      return { result: 'Example' };
    };

    assert.equal(await browser.evaluate('s1', 'document.title'), 'Example');
    browser.close();
  });

  it('keeps non-envelope payloads unchanged', async () => {
    const browser = new Plasmate({ binary: 'unused' });
    const client = browser as unknown as {
      callTool: (name: string, args: Record<string, unknown>) => Promise<unknown>;
    };
    client.callTool = async () => ({ result: 'Example', extra: true });

    assert.deepEqual(await browser.evaluate('s1', 'document.title'), {
      result: 'Example',
      extra: true,
    });
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

  it('forwards an initial SOM selector', async () => {
    const browser = new Plasmate({ binary: 'unused' });
    const client = browser as unknown as {
      callTool: (name: string, args: Record<string, unknown>) => Promise<unknown>;
    };
    client.callTool = async (name, args) => {
      assert.equal(name, 'open_page');
      assert.deepEqual(args, { url: 'fixture', selector: 'main' });
      return { session_id: 's1', regions: [] };
    };

    await browser.openPage('fixture', { selector: 'main' });
    browser.close();
  });

  it('forwards opt-in action tracing', async () => {
    const browser = new Plasmate({ binary: 'unused' });
    const client = browser as unknown as {
      callTool: (name: string, args: Record<string, unknown>) => Promise<unknown>;
    };
    client.callTool = async (name, args) => {
      assert.equal(name, 'open_page');
      assert.deepEqual(args, { url: 'fixture', trace: true });
      return { session_id: 's1', regions: [] };
    };

    await browser.openPage('fixture', { trace: true });
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

  it('uses structured diagnostics when an error has no text block', async () => {
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
      result: { isError: true, content: [], structuredContent: { error: { message: 'structured diagnostic' } } },
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
        return error instanceof Error && error.message === 'structured diagnostic';
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

  it('reads error diagnostics from the first text block', async () => {
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
      result: {
        isError: true,
        content: [
          { type: 'image', data: 'ignored' },
          { type: 'text', text: 'useful diagnostic' },
        ],
      },
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
        return error instanceof Error && error.message === 'useful diagnostic';
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
