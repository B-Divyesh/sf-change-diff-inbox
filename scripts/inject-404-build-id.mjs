import { readFile, writeFile } from 'node:fs/promises';

const build = process.env.BUILD_SHA?.trim() || 'dev';
const path = new URL('../frontend/dist/404.html', import.meta.url);
const html = await readFile(path, 'utf8');

if (!html.includes('__BUILD_SHA__')) {
  throw new Error('The 404 build placeholder is missing.');
}

await writeFile(path, html.replaceAll('__BUILD_SHA__', build));
