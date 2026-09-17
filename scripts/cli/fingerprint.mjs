import { createHash } from 'node:crypto';
import { evidenceSources, digest, json } from './io.mjs';

export function caseSources(test, sources = evidenceSources()) {
  // Integration scripts conservatively depend on all handlers. Direct Coreutils
  // cases omit unrelated leaf handlers; shared shell/VFS/parser changes invalidate all.
  // env invokes another executable, so its evidence depends on the child handler.
  if (test.softwareId !== 'coreutils' || test.script || test.command === 'env') return sources;
  const manifest = json('content/cli-compatibility/manifest.json');
  const impl = manifest.native[test.command]?.implementation;
  const leaf = new Set([
    'src-tauri/src/terminal_text.rs',
    'src-tauri/src/terminal_query.rs',
    'src-tauri/src/terminal_transfer.rs',
    'src-tauri/src/terminal_remove.rs',
    'src-tauri/src/coreutils/bytes.rs',
    'src-tauri/src/coreutils/foundation.rs',
    'src-tauri/src/coreutils/cat.rs',
  ]);
  const foundation = ['basename', 'dirname', 'printenv', 'whoami'].includes(test.command);
  return sources.filter((p) => {
    // Each request carries its own fixture/expectation. GNU freshness is checked
    // separately; a new reference or a sibling case cannot refresh execution.
    if (
      p.startsWith('tests/cli/compat/') ||
      p.startsWith('tests/cli/references/') ||
      p.startsWith('tests/cli/gnu/')
    )
      return false;
    if (p.startsWith('content/cli-compatibility/')) return false;
    if (p.startsWith('src-tauri/src/coreutils/messages/'))
      return (foundation || test.command === 'cat') && p.includes('/' + test.command + '-');
    if (p.endsWith('/foundation_options.rs') || p.endsWith('/foundation_messages.rs'))
      return foundation || test.command === 'cat';
    return !leaf.has(p) || p === impl;
  });
}
export function caseFingerprint(test, cache = new Map(), sources = evidenceSources()) {
  const paths = caseSources(test, sources);
  const key = paths.join('\0');
  if (!cache.has(key)) cache.set(key, digest(paths));
  const manifest = json('content/cli-compatibility/manifest.json');
  const contract =
    test.softwareId === 'coreutils'
      ? json('content/cli-compatibility/coreutils.json').commands[test.command]
      : null;
  return createHash('sha256')
    .update(cache.get(key))
    .update(
      JSON.stringify({
        test,
        native: manifest.native[test.command],
        software: manifest.software[test.softwareId],
        contract,
      }),
    )
    .digest('hex');
}
export function caseFingerprints(cases) {
  const cache = new Map(),
    sources = evidenceSources();
  return Object.fromEntries(cases.map((c) => [c.id, caseFingerprint(c, cache, sources)]));
}
