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
    'src-tauri/src/coreutils/base64.rs',
    'src-tauri/src/coreutils/tee.rs',
    'src-tauri/src/coreutils/tee_tests.rs',
    'src-tauri/src/coreutils/wc.rs',
    'src-tauri/src/coreutils/wc_tests.rs',
    'src-tauri/src/shell_pipeline/streams/wc.rs',
    'src-tauri/src/coreutils/base64_tests.rs',
    'src-tauri/src/coreutils/foundation.rs',
    'src-tauri/src/coreutils/cat.rs',
    'src-tauri/src/coreutils/head.rs',
    'src-tauri/src/coreutils/tail.rs',
  ]);
  const foundation = ['basename', 'dirname', 'printenv', 'whoami'].includes(test.command);
  return sources.filter((p) => {
    if (
      [
        'scripts/cli/wc-cases.mjs',
        'scripts/cli-wc-generate.mjs',
        'scripts/cli/wc.test.mjs',
      ].includes(p)
    )
      return test.command === 'wc';
    if (
      [
        'scripts/cli/tee-cases.mjs',
        'scripts/cli-tee-generate.mjs',
        'scripts/cli/tee.test.mjs',
      ].includes(p)
    )
      return test.command === 'tee';
    if (['scripts/cli/base64-cases.mjs', 'scripts/cli-base64-generate.mjs'].includes(p))
      return test.command === 'base64';
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
      return (
        (foundation || ['cat', 'head', 'tail', 'base64', 'tee', 'wc'].includes(test.command)) &&
        p.includes('/' + test.command + '-')
      );
    if (p.endsWith('/foundation_options.rs') || p.endsWith('/foundation_messages.rs'))
      return foundation || ['cat', 'head', 'tail', 'base64', 'tee', 'wc'].includes(test.command);
    return (
      !leaf.has(p) ||
      p === impl ||
      (['head', 'tail', 'base64', 'tee', 'wc'].includes(test.command) && p.endsWith('/cat.rs')) ||
      (test.command === 'wc' && (p.endsWith('/wc_tests.rs') || p.endsWith('/streams/wc.rs'))) ||
      (test.command === 'base64' && p.endsWith('/base64_tests.rs')) ||
      (test.command === 'tee' && p.endsWith('/tee_tests.rs')) ||
      (test.command === 'tail' && p.endsWith('/head.rs')) ||
      (['head', 'cat', 'basename', 'dirname', 'printenv', 'whoami'].includes(test.command) &&
        p.endsWith('/tail.rs'))
    );
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
