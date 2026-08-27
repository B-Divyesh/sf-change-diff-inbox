import { chromium } from 'playwright';
import AxeBuilder from '@axe-core/playwright';
import { writeFile } from 'node:fs/promises';

const url = process.argv[2] || 'http://127.0.0.1:8080';
const browser = await chromium.launch({ headless: true });
const results = {};
for (const [name, viewport] of Object.entries({ desktop: { width: 1366, height: 900 }, mobile: { width: 390, height: 844 } })) {
  const context = await browser.newContext({ viewport });
  const page = await context.newPage();
  await page.goto(url, { waitUntil: 'networkidle' });
  results[name] = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa', 'wcag21aa']).analyze();
  await context.close();
}
await browser.close();
const summary = Object.fromEntries(Object.entries(results).map(([name, result]) => [name, result.violations.map(v => ({ id: v.id, impact: v.impact, nodes: v.nodes.length, help: v.help }))]));
await writeFile('.factory/evidence/axe.json', JSON.stringify(summary, null, 2));
console.log(JSON.stringify(summary));
if (Object.values(results).some(result => result.violations.some(v => ['serious', 'critical'].includes(v.impact)))) process.exit(1);
