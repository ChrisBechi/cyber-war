import { existsSync } from 'node:fs';
import { resolve } from 'node:path';
import { pipeline } from './cli/pipeline.mjs';
import { generate } from './cli/reports.mjs';
import { regression } from './cli/verification.mjs';
import { json, write, root } from './cli/io.mjs';
import { overridesSchema, debtSchema } from './cli/schema.mjs';
import { format } from 'prettier';
import { scope } from './cli/scope.mjs';
const selected = scope();
const report = pipeline();
const errors = [...report.verification.errors];
for (const c of report.commands.filter(selected.includes))
  if (c.gates.GNU_REFERENCE?.result === 'FAIL')
    errors.push(`${c.id}/GNU_REFERENCE: ${c.gates.GNU_REFERENCE.resultReason}`);
for (const test of report.verification.caseResults)
  if (
    test.result !== 'PASS' &&
    report.commands.some(
      (c) =>
        c.command === report.verification.caseCoverage.find((x) => x.id === test.id)?.command &&
        selected.includes(c),
    )
  )
    errors.push(`Pilot ${test.id}: ${test.result}; run cli:compat with current sources`);
if (report.verification.hostIsolation.result !== 'PASS')
  errors.push(
    'Host isolation failed: ' + JSON.stringify(report.verification.hostIsolation.failures),
  );
const saved = json('docs/cli/inventory.json').commands.map((c) => c.command);
if (JSON.stringify(saved) !== JSON.stringify(report.commands.map((c) => c.command)))
  errors.push('Checked-in command inventory drift');
if (process.argv.includes('--strict'))
  for (const c of report.commands)
    if (
      c.classification === 'REAL_COMPAT' &&
      selected.includes(c) &&
      c.requirement === 'REQUIRED' &&
      c.implementationKind !== 'LAUNCHER' &&
      c.effectiveStatus !== 'VERIFIED'
    )
      errors.push(`Strict: ${c.id} ${c.effectiveStatus}`);
const baselineIndex = process.argv.indexOf('--baseline');
const baseline =
  baselineIndex >= 0
    ? process.argv[baselineIndex + 1]
    : 'docs/cli-compatibility/debt-baseline.json';
const overrides = overridesSchema.parse(json('content/cli-compatibility/overrides.json')).overrides;
if (baselineIndex >= 0 && (!baseline || !existsSync(resolve(root, baseline))))
  throw new Error('Requested comparison baseline does not exist');
if (baseline && existsSync(resolve(root, baseline))) {
  const diff = regression(debtSchema.parse(json(baseline)), report, overrides);
  errors.push(...diff.violations);
  write('artifacts/cli-diff.json', diff);
  write(
    'docs/generated/cli-diff.md',
    await format(
      '# CLI report diff\n\n' +
        (diff.changes.length
          ? diff.changes.map((c) => '- ' + JSON.stringify(c)).join('\n')
          : 'No command, gate, version or subsystem changes.') +
        '\n',
      { parser: 'markdown' },
    ),
  );
}
await generate(report);
write('artifacts/cli-verify-result.json', {
  schemaVersion: 2,
  passed: errors.length === 0,
  strict: process.argv.includes('--strict'),
  errors,
  summary: report.summary,
});
console.log(
  JSON.stringify(
    { passed: errors.length === 0, errors: errors.length, summary: report.summary },
    null,
    2,
  ),
);
if (errors.length) throw new Error(errors.slice(0, 30).join('\n'));
