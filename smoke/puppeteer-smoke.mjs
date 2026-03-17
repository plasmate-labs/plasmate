import puppeteer from 'puppeteer-core';

async function main() {
  const browser = await puppeteer.connect({
    browserWSEndpoint: 'ws://127.0.0.1:9222/devtools/browser/plasmate',
    protocolTimeout: 10_000,
  });

  const page = await browser.newPage();

  await page.goto('https://example.com', { waitUntil: 'load', timeout: 15_000 });

  const title = await page.title();
  if (!title || !title.toLowerCase().includes('example')) {
    throw new Error(`Unexpected title: ${title}`);
  }

  const content = await page.content();
  if (!content || content.length < 100) {
    throw new Error(`Unexpected content length: ${content?.length}`);
  }

  // Verify Plasmate custom domain works through same CDP connection
  const cdp = await page.createCDPSession();
  const { som } = await cdp.send('Plasmate.getSom');
  if (!som?.title) throw new Error('Missing som.title');
  if (!som?.meta?.element_count) throw new Error('Missing som.meta.element_count');

  await browser.disconnect();
}

main().catch((e) => {
  console.error(e?.stack || e?.message || String(e));
  process.exit(1);
});
