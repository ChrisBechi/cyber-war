import { attachCoreutils } from './coreutils.mjs';
import { caseFingerprints } from './fingerprint.mjs';
import { existsSync } from 'node:fs';
import { resolve } from 'node:path';
import { caseSchema, referenceFixtureSchema } from './schema.mjs';
import { files, json, read, root, digest, evidenceSources, runtimeRegistry } from './io.mjs';
import { loadDiscovery } from './discovery.mjs';
import { evaluate, metrics, queue } from './verification.mjs';
import { hostGuard } from './host-guard.mjs';
import { readCapture } from './capture-storage.mjs';
export function loadCases() {
  const cases = files('tests/cli/compat/pilot')
    .filter((p) => p.endsWith('.json'))
    .flatMap((p) => json(p).map((c) => caseSchema.parse(c)));
  const references = files('tests/cli/references')
    .filter((p) => p.endsWith('.json'))
    .map((p) => referenceFixtureSchema.parse(json(p)));
  const seen = new Set();
  for (const ref of references)
    for (const id of ref.caseIds) {
      if (seen.has(id)) throw new Error(`Duplicate versioned reference case ${id}`);
      seen.add(id);
      const test = cases.find((c) => c.id === id);
      if (!test || test.softwareId !== ref.softwareId || test.reference.version !== ref.version)
        throw new Error(`Versioned reference mismatch ${id}`);
    }
  for (const test of cases)
    if (!seen.has(test.id)) throw new Error(`No versioned reference for ${test.id}`);
  return cases;
}
export function currentCapture() {
  const path = resolve(root, 'artifacts/cli-evidence.json');
  if (!existsSync(path)) return null;
  const evidence = json('artifacts/cli-evidence.json');
  if (evidence.schemaVersion !== 2) return null;
  const fresh = evidence.fingerprint === digest(evidenceSources());
  if (!fresh && !evidence.caseFingerprints) return null;
  const raw = 'artifacts/cli-case-actual.json';
  if (!existsSync(resolve(root, raw)) || digest([raw]) !== evidence.captureHash) return null;
  const performance = 'artifacts/shell-performance.json';
  const shellPerformance =
    evidence.performanceHash &&
    existsSync(resolve(root, performance)) &&
    digest([performance]) === evidence.performanceHash
      ? json(performance)
      : null;
  const vfsPath = 'artifacts/vfs-performance.json';
  const vfsPerformance =
    evidence.vfsPerformanceHash &&
    existsSync(resolve(root, vfsPath)) &&
    digest([vfsPath]) === evidence.vfsPerformanceHash
      ? json(vfsPath)
      : null;
  const perfPath = 'artifacts/coreutils-performance.json';
  const coreutilsPerformance =
    fresh &&
    evidence.coreutilsPerformanceHash &&
    existsSync(resolve(root, perfPath)) &&
    digest([perfPath]) === evidence.coreutilsPerformanceHash
      ? json(perfPath)
      : null;
  const captured = readCapture(raw);
  if (!fresh) {
    const current = caseFingerprints(loadCases());
    captured.cases = captured.cases.filter(
      (c) => evidence.caseFingerprints[c.id] === current[c.id],
    );
  }
  return {
    ...captured,
    coreutilsPerformance,
    fingerprint: evidence.fingerprint,
    shellPerformance: fresh ? shellPerformance : null,
    vfsPerformance: fresh ? vfsPerformance : null,
  };
}
export function pipeline(runtime = runtimeRegistry()) {
  runtimeNamesSourceCheck(runtime);
  const report = evaluate(
    attachCoreutils(loadDiscovery(runtime)),
    loadCases(),
    currentCapture(),
    hostGuard(),
  );
  report.summary = metrics(report);
  report.queue = queue(report);
  delete report.performance;
  return report;
}
export function runtimeNamesSourceCheck(runtime) {
  const native = [
    ...read('src-tauri/src/terminal.rs')
      .match(/pub const COMMANDS[^=]*= &\[([\s\S]*?)\];/)[1]
      .matchAll(/"([^"]+)"/g),
  ].map((m) => m[1]);
  if (JSON.stringify(native) !== JSON.stringify(runtime.native))
    throw new Error('Stale runtime registry export');
}
