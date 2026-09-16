// Development only. Fixed Cargo command, no host invocation of player software.
import { cargo, write, digest, evidenceSources, json } from './cli/io.mjs';
import { loadCases, pipeline } from './cli/pipeline.mjs';
import { generate } from './cli/reports.mjs';
const cases = loadCases();
await generate(pipeline());
write('artifacts/cli-case-request.json', {
  schemaVersion: 2,
  cases: cases.map(({ id, command, argv, script, stdin, env, cwd, tty, fixture, inputEvents }) => ({
    id,
    command,
    argv,
    script,
    stdin,
    env,
    cwd,
    tty,
    fixture,
    inputEvents,
  })),
});
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
const actual = json('artifacts/cli-case-actual.json');
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
write('artifacts/cli-evidence.json', {
  schemaVersion: 2,
  fingerprint: digest(evidenceSources()),
  captureHash: digest(['artifacts/cli-case-actual.json']),
  performanceHash: digest(['artifacts/shell-performance.json']),
  runner: 'cli_tooling_capture',
  executedAt: new Date().toISOString(),
});
const report = pipeline();
await generate(report);
const failed = report.verification.caseResults.filter((t) => t.result !== 'PASS');
console.log(
  JSON.stringify({ cases: cases.length, failed: failed.length, summary: report.summary }, null, 2),
);
if (failed.length || report.verification.errors.length)
  throw new Error(
    [...failed.map((f) => `${f.id}: ${f.reason}`), ...report.verification.errors].join('\n'),
  );
