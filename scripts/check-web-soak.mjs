import fs from 'node:fs';
import path from 'node:path';

const directory = path.resolve(process.argv[2] ?? 'artifacts/desktop-web-soak');
const records = fs
  .readFileSync(path.join(directory, 'web-soak.jsonl'), 'utf8')
  .trim()
  .split(/\r?\n/)
  .filter(Boolean)
  .map((line) => JSON.parse(line));
// A short preparation run may precede the full run in the same process.
const start = records.findLastIndex((row) => row.sample.state === 'started');
if (start < 0) throw new Error('No started run found');
const run = records.slice(start);
const last = run.at(-1);
const mounted = run.filter((row) => row.sample.state === 'running');
const unmounted = run.filter((row) => row.sample.state === 'unmounted');
const range = (values) => ({
  min: Math.min(...values),
  max: Math.max(...values),
  first: values[0],
  last: values.at(-1),
});
const failures = [];
if (last.sample.state !== 'completed') failures.push(`Run is ${last.sample.state}`);
if (last.sample.elapsedSeconds < 7200) failures.push('Less than two hours elapsed');
if (new Set(run.map((row) => row.backend.slot)).size !== 5)
  failures.push('Not all five slots were exercised');
if (run.some((row) => row.sample.errors.length)) failures.push('Frontend errors recorded');
if (run.some((row) => row.backend.quickCheck !== 'ok' || !row.backend.autocommit))
  failures.push('SQLite health check failed');
if (run.some((row) => row.backend.history > 200 || row.backend.materializationCache > 64))
  failures.push('Cache or history limit exceeded');
for (const target of ['window', 'document']) {
  const baseline = unmounted[0]?.sample.globalListeners[target];
  if (unmounted.some((row) => row.sample.globalListeners[target] !== baseline))
    failures.push(`${target} listeners changed after unmount`);
}
const summary = {
  status: failures.length ? 'incomplete-or-failed' : 'passed',
  failures,
  runId: last.sample.runId,
  elapsedSeconds: last.sample.elapsedSeconds,
  navigations: last.sample.iterations,
  persistedActions: last.sample.actions,
  slots: [...new Set(run.map((row) => row.backend.slot))],
  samples: run.length,
  mountCycles: unmounted.length,
  sqliteQuickCheck: [...new Set(run.map((row) => row.backend.quickCheck))],
  databaseBytes: range(run.map((row) => row.backend.databaseBytes)),
  domNodes: mounted.length ? range(mounted.map((row) => row.sample.domNodes)) : null,
  unmountedListeners: unmounted[0]?.sample.globalListeners,
  latencyP95Ms: mounted.length ? range(mounted.map((row) => row.sample.latencyMs.p95)) : null,
  visibility: [...new Set(run.map((row) => row.sample.visibility))],
  limitations: [
    'Navigation timing includes a 400 ms observation delay and polling; it is not pure IPC latency.',
    'Listener measurements cover explicit window/document add/remove calls, excluding once, signal and node listeners.',
    'Memory and OS handle measurements are in the companion process-tree CSV; this report alone cannot establish absence of leaks.',
    'One physical host; no independent human evaluation or clean-machine installer certification.',
  ],
};
fs.writeFileSync(
  path.join(directory, 'web-soak-summary.json'),
  JSON.stringify(summary, null, 2) + '\n',
);
console.log(JSON.stringify(summary, null, 2));
if (failures.length) process.exitCode = 1;
