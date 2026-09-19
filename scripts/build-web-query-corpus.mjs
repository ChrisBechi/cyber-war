import { mkdir, writeFile } from 'node:fs/promises';
import { buildPack } from './web-content.mjs';
// Mechanical retrieval judgments: the expected document contains the quoted passage.
// Kept separate from the small authored query set and from human relevance review.
const docs = buildPack().documents.filter(
  (d) => d.path !== '/' && !d.requiredFlags.length && d.publishedAt <= 1789257600,
);
const normalize = (text) =>
  text
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, ' ')
    .trim();
const queues = docs.map((d) => {
  const passages = [
    d.title,
    d.summary,
    ...d.blocks.filter((b) => b.kind === 'paragraph').map((b) => b.text),
  ];
  const queries = [];
  for (const passage of passages) {
    const words = normalize(passage).split(' ');
    for (let length = 3; length <= Math.min(9, words.length); length++)
      for (let i = 0; i + length <= words.length; i++)
        queries.push(words.slice(i, i + length).join(' '));
  }
  return { doc: d, queries: [...new Set(queries)] };
});
const seen = new Map();
for (let round = 0; seen.size < 10_000; round++) {
  let progress = false;
  for (const { doc, queries } of queues) {
    const query = queries[round];
    if (!query) continue;
    progress = true;
    if (!seen.has(query))
      seen.set(query, { query, expectedIds: [doc.id], origin: doc.id, judgment: 'source-passage' });
    else if (!seen.get(query).expectedIds.includes(doc.id))
      seen.get(query).expectedIds.push(doc.id);
    if (seen.size === 10_000) break;
  }
  if (!progress) break;
}
if (seen.size < 10_000) throw new Error(`Only ${seen.size} distinct passages available`);
await mkdir('artifacts/web-audit', { recursive: true });
await writeFile('artifacts/web-audit/query-corpus.json', JSON.stringify([...seen.values()]));
console.log(
  JSON.stringify({
    queries: seen.size,
    sourceDocuments: new Set([...seen.values()].map((q) => q.origin)).size,
    judgment: 'mechanical quoted-passage retrieval, not human reviewed',
    output: 'artifacts/web-audit/query-corpus.json',
  }),
);
