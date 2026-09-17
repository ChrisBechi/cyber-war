// Development only. Fixed Cargo command, no host invocation of player software.
import { cargo, write, digest, evidenceSources, json, root } from './cli/io.mjs';
import { existsSync } from 'node:fs';
import { resolve } from 'node:path';
import { loadCases, pipeline, currentCapture } from './cli/pipeline.mjs';
import { scope } from './cli/scope.mjs';
import { caseFingerprints } from './cli/fingerprint.mjs';
import { generate } from './cli/reports.mjs';
const selected = scope();
const baseline = json('content/cli-compatibility/manifest.json').software.coreutils
  .referenceVersion;
const closedCoreutils = new Set(
  Object.entries(json('content/cli-compatibility/coreutils.json').commands)
    .filter(
      ([name, s]) =>
        (s.area === 'foundation' || ['cat', 'head'].includes(name)) &&
        existsSync(resolve(root, `tests/cli/gnu/coreutils/${baseline}/${name}.json`)),
    )
    .map(([name]) => name),
);
const requiredSubsystemCases = new Set(
  json('content/cli-compatibility/subsystems.json').subsystems.flatMap((s) => [
    ...s.requiredTests,
    ...(s.capabilities ?? []).flatMap((c) => c.requiredTests),
  ]),
);
const cases = loadCases().filter(
  (c) =>
    selected.includes(c) ||
    (selected.names &&
      (requiredSubsystemCases.has(c.id) ||
        (c.softwareId === 'coreutils' && closedCoreutils.has(c.command)))),
);
const previous = selected.names ? (currentCapture()?.cases ?? []) : [];
await generate(pipeline());
const executionFingerprint = digest(evidenceSources());
const executionCaseFingerprints = caseFingerprints(loadCases());
write('artifacts/cli-case-request.json', {
  schemaVersion: 2,
  cases: cases.map(
    ({
      id,
      command,
      invocation,
      transport,
      process,
      io,
      interaction,
      argv,
      script,
      stdin,
      stdinHex,
      env,
      cwd,
      tty,
      fixture,
      inputEvents,
      roundtrip,
    }) => ({
      id,
      command,
      invocation,
      transport,
      process,
      io,
      interaction,
      argv,
      script,
      stdin,
      stdinHex,
      env,
      cwd,
      tty,
      fixture,
      inputEvents,
      roundtrip,
    }),
  ),
});
if (!selected.names)
  cargo([
    'test',
    '--offline',
    '--manifest-path',
    'src-tauri/Cargo.toml',
    '--lib',
    'cli_compat',
    '--',
    '--nocapture',
  ]);
cargo([
  'test',
  '--offline',
  '--manifest-path',
  'src-tauri/Cargo.toml',
  '--lib',
  'cli_tooling_capture',
  '--',
  '--ignored',
  '--nocapture',
]);
if (!selected.names)
  cargo([
    'test',
    '--offline',
    '--manifest-path',
    'src-tauri/Cargo.toml',
    '--lib',
    'vfs::',
    '--',
    '--nocapture',
  ]);
if (!selected.names)
  cargo([
    'test',
    '--offline',
    '--manifest-path',
    'src-tauri/Cargo.toml',
    '--lib',
    'cli_tooling_vfs_performance',
    '--',
    '--ignored',
    '--nocapture',
  ]);
cargo([
  'test',
  '--offline',
  '--manifest-path',
  'src-tauri/Cargo.toml',
  '--lib',
  'coreutils::',
  '--',
  '--nocapture',
]);
if (!selected.names)
  cargo([
    'test',
    '--offline',
    '--manifest-path',
    'src-tauri/Cargo.toml',
    '--lib',
    'cli_tooling_coreutils_performance',
    '--',
    '--ignored',
    '--nocapture',
  ]);
const actual = json('artifacts/cli-case-actual.json');
if (!selected.names)
  cargo([
    'test',
    '--offline',
    '--manifest-path',
    'src-tauri/Cargo.toml',
    '--lib',
    'cli_tooling_shell_performance',
    '--',
    '--ignored',
    '--nocapture',
  ]);
if (actual.cases.length !== cases.length) throw new Error('Incomplete capture');
if (digest(evidenceSources()) !== executionFingerprint)
  throw new Error(
    'Sources changed during capture; refusing to attach new fingerprints to old execution',
  );
if (selected.names) {
  const ids = new Set(actual.cases.map((c) => c.id));
  actual.cases.push(...previous.filter((c) => !ids.has(c.id)));
  write('artifacts/cli-case-actual.json', actual);
}
const currentIds = new Set(actual.cases.map((c) => c.id));

write('artifacts/cli-evidence.json', {
  schemaVersion: 2,
  fingerprint: selected.names ? null : executionFingerprint,
  caseFingerprints: Object.fromEntries(
    Object.entries(executionCaseFingerprints).filter(([id]) => currentIds.has(id)),
  ),
  captureHash: digest(['artifacts/cli-case-actual.json']),
  performanceHash: digest(['artifacts/shell-performance.json']),
  coreutilsPerformanceHash: digest(['artifacts/coreutils-performance.json']),
  vfsPerformanceHash: digest(['artifacts/vfs-performance.json']),
  runner: 'cli_tooling_capture',
  executedAt: new Date().toISOString(),
});
const report = pipeline();
await generate(report);
const failed = report.verification.caseResults.filter(
  (t) => cases.some((c) => c.id === t.id) && t.result !== 'PASS',
);
console.log(
  JSON.stringify({ cases: cases.length, failed: failed.length, summary: report.summary }, null, 2),
);
const errors = report.verification.errors.filter(
  (e) => !selected.names || !e.includes('without current required evidence'),
);
for (const c of report.commands.filter(selected.includes))
  if (c.gates.GNU_REFERENCE?.result === 'FAIL')
    errors.push(`${c.id}/GNU_REFERENCE: ${c.gates.GNU_REFERENCE.resultReason}`);
if (failed.length || errors.length)
  throw new Error([...failed.map((f) => `${f.id}: ${f.reason}`), ...errors].join('\n'));
