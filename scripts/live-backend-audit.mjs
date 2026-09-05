import { request } from 'playwright';
import { mkdir, writeFile } from 'node:fs/promises';

const origin = process.argv[2] || 'https://change-diff-inbox.sociobot.in';
const source = {
  name: 'Isolation audit source',
  url: 'https://example.com/',
  selector: 'main',
  extract_mode: 'selector',
  threshold: 0.03,
  interval_minutes: 1440,
};

async function workspace(ip) {
  const context = await request.newContext({ baseURL: origin, extraHTTPHeaders: { 'x-forwarded-for': ip } });
  const response = await context.post('/api/session');
  if (!response.ok()) throw new Error(`session failed with ${response.status()}`);
  return context;
}

const first = await workspace('203.0.113.211');
const second = await workspace('203.0.113.212');
const rateClient = await workspace('203.0.113.213');
try {
  const createdResponse = await first.post('/api/sources', { data: source });
  if (createdResponse.status() !== 201) throw new Error(`create failed with ${createdResponse.status()}`);
  const created = await createdResponse.json();
  const secondSources = await (await second.get('/api/sources')).json();
  const crossDeleteStatus = (await second.delete(`/api/sources/${created.id}`)).status();
  const firstSources = await (await first.get('/api/sources')).json();
  const ownerDeleteStatus = (await first.delete(`/api/sources/${created.id}`)).status();
  const ownerAfterDelete = await (await first.get('/api/sources')).json();

  const responses = await Promise.all(Array.from({ length: 50 }, () => rateClient.get('/api/stats')));
  const limited = responses.find((response) => response.status() === 429);
  const result = {
    tenantIsolation: {
      secondSourceCount: secondSources.length,
      crossDeleteStatus,
      firstSourceNames: firstSources.map((item) => item.name),
      ownerDeleteStatus,
      ownerSourceCountAfterCleanup: ownerAfterDelete.length,
    },
    rateLimit: {
      statuses: [...new Set(responses.map((response) => response.status()))].sort(),
      limitedCount: responses.filter((response) => response.status() === 429).length,
      retryAfter: limited?.headers()['retry-after'] || null,
    },
  };
  await mkdir('.factory/evidence/live', { recursive: true });
  await writeFile('.factory/evidence/live/backend-boundaries.json', JSON.stringify(result, null, 2));
  console.log(JSON.stringify(result));

  if (secondSources.length !== 0
    || crossDeleteStatus !== 404
    || firstSources.length !== 1
    || ownerDeleteStatus !== 204
    || ownerAfterDelete.length !== 0
    || !limited
    || !/^\d+$/.test(limited.headers()['retry-after'] || '')) process.exitCode = 1;
} finally {
  await first.dispose();
  await second.dispose();
  await rateClient.dispose();
}
