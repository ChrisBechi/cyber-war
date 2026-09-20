import { createHash } from 'node:crypto';
import { evidenceSources, digest, json } from './io.mjs';

export function caseSources(test, sources = evidenceSources(), context) {
  // Integration scripts conservatively depend on all handlers. Direct Coreutils
  // cases omit unrelated leaf handlers; shared shell/VFS/parser changes invalidate all.
  // env invokes another executable, so its evidence depends on the child handler.
  if (test.softwareId !== 'coreutils' || test.script || test.command === 'env') return sources;
  const manifest = context?.manifest ?? json('content/cli-compatibility/manifest.json');
  const impl = manifest.native[test.command]?.implementation;
  const leaf = new Set([
    'src-tauri/src/terminal_text.rs',
    'src-tauri/src/terminal_query.rs',
    'src-tauri/src/terminal_transfer.rs',
    'src-tauri/src/terminal_remove.rs',
    'src-tauri/src/coreutils/bytes.rs',
    'src-tauri/src/coreutils/links.rs',
    'src-tauri/src/coreutils/backup.rs',
    'src-tauri/src/coreutils/links_tests.rs',
    'src-tauri/src/shell_pipeline/streams/links.rs',
    'src-tauri/src/coreutils/directories.rs',
    'src-tauri/src/coreutils/directories_tests.rs',
    'src-tauri/src/coreutils/pathnames.rs',
    'src-tauri/src/coreutils/pathnames_tests.rs',
    'src-tauri/src/vfs/canonical.rs',
    'src-tauri/src/coreutils/sha256sum.rs',
    'src-tauri/src/coreutils/sha256sum_tests.rs',
    'src-tauri/src/shell_pipeline/streams/sha256sum.rs',
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
  const filesystem = ['readlink', 'realpath', 'mkdir', 'rmdir', 'ln'].includes(test.command);
  return sources.filter((p) => {
    if (
      [
        'scripts/cli/filesystem-cases.mjs',
        'scripts/cli-filesystem-generate.mjs',
        'scripts/cli/filesystem.test.mjs',
      ].includes(p)
    )
      return filesystem;
    if (
      [
        'scripts/cli/sha256sum-cases.mjs',
        'scripts/cli-sha256sum-generate.mjs',
        'scripts/cli/sha256sum.test.mjs',
        'tests/cli/fixtures/sha256sum-records.json',
      ].includes(p)
    )
      return test.command === 'sha256sum';
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
        (foundation ||
          [
            'cat',
            'head',
            'tail',
            'base64',
            'tee',
            'wc',
            'sha256sum',
            'readlink',
            'realpath',
            'mkdir',
            'rmdir',
            'ln',
          ].includes(test.command)) &&
        p.includes('/' + test.command + '-')
      );
    if (p.endsWith('/foundation_options.rs') || p.endsWith('/foundation_messages.rs'))
      return (
        foundation ||
        [
          'cat',
          'head',
          'tail',
          'base64',
          'tee',
          'wc',
          'sha256sum',
          'readlink',
          'realpath',
          'mkdir',
          'rmdir',
          'ln',
        ].includes(test.command)
      );
    return (
      !leaf.has(p) ||
      p === impl ||
      (test.command === 'ln' &&
        (p.endsWith('/links_tests.rs') ||
          p.endsWith('/backup.rs') ||
          p.endsWith('/streams/links.rs'))) ||
      (['mkdir', 'rmdir'].includes(test.command) && p.endsWith('/directories_tests.rs')) ||
      (['readlink', 'realpath', 'ln'].includes(test.command) && p.endsWith('/vfs/canonical.rs')) ||
      (filesystem &&
        (p.endsWith('/pathnames_tests.rs') ||
          p.endsWith('/pathnames.rs') ||
          p.endsWith('/foundation.rs') ||
          p.endsWith('/wc.rs'))) ||
      (test.command === 'sha256sum' &&
        (p.endsWith('/sha256sum_tests.rs') ||
          p.endsWith('/streams/sha256sum.rs') ||
          p.endsWith('/wc.rs'))) ||
      (['head', 'tail', 'base64', 'tee', 'wc', 'sha256sum'].includes(test.command) &&
        p.endsWith('/cat.rs')) ||
      (test.command === 'wc' && (p.endsWith('/wc_tests.rs') || p.endsWith('/streams/wc.rs'))) ||
      (test.command === 'base64' && p.endsWith('/base64_tests.rs')) ||
      (test.command === 'tee' && p.endsWith('/tee_tests.rs')) ||
      (test.command === 'tail' && p.endsWith('/head.rs')) ||
      (['head', 'cat', 'basename', 'dirname', 'printenv', 'whoami'].includes(test.command) &&
        p.endsWith('/tail.rs'))
    );
  });
}
export function caseFingerprint(test, cache = new Map(), sources = evidenceSources(), context) {
  const paths = caseSources(test, sources, context);
  const key = paths.join('\0');
  if (!cache.has(key)) cache.set(key, digest(paths));
  const manifest = context?.manifest ?? json('content/cli-compatibility/manifest.json');
  const contract =
    test.softwareId === 'coreutils'
      ? (context?.coreutils ?? json('content/cli-compatibility/coreutils.json')).commands[
          test.command
        ]
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
  // One immutable configuration snapshot per calculation, never a global cache.
  // Avoid reparsing megabytes of gate metadata for every individual case.
  const context = {
    manifest: json('content/cli-compatibility/manifest.json'),
    coreutils: json('content/cli-compatibility/coreutils.json'),
  };
  return Object.fromEntries(cases.map((c) => [c.id, caseFingerprint(c, cache, sources, context)]));
}
