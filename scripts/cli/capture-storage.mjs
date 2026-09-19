// DEV-only lossless case shards. Keep a small index and at most eight parsed
// results resident; neither JSON strings nor heap use grow with the full VFS matrix.
import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { json, root, write } from './io.mjs';

const references = new WeakMap();
const hash = (data) => createHash('sha256').update(data).digest('hex');
const directoryFor = (path) => resolve(root, path.replace(/\.json$/, '.cases'));
export function readCapture(path = 'artifacts/cli-case-actual.json') {
  const index = json(path);
  if (index.schemaVersion === 2) return index;
  if (index.schemaVersion !== 3 || !Array.isArray(index.cases))
    throw Error('Invalid capture index');
  const directory = directoryFor(path),
    cache = new Map();
  const cases = index.cases.map((entry) => {
    if (typeof entry.id !== 'string' || !/^[a-f0-9]{64}$/.test(entry.sha256))
      throw Error('Invalid case shard');
    const load = () => {
      if (!cache.has(entry.sha256)) {
        const bytes = readFileSync(resolve(directory, entry.sha256 + '.json'));
        if (hash(bytes) !== entry.sha256)
          throw Error('Case shard fingerprint mismatch: ' + entry.id);
        const value = JSON.parse(bytes.toString('utf8'));
        if (value.id !== entry.id) throw Error('Case shard identity mismatch');
        cache.set(entry.sha256, value);
        if (cache.size > 8) cache.delete(cache.keys().next().value);
      }
      return cache.get(entry.sha256);
    };
    const result = new Proxy(
      {},
      {
        get: (_, key) => (key === 'id' ? entry.id : load()[key]),
        has: (_, key) => key in load(),
        ownKeys: () => Reflect.ownKeys(load()),
        getOwnPropertyDescriptor: (_, key) => Object.getOwnPropertyDescriptor(load(), key),
      },
    );
    references.set(result, { directory, entry });
    return result;
  });
  return { schemaVersion: 2, cases };
}

export function writeCapture(path, capture) {
  const directory = directoryFor(path);
  mkdirSync(directory, { recursive: true });
  const cases = capture.cases.map((result) => {
    const reference = references.get(result);
    if (
      reference?.directory === directory &&
      existsSync(resolve(directory, reference.entry.sha256 + '.json'))
    )
      return reference.entry;
    const bytes = Buffer.from(JSON.stringify(result));
    const sha256 = hash(bytes);
    writeFileSync(resolve(directory, sha256 + '.json'), bytes);
    return { id: result.id, sha256 };
  });
  write(path, { schemaVersion: 3, cases });
}
