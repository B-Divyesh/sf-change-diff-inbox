import { chromium } from 'playwright';
import { mkdir, writeFile } from 'node:fs/promises';

const origin = process.argv[2] || 'https://change-diff-inbox.sociobot.in';
const browser = await chromium.launch({ headless: true });
const result = {};

try {
  const context = await browser.newContext({
    viewport: { width: 390, height: 844 },
    reducedMotion: 'reduce',
    extraHTTPHeaders: { 'x-forwarded-for': '203.0.113.214' },
  });
  const page = await context.newPage();
  const errors = [];
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text());
  });
  page.on('pageerror', (error) => errors.push(String(error)));

  await page.goto(`${origin}/demo`, { waitUntil: 'networkidle' });
  await page.keyboard.press('Tab');
  const focusedText = await page.locator(':focus').textContent();
  const focusOutline = await page.locator(':focus').evaluate((element) => getComputedStyle(element).outlineStyle);
  await page.evaluate(() => { document.documentElement.style.fontSize = '200%'; });
  await page.waitForTimeout(100);
  const resized = await page.evaluate(() => ({
    scroll: document.documentElement.scrollWidth,
    client: document.documentElement.clientWidth,
    reducedMotion: matchMedia('(prefers-reduced-motion: reduce)').matches,
    runningAnimations: document.getAnimations().filter((item) => item.playState === 'running').length,
  }));

  await page.goto(`${origin}/privacy`);
  const privacy = { title: await page.title(), h1: await page.locator('h1').textContent() };
  await page.getByRole('link', { name: 'Terms', exact: true }).first().click();
  const terms = { title: await page.title(), h1: await page.locator('h1').textContent() };
  await page.goBack();
  const backFocus = await page.locator('h1').evaluate((element) => element === document.activeElement);
  const errorsBeforeExpected404 = [...errors];
  const missingResponse = await page.goto(`${origin}/missing-live-contract`);
  const missing = {
    status: missingResponse?.status(),
    title: await page.title(),
    h1: await page.locator('h1').textContent(),
  };
  result.browser = {
    focusedText,
    focusOutline,
    resized,
    privacy,
    terms,
    backFocus,
    missing,
    errors: errorsBeforeExpected404,
    expected404ConsoleMessages: errors.slice(errorsBeforeExpected404.length),
  };
  await context.close();

  const offlineContext = await browser.newContext({ extraHTTPHeaders: { 'x-forwarded-for': '203.0.113.215' } });
  const offlinePage = await offlineContext.newPage();
  await offlinePage.goto(`${origin}/demo`);
  await offlinePage.waitForFunction(() => navigator.serviceWorker?.controller !== null, undefined, { timeout: 10_000 }).catch(async () => {
    await offlinePage.reload();
    await offlinePage.waitForFunction(() => navigator.serviceWorker?.controller !== null);
  });
  await offlineContext.setOffline(true);
  await offlinePage.reload({ waitUntil: 'domcontentloaded' });
  result.offline = {
    labelVisible: await offlinePage.getByText('Demo — sample data, nothing is saved to your workspace').isVisible(),
    noticeVisible: await offlinePage.getByText(/Offline — the app shell is available/).isVisible(),
    title: await offlinePage.title(),
  };
  await offlineContext.close();

  await mkdir('.factory/evidence/live', { recursive: true });
  await writeFile('.factory/evidence/live/browser-contract.json', JSON.stringify(result, null, 2));
  console.log(JSON.stringify(result));

  const failed = result.browser.focusedText?.trim() !== 'Skip to main content'
    || result.browser.focusOutline === 'none'
    || result.browser.resized.scroll > result.browser.resized.client
    || !result.browser.resized.reducedMotion
    || result.browser.resized.runningAnimations !== 0
    || result.browser.privacy.title !== 'Privacy — Change Diff Inbox'
    || result.browser.terms.title !== 'Terms — Change Diff Inbox'
    || !result.browser.backFocus
    || result.browser.missing.status !== 404
    || result.browser.missing.title !== 'Page not found — Change Diff Inbox'
    || result.browser.errors.length
    || !result.offline.labelVisible
    || !result.offline.noticeVisible;
  if (failed) process.exitCode = 1;
} finally {
  await browser.close();
}
