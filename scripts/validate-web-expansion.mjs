import { mkdir, writeFile } from 'node:fs/promises';
import { buildPack } from './web-content.mjs';
import { validateWebPack } from './web-schema.mjs';
const rows = [];
for (const count of [50, 100, 150, 210]) {
  const pack = buildPack({ longTailLimit: count - 18 });
  const validation = validateWebPack(pack);
  const small = pack.brands.filter((b) => b.depth === 'LONG_TAIL');
  const identities = new Set(small.map((b) => JSON.stringify(b.identity)));
  if (identities.size !== small.length)
    throw new Error(`Repeated structural identity in ${count} brands`);
  for (const brand of small) {
    const pages = pack.documents.filter((d) => d.brandId === brand.id && d.path !== '/');
    if (pages.length < 2 || new Set(pages.map((d) => d.summary)).size !== pages.length)
      throw new Error(`Incomplete publisher ${brand.id}`);
  }
  rows.push({
    brands: validation.brands,
    documents: validation.documents,
    links: validation.checkedLinks,
    uniqueLongTailIdentities: identities.size,
    bytes: validation.bytes,
  });
}
await mkdir('artifacts/web-audit', { recursive: true });
await writeFile(
  'artifacts/web-audit/expansion.json',
  JSON.stringify(
    {
      runs: rows,
      limitation:
        'Structural and content integrity audit; does not replace visual or human editorial review.',
    },
    null,
    2,
  ),
);
console.log(JSON.stringify(rows, null, 2));
