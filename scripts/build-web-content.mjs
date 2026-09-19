import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { buildPack } from './web-content.mjs';
import { validateWebPack } from './web-schema.mjs';

const pack = buildPack();
const report = validateWebPack(pack);
const output = `${JSON.stringify(pack)}\n`;
const path = new URL('../content/web/web-core.json', import.meta.url);
if (process.argv.includes('--check')) {
  if ((await readFile(path, 'utf8')) !== output)
    throw new Error('web-core is stale; run node scripts/build-web-content.mjs');
} else {
  await mkdir(new URL('../content/web/', import.meta.url), { recursive: true });
  await writeFile(path, output);
}
console.log(JSON.stringify(report, null, 2));
