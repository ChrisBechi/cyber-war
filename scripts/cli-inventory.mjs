import { performance } from 'node:perf_hooks';
import { pipeline } from './cli/pipeline.mjs';
import { generate } from './cli/reports.mjs';
import { json, write } from './cli/io.mjs';
import { manifestSchema } from './cli/schema.mjs';
const started = performance.now();
const report = pipeline();
const inventoryMs = performance.now() - started;
if (process.argv.includes('--check')) {
  const names = json('docs/cli/inventory.json').commands.map((c) => c.command);
  if (JSON.stringify(names) !== JSON.stringify(report.commands.map((c) => c.command)))
    throw new Error('Discovery changed; regenerate cli:inventory');
} else await generate(report);
const index = new Map(report.commands.map((c) => [c.command, c]));
const software = new Map(report.software.map((s) => [s.id, s]));
function measure(operation) {
  const began = performance.now();
  let hits = 0;
  for (let i = 0; i < 100_000; i++) if (operation()) hits++;
  return { ms: performance.now() - began, iterations: 100_000, hits };
}
const loadStart = performance.now();
manifestSchema.parse(json('content/cli-compatibility/manifest.json'));
const manifestLoadMs = performance.now() - loadStart;
write('artifacts/cli-performance.json', {
  inventoryMs,
  inventoryAndReportsMs: performance.now() - started,
  manifestLoadMs,
  commandLookup: measure(() => index.get('ls')),
  softwareLookup: measure(() => software.get('coreutils')),
  verificationLookup: measure(() => index.get('ls')?.gates.STDOUT),
  commandCount: index.size,
  softwareCount: software.size,
  lookupImplementation: 'Map indexes; runtime BTreeMap/BTreeSet',
});
console.log(JSON.stringify(report.summary, null, 2));
if (report.verification.errors.length) throw new Error(report.verification.errors.join('\n'));
