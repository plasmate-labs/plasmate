import puppeteer from 'puppeteer-core';
import { createServer } from 'node:http';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const fixture = readFileSync(join(__dirname, 'fixture.html'), 'utf8');

// Spin up a tiny local server so we don't depend on external networks in CI.
const srv = createServer((_, res) => {
  res.writeHead(200, { 'Content-Type': 'text/html' });
  res.end(fixture);
});
await new Promise((r) => srv.listen(0, '127.0.0.1', r));
const port = srv.address().port;
const testUrl = `http://127.0.0.1:${port}/`;

try {
  const browser = await puppeteer.connect({
    browserWSEndpoint: 'ws://127.0.0.1:9222/devtools/browser/plasmate',
    protocolTimeout: 10_000,
  });

  const page = await browser.newPage();
  console.log('1. newPage OK');

  await page.goto(testUrl, { waitUntil: 'load', timeout: 15_000 });
  console.log('2. goto OK');

  const title = await page.title();
  if (title !== 'Plasmate Smoke Test') {
    throw new Error(`Unexpected title: ${title}`);
  }
  console.log('3. title OK:', title);

  const content = await page.content();
  if (!content || content.length < 50) {
    throw new Error(`Unexpected content length: ${content?.length}`);
  }
  console.log('4. content OK, length:', content.length);

  // Verify Plasmate custom CDP domain
  const cdp = await page.createCDPSession();
  const { som } = await cdp.send('Plasmate.getSom');
  if (!som?.title) throw new Error('Missing som.title');
  if (!som?.meta?.element_count) throw new Error('Missing som.meta.element_count');
  console.log('5. SOM OK:', som.title, '|', som.meta.element_count, 'elements');

  const { elements, count } = await cdp.send('Plasmate.getInteractiveElements');
  console.log('6. Interactive OK:', count, 'elements');

  await browser.disconnect();
  console.log('PASS');
} finally {
  srv.close();
}
