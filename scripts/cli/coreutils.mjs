import { z } from 'zod';
import { existsSync } from 'node:fs';
import { resolve } from 'node:path';
import { json, digest, root } from './io.mjs';
import {
  referenceCapture,
  validateReference,
  referenceDifference,
} from './coreutils-reference.mjs';

const contract = z
  .object({
    id: z.string(),
    applicability: z.enum(['REQUIRED', 'OPTIONAL', 'OUT_OF_SCOPE']),
    description: z.string().min(12),
    evidence: z.array(z.string()),
  })
  .strict();
export const coreutilsSchema = z
  .object({
    schemaVersion: z.literal(1),
    softwareId: z.literal('coreutils'),
    referenceVersion: z.string(),
    commands: z.record(
      z.string(),
      z
        .object({
          area: z.enum([
            'foundation',
            'reading',
            'filesystem',
            'manipulation',
            'text',
            'execution',
            'advanced',
          ]),
          complexity: z.number().int().min(1).max(5),
          before: z.enum(['PARTIAL', 'UNVERIFIED', 'VERIFIED']),
          implementation: z.string(),
          flags: z.array(z.string()),
          contracts: z.array(contract).min(1),
          knownGaps: z.array(z.string()),
          unsupportedFlagsPolicy: z.string(),
          referenceRequired: z.literal(true),
          subsystems: z.array(z.string()),
          intentionalDeviations: z.array(z.string()),
        })
        .strict(),
    ),
  })
  .strict();

export function attachCoreutils(
  inventory,
  config = coreutilsSchema.parse(json('content/cli-compatibility/coreutils.json')),
) {
  const family = inventory.software.find((s) => s.id === 'coreutils');
  if (family.referenceVersion !== config.referenceVersion)
    throw new Error('Coreutils contract baseline mismatch');
  const commands = inventory.commands.filter((c) => c.softwareId === 'coreutils');
  const names = new Set(commands.map((c) => c.command));
  for (const name of Object.keys(config.commands))
    if (!names.has(name)) throw new Error(`Orphan Coreutils contract ${name}`);
  for (const c of commands) {
    const spec = config.commands[c.command];
    if (!spec) throw new Error(`Missing Coreutils contract ${c.command}`);
    if (c.implementation !== spec.implementation)
      throw new Error(`Coreutils implementation drift ${c.command}`);
    if (new Set(spec.contracts.map((x) => x.id)).size !== spec.contracts.length)
      throw new Error(`Duplicate Coreutils contract ${c.command}`);
    c.coreutils = spec;
    c.requirements = [...new Set([...c.requirements, ...spec.subsystems])];
    for (const name of ['COMMAND_CONTRACTS', 'GNU_REFERENCE', 'KNOWN_GAPS'])
      c.gates[name] = {
        applicability: 'REQUIRED',
        reason: 'All committed contracts need passing project and pinned GNU reference evidence.',
        testIds: [],
      };
  }
  return inventory;
}

export function coreutilsGate(command, name, byCase, results, actuals = new Map()) {
  if (name === 'KNOWN_GAPS')
    return {
      result: command.coreutils.knownGaps.length ? 'SKIPPED' : 'PASS',
      reason: command.coreutils.knownGaps.join('; ') || 'No remaining known contract blockers',
    };
  const contracts =
    command.coreutils?.contracts.filter((c) => c.applicability === 'REQUIRED') ?? [];
  const missing = contracts.filter(
    (c) =>
      !c.evidence.length ||
      c.evidence.some(
        (id) => byCase.get(id)?.command !== command.command || results.get(id)?.result !== 'PASS',
      ),
  );
  const coveredFlags = new Set(
    [...byCase.values()]
      .filter((c) => c.command === command.command && results.get(c.id)?.result === 'PASS')
      .flatMap((c) => c.flags ?? []),
  );
  const missingFlags = (command.coreutils?.flags ?? []).filter((f) => !coveredFlags.has(f));
  if (name === 'COMMAND_CONTRACTS')
    return {
      result: missing.length || missingFlags.length ? 'SKIPPED' : 'PASS',
      reason:
        missing.length || missingFlags.length
          ? `Missing/failing required contracts: ${missing.map((c) => c.id).join(', ')}; flags without explicit evidence: ${missingFlags.join(', ')}`
          : 'All linked project contracts and supported flags passed',
    };
  const path = `tests/cli/gnu/coreutils/${command.referenceVersion}/capture.json`;
  if (
    command.coreutils.area === 'foundation' ||
    ['cat', 'head', 'tail', 'base64', 'tee', 'wc'].includes(command.command)
  ) {
    const capture = referenceCapture(command),
      invalid = validateReference(command, capture);
    if (invalid) return { result: capture ? 'FAIL' : 'SKIPPED', reason: invalid };
    const cases = [...byCase.values()].filter((c) => c.command === command.command);
    const differences = cases
      .map((c) => [
        c.id,
        referenceDifference(
          c,
          actuals.get(c.id),
          capture.cases.find((r) => r.id === c.id),
        ),
      ])
      .filter(([, e]) => e);
    return {
      result: differences.length || !cases.length ? 'FAIL' : 'PASS',
      reason: differences.length
        ? differences.map(([id, e]) => `${id}: ${e}`).join('\n')
        : `${cases.length} cases matched GNU bytes, status and filesystem effects`,
    };
  }
  if (!existsSync(resolve(root, path)))
    return {
      result: 'SKIPPED',
      reason: `No captured GNU ${command.referenceVersion} evidence; project expectations do not qualify`,
    };
  const reference = json(path);
  if (
    reference.provenance !== 'GNU_REFERENCE' ||
    reference.version !== command.referenceVersion ||
    reference.locale !== 'C' ||
    !reference.environment?.binaryHashes?.[command.command]?.sha256?.match(/^[a-f0-9]{64}$/) ||
    !reference.environment.binaryHashes[command.command].versionLine?.endsWith(
      `(GNU coreutils) ${command.referenceVersion}`,
    ) ||
    reference.caseDigest !== digest(['tests/cli/compat/pilot/coreutils-foundation.json'])
  )
    return {
      result: 'FAIL',
      reason: 'Reference version, provenance, environment or fixture digest mismatch',
    };
  const ids = [...new Set(contracts.flatMap((c) => c.evidence))];
  const absent = ids.filter((id) => {
    const row = reference.cases.find((c) => c.id === id),
      test = byCase.get(id);
    if (!row || !test || row.reproducible !== true || results.get(id)?.result !== 'PASS')
      return true;
    const expected = test.expected;
    return (
      row.stdoutHex !==
        (expected.stdoutHex ?? Buffer.from(expected.stdout.value ?? '').toString('hex')) ||
      expected.stdout.kind !== 'EXACT' ||
      expected.stderr.kind !== 'EXACT' ||
      row.stderrHex !== Buffer.from(expected.stderr.value).toString('hex') ||
      row.exitCode !== expected.exitCode ||
      (expected.state.length > 0 && !row.stateEquivalent)
    );
  });
  return {
    result: missing.length || absent.length || !ids.length ? 'SKIPPED' : 'PASS',
    reason: absent.length
      ? `Missing/divergent GNU reference cases: ${absent.join(', ')}`
      : 'Pinned, reproducible reference matched every committed case',
  };
}

export function coreutilsReport(report) {
  const ranks = {
    foundation: 1,
    reading: 2,
    filesystem: 3,
    manipulation: 4,
    text: 5,
    execution: 6,
    advanced: 7,
  };
  const cases = report.verification.caseCoverage;
  const rows = report.commands
    .filter((c) => c.coreutils)
    .map((c) => {
      const spec = c.coreutils;
      const coverage = cases.filter((t) => t.command === c.command && t.softwareId === 'coreutils');
      const contracts = spec.contracts.map((x) => ({
        ...x,
        result:
          x.applicability === 'OUT_OF_SCOPE'
            ? 'OUT_OF_SCOPE'
            : x.evidence.length &&
                x.evidence.every(
                  (id) =>
                    report.verification.caseResults.find((r) => r.id === id)?.result === 'PASS',
                )
              ? 'PASS'
              : 'NO_EVIDENCE',
      }));
      const gates = Object.values(c.gates).filter((g) => g.applicability === 'REQUIRED');
      const blockers = [
        ...c.blockedBy,
        ...c.missingGates.map((g) => `GATE:${g}`),
        ...spec.knownGaps.map((g) => `GAP:${g}`),
      ];
      const reference = c.gates.GNU_REFERENCE.result === 'PASS';
      return {
        command: c.command,
        aliases: report.commands
          .filter((a) => a.aliasOf === c.command || a.aliasOf === c.id)
          .map((a) => a.command),
        baseline: c.referenceVersion,
        implementation: c.implementation,
        implementationKind: c.implementationKind,
        capabilities: c.capabilities,
        flags: spec.flags,
        contracts,
        wave: ranks[spec.area],
        area: spec.area,
        complexity: spec.complexity,
        missionUses: c.usedByMissions ?? [],
        before: spec.before,
        after: c.effectiveStatus,
        projectCases: coverage.filter((t) => t.result === 'PASS').length,
        referenceCases:
          spec.area === 'foundation' ||
          ['cat', 'head', 'tail', 'base64', 'tee', 'wc'].includes(c.command)
            ? (referenceCapture(c)?.cases.length ?? 0)
            : reference
              ? c.evidenceIds.length
              : 0,
        referenceState: c.gates.GNU_REFERENCE.result,
        matchedReferenceCases: reference ? coverage.filter((t) => t.result === 'PASS').length : 0,
        requiredGates: gates.length,
        passedGates: gates.filter((g) => g.result === 'PASS').length,
        blockers,
        action: c.blockedBy.length
          ? 'NEEDS_SUBSYSTEM'
          : c.gates.COMMAND_CONTRACTS.result !== 'PASS' || spec.knownGaps.length
            ? 'NEEDS_IMPLEMENTATION_AND_EVIDENCE'
            : !reference
              ? 'NEEDS_REFERENCE'
              : 'NEEDS_REMAINING_GATES',
      };
    })
    .sort(
      (a, b) =>
        Number(a.blockers.some((x) => x.startsWith('SUBSYSTEM:'))) -
          Number(b.blockers.some((x) => x.startsWith('SUBSYSTEM:'))) ||
        a.wave - b.wave ||
        b.missionUses.length - a.missionUses.length ||
        a.complexity - b.complexity ||
        b.projectCases - a.projectCases ||
        a.command.localeCompare(b.command),
    );
  const totals = Object.fromEntries(
    ['VERIFIED', 'PARTIAL', 'UNVERIFIED'].map((s) => [s, rows.filter((c) => c.after === s).length]),
  );
  return {
    schemaVersion: 1,
    baseline: report.software.find((s) => s.id === 'coreutils').referenceVersion,
    policy:
      'Discover registered membership; order by dependency readiness, semantic area, mission use, complexity and existing passing evidence. Project expectations cannot satisfy GNU_REFERENCE.',
    totals,
    commands: rows,
    next: rows.find((c) => c.after !== 'VERIFIED') ?? null,
  };
}

export function coreutilsMarkdown(report) {
  const r = coreutilsReport(report);
  const cell = (v) => String(v).replaceAll('|', '\\|').replaceAll('\n', ' ');
  const rows = r.commands.map((c) =>
    [
      c.command,
      c.wave,
      c.baseline,
      c.implementation,
      c.contracts.filter((x) => x.result === 'PASS').length + '/' + c.contracts.length,
      c.referenceCases,
      c.projectCases,
      `${c.passedGates}/${c.requiredGates}`,
      c.before,
      c.after,
      c.blockers.join('; '),
    ].map(cell),
  );
  return (
    '# Coreutils compatibility\n\nGenerated from discovery, contracts and current evidence.\n\n' +
    JSON.stringify(r.totals) +
    '\n\n' +
    r.policy +
    '\n\n|Command|Wave|Baseline|Implementation|Contracts|GNU reference cases|Project cases|Required gates|Before|After|Blockers|\n|---|---|---|---|---|---|---|---|---|---|---|\n' +
    rows.map((r) => '|' + r.join('|') + '|').join('\n') +
    '\n\n## Executable queue\n\n' +
    r.commands
      .filter((c) => c.after !== 'VERIFIED')
      .map((c) => `- coreutils/${c.command}: ${c.action}`)
      .join('\n') +
    '\n'
  );
}
