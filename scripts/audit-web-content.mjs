import { mkdir, writeFile } from 'node:fs/promises';
import { buildPack } from './web-content.mjs';
import { validateWebPack } from './web-schema.mjs';
const pack = buildPack();
const validation = validateWebPack(pack);
const grouped = (items, key) => {
  const groups = new Map();
  for (const item of items) {
    const value = key(item);
    if (!value) continue;
    groups.set(value, [...(groups.get(value) ?? []), item.id]);
  }
  return [...groups]
    .filter(([, ids]) => ids.length > 1)
    .map(([text, ids]) => ({ text, ids, count: ids.length }))
    .sort((a, b) => b.count - a.count);
};
const authored = pack.documents.filter((d) => d.path !== '/' && !d.materializer);
const report = {
  validation,
  editorial: {
    authored: authored.length,
    materialized: pack.documents.filter((d) => d.materializer).length,
    byPlatform: Object.fromEntries(
      pack.brands.map((b) => [
        b.name,
        { platform: b.platform, documents: authored.filter((d) => d.brandId === b.id).length },
      ]),
    ),
  },
  duplicateBodies: grouped(authored, (d) => JSON.stringify(d.blocks)),
  duplicateComments: grouped(
    authored.flatMap((d) => d.comments.map((c) => ({ ...c, id: `${d.id}:${c.id}` }))),
    (c) => c.text,
  ),
  intentionalRemovals: pack.events.flatMap((e) =>
    e.remove.map((documentId) => ({ event: e.id, documentId, reason: 'REMOVED', archive: true })),
  ),
  deferred: pack.documents.filter((d) => d.materializer).map((d) => d.id),
  missingEntities: pack.documents
    .filter((d) => d.path !== '/' && !d.entities.length)
    .map((d) => d.id),
  reviewRequired: [
    'A revisão humana de relevância e diversidade não é substituída por este relatório.',
    'Contagens de variantes não representam textos editoriais independentes.',
  ],
};
await mkdir('artifacts/web-audit', { recursive: true });
await writeFile('artifacts/web-audit/content.json', JSON.stringify(report, null, 2));
console.log(
  JSON.stringify(
    {
      documents: validation.documents,
      authored: authored.length,
      duplicateBodies: report.duplicateBodies.length,
      duplicateComments: report.duplicateComments.length,
      output: 'artifacts/web-audit/content.json',
    },
    null,
    2,
  ),
);
