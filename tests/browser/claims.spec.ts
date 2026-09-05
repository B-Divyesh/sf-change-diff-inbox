import { expect, request, test } from 'playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { spawn, type ChildProcess } from 'node:child_process';
import { createHmac } from 'node:crypto';

const baseURL = 'http://127.0.0.1:4173';
const licenseKey = 'sb_license:change-diff-inbox';
const verdictKey = `${licenseKey}:verdict`;
let nextIp = 20;

function ip() {
  nextIp += 1;
  return `198.51.100.${nextIp}`;
}

async function apiSession(demo = false) {
  const api = await request.newContext({ baseURL, extraHTTPHeaders: {'x-forwarded-for': ip()} });
  const response = await api.post(demo ? '/api/demo/session' : '/api/session');
  expect(response.ok()).toBeTruthy();
  return api;
}

const source = (name:string, interval = 1440) => ({
  name,
  url: 'https://example.com/',
  selector: 'main',
  extract_mode: 'selector',
  threshold: 0.03,
  interval_minutes: interval,
});

test.beforeEach(async ({page}) => {
  await page.setExtraHTTPHeaders({'x-forwarded-for': ip()});
});

test('@claim:demo-sandbox opens populated sample data, resets it, and keeps the label visible', async ({page}) => {
  await page.goto('/demo');
  await expect(page.getByRole('heading', {name:'Review sample page changes'})).toBeVisible();
  await expect(page.getByText('Demo — sample data, nothing is saved to your workspace')).toBeVisible();
  await expect(page.locator('.change-card')).toHaveCount(3);
  await page.locator('.change-toggle').first().click();
  await expect(page.getByRole('heading', {name:'Previous'})).toBeVisible();
  await page.getByRole('button', {name:'Archive'}).first().click();
  await page.getByRole('button', {name:'Reset demo'}).click();
  await expect(page.locator('.change-card')).toHaveCount(3);
  await expect(page.getByText('Sample data reset.')).toBeVisible();
});

test('@claim:tenant-isolation separates two browser workspaces at every record boundary', async () => {
  const first = await apiSession();
  const second = await apiSession();
  const created = await first.post('/api/sources', {data: source('Only in first workspace')});
  expect(created.status()).toBe(201);
  const record = await created.json();
  expect(await (await second.get('/api/sources')).json()).toEqual([]);
  expect((await second.delete(`/api/sources/${record.id}`)).status()).toBe(404);
  expect((await (await first.get('/api/sources')).json()).map((item:any) => item.name)).toEqual(['Only in first workspace']);
  await first.dispose(); await second.dispose();
});

test('@claim:structured-extraction extracts selected tables, code blocks, and JSON-LD into text', async () => {
  const api = await apiSession(true);
  const cases = [
    {mode:'table', selector:'#plans', html:'<table id="plans"><tr><td>Pro</td><td>$12</td></tr></table><p>ignore</p>', expected:'Pro $12'},
    {mode:'code', selector:'pre', html:'<pre>npm install package</pre>', expected:'npm install package'},
    {mode:'jsonld', selector:'', html:'<script type="application/ld+json">{"status":"ready"}</script>', expected:'{"status":"ready"}'},
  ];
  for (const item of cases) {
    const response = await api.post('/api/demo/sample/extract', {data:{...item, previous:''}});
    expect(response.ok()).toBeTruthy();
    expect((await response.json()).extracted).toBe(item.expected);
  }
  await api.dispose();
});

test('@claim:semantic-threshold creates word changes above the threshold and ignores smaller changes', async () => {
  const api = await apiSession(true);
  const data = {html:'<main>price is 12 now</main>', selector:'main', mode:'selector', previous:'price is 10'};
  const noise = await (await api.post('/api/demo/sample/extract', {data:{...data, threshold:1}})).json();
  const changed = await (await api.post('/api/demo/sample/extract', {data:{...data, threshold:0}})).json();
  expect(noise.outcome).toBe('noise');
  expect(changed.outcome).toBe('changed');
  expect(changed.summary).toContain('12');
  await api.dispose();
});

test('@claim:schedules records daily and weekly schedules and provides manual checks', async ({page}) => {
  await page.goto('/');
  expect((await page.request.post('/api/sources', {data:source('Daily source', 1440)})).status()).toBe(201);
  expect((await page.request.post('/api/sources', {data:source('Weekly source', 10080)})).status()).toBe(201);
  const records = await (await page.request.get('/api/sources')).json();
  expect(records.map((item:any) => item.interval_minutes).sort((a:number,b:number)=>a-b)).toEqual([1440,10080]);
  await page.reload();
  await page.getByRole('button', {name:'Sources', exact:true}).click();
  await expect(page.getByRole('button', {name:'Check now'}).first()).toBeVisible();
});

test('@claim:safety-boundaries rejects authenticated and private targets and enforces robots rules', async () => {
  const api = await apiSession();
  const auth = await api.post('/api/sources', {data:{...source('Authenticated'), url:'https://user:pass@example.com/'}});
  expect(auth.status()).toBe(400);
  const local = await api.post('/api/sources', {data:{...source('Local target'), url:'http://127.0.0.1/private'}});
  const localRecord = await local.json();
  const checked = await (await api.post(`/api/sources/${localRecord.id}/check`)).json();
  expect(checked.outcome).toBe('error');
  expect(checked.message).toContain('Private or local');
  await api.dispose();
  const demo = await apiSession(true);
  const robots = await demo.post('/api/demo/sample/extract', {data:{html:'<main>secret</main>',selector:'main',mode:'selector',robots:'User-agent: *\nDisallow: /private',path:'/private'}});
  expect(robots.status()).toBe(403);
  await demo.dispose();
});

test('@claim:size-limits rejects source bodies above 2 MB and extracted text above 250 KB', async () => {
  const api = await apiSession(true);
  const tooLarge = await api.post('/api/demo/sample/extract', {data:{html:'x'.repeat(2_000_001),selector:'main',mode:'selector'}});
  expect(tooLarge.status()).toBe(413);
  const extracted = await api.post('/api/demo/sample/extract', {data:{html:`<main>${'x'.repeat(250_001)}</main>`,selector:'main',mode:'selector'}});
  expect(extracted.status()).toBe(413);
  await api.dispose();
});

test('@claim:review-states saves unread, reviewed, archived, useful, and noise decisions', async () => {
  const api = await apiSession(true);
  const changes = await (await api.get('/api/demo/changes')).json();
  expect(new Set(changes.map((item:any) => item.review_state))).toEqual(new Set(['unread','reviewed','archived']));
  const target = changes.find((item:any) => item.review_state === 'unread');
  expect((await api.patch(`/api/demo/changes/${target.id}`, {data:{review_state:'reviewed',useful:false}})).ok()).toBeTruthy();
  const reviewed = await (await api.get('/api/demo/changes?state=reviewed')).json();
  expect(reviewed.some((item:any) => item.id === target.id && item.useful === 0)).toBeTruthy();
  await api.dispose();
});

test('@claim:csv-export downloads every displayed sample change as CSV', async ({page}) => {
  await page.goto('/demo');
  const downloadPromise = page.waitForEvent('download');
  await page.getByRole('button', {name:'Export CSV'}).click();
  const download = await downloadPromise;
  const stream = await download.createReadStream();
  let csv = '';
  for await (const chunk of stream!) csv += chunk.toString();
  expect(download.suggestedFilename()).toBe('change-diff-inbox.csv');
  expect(csv.split('\n')).toHaveLength(4);
  expect(csv).toContain('Northstar API limits');
});

test('@claim:responsive-keyboard keeps mobile content contained, keyboard focus visible, and serious accessibility errors absent', async ({page}) => {
  await page.setViewportSize({width:390,height:844});
  await page.goto('/demo');
  const dimensions = await page.evaluate(() => ({scroll:document.documentElement.scrollWidth,client:document.documentElement.clientWidth}));
  expect(dimensions.scroll).toBeLessThanOrEqual(dimensions.client);
  await page.keyboard.press('Tab');
  await expect(page.getByRole('link', {name:'Skip to main content'})).toBeFocused();
  const focus = await page.getByRole('link', {name:'Skip to main content'}).evaluate(element => getComputedStyle(element).outlineStyle);
  expect(focus).not.toBe('none');
  const results = await new AxeBuilder({page}).withTags(['wcag2a','wcag2aa','wcag21aa']).analyze();
  expect(results.violations.filter(item => ['serious','critical'].includes(item.impact || ''))).toEqual([]);
});

test('@claim:offline-shell reloads the product shell after the first visit without reusing the shared browser', async ({browser}) => {
  const context = await browser.newContext({extraHTTPHeaders:{'x-forwarded-for':ip()}});
  const page = await context.newPage();
  await page.goto(`${baseURL}/demo`);
  await page.waitForFunction(() => navigator.serviceWorker?.controller !== null, undefined, {timeout:10_000}).catch(async () => {
    await page.reload();
    await page.waitForFunction(() => navigator.serviceWorker?.controller !== null);
  });
  await context.setOffline(true);
  await page.reload({waitUntil:'domcontentloaded'});
  await expect(page.getByText('Demo — sample data, nothing is saved to your workspace')).toBeVisible();
  await expect(page.getByText(/Offline — the app shell is available/)).toBeVisible();
  await context.close();
});

test('@claim:privacy-network keeps the demo flow same-origin with no analytics, remote fonts, or tracking calls', async ({browser}) => {
  const context = await browser.newContext({extraHTTPHeaders:{'x-forwarded-for':ip()}});
  const page = await context.newPage();
  const origins = new Set<string>();
  page.on('request', requestItem => origins.add(new URL(requestItem.url()).origin));
  await page.goto(`${baseURL}/demo`);
  await page.locator('.change-toggle').first().click();
  await page.getByRole('button', {name:'Reset demo'}).click();
  await expect(page.getByText('Sample data reset.')).toBeVisible();
  expect([...origins]).toEqual([baseURL]);
  await context.close();
});

test('@claim:free-limits enforces five sources and daily or weekly checks at the server', async () => {
  const api = await apiSession();
  expect((await api.post('/api/sources', {data:source('Too frequent',15)})).status()).toBe(402);
  for (let count=1; count<=5; count++) expect((await api.post('/api/sources', {data:source(`Allowed ${count}`)})).status()).toBe(201);
  expect((await api.post('/api/sources', {data:source('Sixth source')})).status()).toBe(402);
  await api.dispose();
});

test('@claim:paid-entitlement accepts short schedules and more than five sources only with a signed Pro session', async () => {
  const bootstrap = await request.newContext({baseURL, extraHTTPHeaders:{'x-forwarded-for':ip()}});
  const sessionResponse = await bootstrap.post('/api/session');
  const workspaceCookie = sessionResponse.headers()['set-cookie'].split(';')[0];
  const signedWorkspace = workspaceCookie.split('=')[1];
  const encodedPayload = signedWorkspace.split('.')[0];
  const tenantId = Buffer.from(encodedPayload, 'base64url').toString().split(':')[1];
  const payload = Buffer.from(`pro:${tenantId}:${Math.floor(Date.now()/1000)+3600}`).toString('base64url');
  const signature = createHmac('sha256', 'claim-suite-session-secret-32-bytes-minimum').update(payload).digest('base64url');
  const api = await request.newContext({baseURL, extraHTTPHeaders:{'x-forwarded-for':ip(),'cookie':`${workspaceCookie}; cdi_pro=${payload}.${signature}`}});
  expect((await api.post('/api/sources', {data:source('Fast Pro source',15)})).status()).toBe(201);
  for (let count=2; count<=6; count++) expect((await api.post('/api/sources', {data:source(`Pro source ${count}`)})).status()).toBe(201);
  expect((await (await api.get('/api/sources')).json()).length).toBe(6);
  await bootstrap.dispose(); await api.dispose();
});

test('@claim:rate-limit returns 429 and Retry-After beyond the live request allowance', async () => {
  const clientIp = ip();
  const api = await request.newContext({baseURL, extraHTTPHeaders:{'x-forwarded-for':`${clientIp}, 10.1.2.3`}});
  expect((await api.post('/api/session')).ok()).toBeTruthy();
  const responses = await Promise.all(Array.from({length:50}, () => api.get('/api/stats')));
  const limited = responses.find(response => response.status() === 429);
  expect(limited).toBeTruthy();
  expect(limited!.headers()['retry-after']).toMatch(/^\d+$/);
  await api.dispose();
});

test('@claim:delete-cascade removes a source and all of its saved changes', async () => {
  const api = await apiSession(true);
  const changes = await (await api.get('/api/demo/changes')).json();
  const target = changes[0];
  expect((await api.delete(`/api/demo/sources/${target.source_id}`)).status()).toBe(204);
  const after = await (await api.get('/api/demo/changes')).json();
  expect(after.some((item:any) => item.source_id === target.source_id)).toBeFalsy();
  expect(after).toHaveLength(2);
  await api.dispose();
});

test('@claim:license-cache uses a daily cached verdict without blocking the free inbox', async ({page}) => {
  const requests:string[] = [];
  page.on('request', requestItem => requests.push(requestItem.url()));
  await page.addInitScript(({licenseKey, verdictKey}) => {
    localStorage.setItem(licenseKey, 'stored-license');
    localStorage.setItem(verdictKey, JSON.stringify({valid:true,reason:'ok',checkedAt:Date.now()}));
  }, {licenseKey, verdictKey});
  await page.goto('/');
  await expect(page.getByRole('heading', {name:'Review meaningful page changes'})).toBeVisible();
  expect(requests.some(url => url.includes('/api/license'))).toBeFalsy();
});

test('@claim:route-contract gives demo, legal, back-navigation, and 404 pages correct titles and headings', async ({page}) => {
  await page.goto('/privacy');
  await expect(page).toHaveTitle('Privacy — Change Diff Inbox');
  await page.getByRole('link', {name:'Terms', exact:true}).first().click();
  await expect(page).toHaveTitle('Terms — Change Diff Inbox');
  await page.goBack();
  await expect(page.getByRole('heading', {name:'Privacy for your monitored pages'})).toBeFocused();
  const missing = await page.goto('/missing-page');
  expect(missing?.status()).toBe(404);
  await expect(page).toHaveTitle('Page not found — Change Diff Inbox');
  await expect(page.locator('h1')).toHaveText('This page does not exist');
});

test('@claim:restart-persistence retains one workspace and its source after a clean server restart', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'change-diff-claim-'));
  const port = 4174;
  const binary = resolve('target/debug/change-diff-inbox');
  const env = {...process.env, PORT:String(port), DATA_DIR:dataDir, FRONTEND_DIR:resolve('frontend/dist')};
  let processHandle:ChildProcess | undefined;
  const start = async () => {
    processHandle = spawn(binary, [], {env, stdio:'ignore'});
    for (let attempt=0; attempt<80; attempt++) {
      try { if ((await fetch(`http://127.0.0.1:${port}/health`)).ok) return; } catch {}
      await new Promise(resolveWait => setTimeout(resolveWait, 100));
    }
    throw new Error('restart test server did not start');
  };
  const stop = async () => {
    if (!processHandle) return;
    processHandle.kill('SIGTERM');
    await new Promise<void>(resolveClose => processHandle!.once('exit', () => resolveClose()));
  };
  try {
    await start();
    const api = await request.newContext({baseURL:`http://127.0.0.1:${port}`,extraHTTPHeaders:{'x-forwarded-for':ip()}});
    expect((await api.post('/api/session')).ok()).toBeTruthy();
    expect((await api.post('/api/sources', {data:source('Persists after restart')})).status()).toBe(201);
    await stop();
    await start();
    const saved = await (await api.get('/api/sources')).json();
    expect(saved.map((item:any) => item.name)).toEqual(['Persists after restart']);
    await api.dispose();
    await stop();
  } finally {
    if (processHandle && processHandle.exitCode === null) processHandle.kill('SIGTERM');
    await rm(dataDir, {recursive:true,force:true});
  }
});
