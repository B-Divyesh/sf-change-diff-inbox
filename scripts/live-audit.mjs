import { chromium } from 'playwright';
import { mkdir, writeFile } from 'node:fs/promises';

const origin = process.argv[2] || 'https://change-diff-inbox.sociobot.in';
const evidenceDir = '.factory/evidence/live';
await mkdir(evidenceDir, { recursive: true });

const browser = await chromium.launch({ headless: true });
const context = await browser.newContext({
  viewport: { width: 1366, height: 900 },
  extraHTTPHeaders: { 'x-forwarded-for': '198.51.100.241' },
});
const page = await context.newPage();
page.setDefaultTimeout(10_000);
const errors = [];
const origins = new Set();
page.on('console', (message) => {
  if (message.type() === 'error') errors.push(message.text());
});
page.on('pageerror', (error) => errors.push(String(error)));
page.on('request', (request) => origins.add(new URL(request.url()).origin));

try {
  console.log('home');
  await page.goto(origin, { waitUntil: 'networkidle' });
  const firstScreen = await page.evaluate(() => ({
    h1: document.querySelector('h1')?.textContent?.trim(),
    audience: [...document.querySelectorAll('p')]
      .find((item) => item.textContent?.includes('For developers'))?.textContent?.trim(),
    action: [...document.querySelectorAll('a')]
      .find((item) => item.textContent?.includes('Try it with sample data'))?.textContent?.trim(),
    facts: [...document.querySelectorAll('.plain-facts li')].map((item) => item.textContent?.trim()),
  }));
  const realBefore = await page.evaluate(async () => (await fetch('/api/stats')).json());

  console.log('enter demo');
  await page.getByRole('link', { name: 'Try it with sample data' }).click();
  await page.waitForURL('**/demo');
  const label = await page
    .getByText('Demo — sample data, nothing is saved to your workspace')
    .textContent();
  const count = await page.locator('.change-card').count();
  await page.locator('.change-toggle').first().click();
  const sample = await page.locator('.change-card').first().innerText();

  console.log('change and reset demo');
  await page.getByRole('button', { name: 'Archive' }).first().click();
  await page.getByRole('button', { name: 'Reset demo' }).click();
  await page.getByText('Sample data reset.').waitFor();
  const resetCount = await page.locator('.change-card').count();
  await page.screenshot({ path: `${evidenceDir}/demo-desktop.png`, fullPage: true });

  console.log('leave demo');
  await page.getByRole('link', { name: 'Start for real' }).click();
  await page.waitForURL(`${origin}/`);
  const realAfter = await page.evaluate(async () => (await fetch('/api/stats')).json());

  console.log('phone demo');
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${origin}/demo`, { waitUntil: 'networkidle' });
  const mobileWidth = await page.evaluate(() => ({
    scroll: document.documentElement.scrollWidth,
    client: document.documentElement.clientWidth,
  }));
  await page.screenshot({ path: `${evidenceDir}/demo-mobile.png`, fullPage: true });

  const result = {
    firstScreen,
    label,
    count,
    resetCount,
    sample: sample.slice(0, 500),
    realBefore,
    realAfter,
    origins: [...origins],
    errors,
    mobileWidth,
  };
  await writeFile(`${evidenceDir}/demo-flow.json`, JSON.stringify(result, null, 2));
  console.log(JSON.stringify(result));

  const failed = errors.length
    || count !== 3
    || resetCount !== 3
    || JSON.stringify(realBefore) !== JSON.stringify(realAfter)
    || [...origins].some((item) => item !== origin)
    || mobileWidth.scroll > mobileWidth.client;
  if (failed) process.exitCode = 1;
} finally {
  await browser.close();
}
