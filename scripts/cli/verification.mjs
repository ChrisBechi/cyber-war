import { coreutilsGate, coreutilsReport } from './coreutils.mjs';
import { compareCase } from './matchers.mjs';
export function evaluate(
  inventory,
  cases = [],
  capture = null,
  host = { result: 'FAIL', failures: [] },
) {
  const errors = [];
  const byCase = new Map(cases.map((c) => [c.id, c]));
  const actuals = new Map((capture?.cases ?? []).map((c) => [c.id, c]));
  const caseResults = new Map();
  if (byCase.size !== cases.length) errors.push('Duplicate compatibility case ID');
  if (actuals.size !== (capture?.cases.length ?? 0)) errors.push('Duplicate captured case ID');
  const commandIndex = new Map(inventory.commands.map((c) => [c.command, c]));
  for (const test of cases) {
    const errorCount = errors.length;
    const command = commandIndex.get(test.command);
    if (!command || command.softwareId !== test.softwareId)
      errors.push(`Orphan or wrong software case ${test.id}`);
    if (
      test.reference.softwareId !== test.softwareId ||
      test.reference.version !== command?.referenceVersion
    )
      errors.push(`Case reference version mismatch ${test.id}`);
    if (
      test.gates.some((g) =>
        ['SIDE_EFFECTS', 'STATE_CONSISTENCY', 'CROSS_TOOL_CONSISTENCY'].includes(g),
      ) &&
      !test.expected.state.length
    )
      errors.push(`Missing state assertions ${test.id}`);
    if (test.gates.includes('ERRORS') && !test.errorCase)
      errors.push(`Error gate without error case ${test.id}`);
    const actual = actuals.get(test.id);
    caseResults.set(
      test.id,
      errors.length > errorCount
        ? { id: test.id, result: 'FAIL', reason: errors.slice(errorCount).join('; ') }
        : actual
          ? compareCase(test, actual)
          : { id: test.id, result: 'SKIPPED', reason: 'No current captured execution' },
    );
  }
  for (const id of actuals.keys()) if (!byCase.has(id)) errors.push(`Unknown captured case ${id}`);
  const subsystems = inventory.subsystems.map((s) => {
    const capabilities = (s.capabilities ?? []).map((capability) => {
      const ready =
        capability.state === 'READY' &&
        capability.requiredTests.length > 0 &&
        capability.requiredTests.every((id) => caseResults.get(id)?.result === 'PASS') &&
        host.result === 'PASS';
      if (!capability.id.startsWith(`${s.id}.`))
        errors.push(`Capability parent mismatch ${capability.id}`);
      if (
        capability.state === 'READY' &&
        !ready &&
        (capture || capability.requiredTests.length === 0)
      )
        errors.push(`Capability ${capability.id} claims READY without current required evidence`);
      return {
        ...capability,
        effectiveState: ready ? 'READY' : capability.state === 'MISSING' ? 'MISSING' : 'PARTIAL',
      };
    });
    const ready =
      s.state === 'READY' &&
      s.requiredTests.length > 0 &&
      s.requiredTests.every((id) => caseResults.get(id)?.result === 'PASS');
    if (s.state === 'READY' && !ready && (capture || s.requiredTests.length === 0))
      errors.push(`Subsystem ${s.id} claims READY without current required evidence`);
    return {
      ...s,
      capabilities,
      effectiveState: ready ? 'READY' : s.state === 'MISSING' ? 'MISSING' : 'PARTIAL',
    };
  });
  const subIndex = new Map(subsystems.flatMap((s) => [s, ...s.capabilities]).map((s) => [s.id, s]));
  const softwareIndex = new Map(inventory.software.map((s) => [s.id, s]));
  const commands = inventory.commands.map((command) => {
    const gates = {};
    const family = softwareIndex.get(command.softwareId);
    for (const [name, spec] of Object.entries(command.gates).sort()) {
      let result = 'SKIPPED',
        reason = 'No linked current evidence';
      if (spec.applicability === 'NOT_APPLICABLE') reason = spec.reason;
      else if (
        command.coreutils &&
        ['COMMAND_CONTRACTS', 'GNU_REFERENCE', 'KNOWN_GAPS'].includes(name)
      ) {
        ({ result, reason } = coreutilsGate(command, name, byCase, caseResults, actuals));
      } else if (name === 'DISCOVERY') {
        result = 'PASS';
        reason = 'Runtime registry and validated manifest agree';
      } else if (name === 'REFERENCE_PINNED') {
        result =
          family.referenceVersion && family.upstream && family.referenceSources.length
            ? 'PASS'
            : 'SKIPPED';
        reason =
          result === 'PASS'
            ? 'Version and reference metadata pinned'
            : 'Version/upstream/reference data missing';
      } else if (name === 'HOST_ISOLATION') {
        result = host.result;
        reason =
          result === 'PASS' ? 'Current source/build guard passed' : 'Host boundary guard failed';
      } else if (spec.testIds.length) {
        const tests = spec.testIds.map((id) => {
          const test = byCase.get(id);
          if (!test || test.command !== command.command || !test.gates.includes(name)) {
            errors.push(`Invalid ${command.command}/${name} evidence link ${id}`);
            return { result: 'FAIL' };
          }
          return caseResults.get(id);
        });
        result = tests.some((t) => t.result === 'FAIL')
          ? 'FAIL'
          : tests.every((t) => t.result === 'PASS')
            ? 'PASS'
            : 'SKIPPED';
        reason = tests.map((t, i) => `${spec.testIds[i]}: ${t.result}`).join('; ');
      }
      gates[name] = { ...spec, result, resultReason: reason };
    }
    const blockedBy = command.requirements
      .filter((r) => subIndex.get(r)?.effectiveState !== 'READY')
      .map((r) => `SUBSYSTEM:${r}:${subIndex.get(r)?.effectiveState ?? 'MISSING'}`);
    const missingGates = Object.entries(gates)
      .filter(([, g]) => g.applicability === 'REQUIRED' && g.result !== 'PASS')
      .map(([name]) => name);
    const implemented = !['CATALOG_ONLY', 'LAUNCHER'].includes(command.implementationKind);
    const referenceOk =
      command.classification === 'FICTIONAL_NATIVE' || gates.REFERENCE_PINNED.result === 'PASS';
    const effectiveStatus =
      implemented && referenceOk && !blockedBy.length && !missingGates.length
        ? 'VERIFIED'
        : implemented
          ? 'PARTIAL'
          : 'UNVERIFIED';
    if (command.declaredStatus === 'VERIFIED' && effectiveStatus !== 'VERIFIED')
      errors.push(`Fake VERIFIED command: ${command.id}`);
    return {
      ...command,
      gates,
      blockedBy,
      missingGates,
      effectiveStatus,
      status: effectiveStatus,
      evidenceIds: [
        ...new Set(
          Object.values(gates)
            .filter((g) => g.result === 'PASS')
            .flatMap((g) => g.testIds),
        ),
      ].sort(),
    };
  });
  const commandIds = new Map(commands.map((c) => [c.id, c]));
  // Dependencies are acyclic (validated by discovery); resolve families top-down.
  const software = [];
  const computed = new Map();
  function familyStatus(spec) {
    if (computed.has(spec.id)) return computed.get(spec.id);
    const executable = spec.executables
      .map((id) => commandIds.get(id))
      .filter((c) => c.requirement === 'REQUIRED');
    const aliases = spec.aliases.map((id) => commandIds.get(id));
    const dependencies = spec.dependencies.map((id) => familyStatus(softwareIndex.get(id)));
    const blockedBy = [
      ...new Set([
        ...executable.flatMap((c) => c.blockedBy),
        ...aliases.flatMap((c) => c.blockedBy),
        ...(spec.requirements ?? [])
          .filter((r) => subIndex.get(r)?.effectiveState !== 'READY')
          .map((r) => `SUBSYSTEM:${r}:${subIndex.get(r)?.effectiveState ?? 'MISSING'}`),
        ...dependencies
          .filter((s) => s.effectiveStatus !== 'VERIFIED')
          .map((s) => `SOFTWARE:${s.id}:${s.effectiveStatus}`),
      ]),
    ].sort();
    const all =
      executable.length > 0 &&
      executable.every((c) => c.effectiveStatus === 'VERIFIED') &&
      aliases.every((c) => c.gates.ALIAS_RESOLUTION.result === 'PASS') &&
      !blockedBy.length;
    const effectiveStatus = all
      ? 'VERIFIED'
      : executable.some((c) => c.effectiveStatus !== 'UNVERIFIED')
        ? 'PARTIAL'
        : 'UNVERIFIED';
    if (spec.declaredStatus === 'VERIFIED' && effectiveStatus !== 'VERIFIED')
      errors.push(`Fake VERIFIED software: ${spec.id}`);
    const row = {
      ...spec,
      effectiveStatus,
      blockedBy,
      verificationSummary: {
        requiredExecutables: executable.length,
        verifiedExecutables: executable.filter((c) => c.effectiveStatus === 'VERIFIED').length,
        aliasResolutionPassed: aliases.every((c) => c.gates.ALIAS_RESOLUTION.result === 'PASS'),
      },
    };
    computed.set(spec.id, row);
    return row;
  }
  for (const spec of inventory.software) software.push(familyStatus(spec));
  // Dependencies must block commands too, not just the aggregate family.
  for (const command of commands) {
    const deps = computed
      .get(command.softwareId)
      .blockedBy.filter((b) => b.startsWith('SOFTWARE:'));
    if (deps.length) {
      command.blockedBy.push(...deps);
      if (command.effectiveStatus === 'VERIFIED')
        command.status = command.effectiveStatus = 'PARTIAL';
      if (command.declaredStatus === 'VERIFIED')
        errors.push(`Dependency-blocked VERIFIED command: ${command.id}`);
    }
  }
  return {
    ...inventory,
    commands,
    software,
    subsystems,
    coreutilsPerformance: capture?.coreutilsPerformance ?? null,
    shellPerformance: capture?.shellPerformance ?? null,
    vfsPerformance: capture?.vfsPerformance ?? null,
    verification: {
      errors,
      caseResults: [...caseResults.values()],
      caseCoverage: cases.map((c) => ({
        id: c.id,
        command: c.command,
        softwareId: c.softwareId,
        flags: c.flags ?? [],
        errorCase: !!c.errorCase,
        result: caseResults.get(c.id).result,
      })),
      hostIsolation: host,
      optionalFailurePolicy:
        'Optional failures are reported but do not block; REQUIRED SKIPPED blocks.',
      evidenceFingerprint: capture?.fingerprint ?? null,
    },
  };
}

export function metrics(inventory) {
  const counts = (rows) =>
    Object.fromEntries(
      ['VERIFIED', 'PARTIAL', 'UNVERIFIED'].map((s) => [
        s,
        rows.filter((r) => r.effectiveStatus === s).length,
      ]),
    );
  const unique = inventory.commands.filter(
    (c) => !['ALIAS', 'LAUNCHER'].includes(c.implementationKind),
  );
  const real = unique.filter((c) => c.classification === 'REAL_COMPAT');
  const realSoftware = inventory.software.filter(
    (s) => s.classification === 'REAL_COMPAT' && s.executables.length,
  );
  const ratio = (rows) => ({
    verified: rows.filter((r) => r.effectiveStatus === 'VERIFIED').length,
    total: rows.length,
    percent: rows.length
      ? (100 * rows.filter((r) => r.effectiveStatus === 'VERIFIED').length) / rows.length
      : 0,
  });
  const gates = unique.flatMap((c) => Object.values(c.gates));
  const coverage = inventory.verification.caseCoverage;
  return {
    commandNames: inventory.commands.length,
    uniqueExecutables: unique.length,
    aliases: inventory.commands.filter((c) => c.implementationKind === 'ALIAS').length,
    launcherNames: inventory.commands.filter((c) => c.implementationKind === 'LAUNCHER').length,
    catalogLaunchers: inventory.launchers.filter((l) => l.kind === 'CATALOG').length,
    guiLaunchers: inventory.launchers.filter((l) => l.kind === 'GUI').length,
    softwareGroups: inventory.software.length,
    realCompat: real.length,
    fictionalNative: unique.filter((c) => c.classification === 'FICTIONAL_NATIVE').length,
    catalogOnly: unique.filter((c) => c.implementationKind === 'CATALOG_ONLY').length,
    commands: counts(unique),
    software: counts(inventory.software),
    commandVerification: ratio(real),
    softwareVerification: ratio(realSoftware),
    pinnedSoftware: realSoftware.filter((s) => s.referenceVersion !== null).length,
    unpinnedSoftware: realSoftware.filter((s) => s.referenceVersion === null).length,
    commandsTested: new Set(inventory.commands.filter((c) => c.evidenceIds.length).map((c) => c.id))
      .size,
    gatesPassed: inventory.commands
      .flatMap((c) => Object.values(c.gates))
      .filter((g) => g.result === 'PASS').length,
    requiredGates: {
      passed: gates.filter((g) => g.applicability === 'REQUIRED' && g.result === 'PASS').length,
      total: gates.filter((g) => g.applicability === 'REQUIRED').length,
    },
    pilotCases: Object.fromEntries(
      ['PASS', 'FAIL', 'SKIPPED'].map((s) => [s, coverage.filter((c) => c.result === s).length]),
    ),
    flagCasesPassed: coverage.filter((c) => c.result === 'PASS' && c.flags.length).length,
    errorCasesPassed: coverage.filter((c) => c.result === 'PASS' && c.errorCase).length,
  };
}
export function queue(inventory) {
  const pending = inventory.software
    .filter(
      (s) =>
        s.effectiveStatus !== 'VERIFIED' &&
        s.executables.length &&
        s.classification === 'REAL_COMPAT',
    )
    .map((s) => ({
      ...s,
      needsVersion: !s.referenceVersion,
      needsReference: !s.referenceSources.length,
      queueState: s.blockedBy.length
        ? 'NEEDS_SUBSYSTEM'
        : !s.referenceVersion
          ? 'NEEDS_VERSION_PIN'
          : !s.referenceSources.length
            ? 'NEEDS_REFERENCE_DATA'
            : 'READY_TO_IMPLEMENT',
    }));
  pending.sort(
    (a, b) =>
      a.wave.localeCompare(b.wave) ||
      a.priority.localeCompare(b.priority) ||
      Number(!!a.blockedBy.length) - Number(!!b.blockedBy.length) ||
      Number(a.needsVersion) - Number(b.needsVersion) ||
      b.usedByMissions.length - a.usedByMissions.length ||
      a.id.localeCompare(b.id),
  );
  const dependencies = inventory.subsystems
    .filter((s) => s.effectiveState !== 'READY')
    .map((s) => ({
      id: s.id,
      state: s.effectiveState,
      reason: s.reason,
      blocks: pending
        .filter((p) =>
          p.blockedBy.some(
            (b) => b.startsWith(`SUBSYSTEM:${s.id}:`) || b.startsWith(`SUBSYSTEM:${s.id}.`),
          ),
        )
        .map((p) => p.id),
    }));
  return {
    schemaVersion: 2,
    orderPolicy: [
      'wave',
      'priority',
      'dependency readiness',
      'reference readiness',
      'mission use',
      'software ID',
    ],
    commands: inventory.commands.some((c) => c.coreutils)
      ? coreutilsReport(inventory)
          .commands.filter((c) => c.after !== 'VERIFIED')
          .map((c) => ({
            softwareId: 'coreutils',
            command: c.command,
            wave: c.wave,
            action: c.action,
            blockers: c.blockers,
          }))
      : [],
    next: pending[0]
      ? { softwareId: pending[0].id, action: pending[0].queueState, blockers: pending[0].blockedBy }
      : null,
    software: pending,
    subsystems: dependencies,
  };
}

export function debtSnapshot(report) {
  return {
    schemaVersion: 2,
    software: report.software.map((s) => ({
      id: s.id,
      effectiveStatus: s.effectiveStatus,
      referenceVersion: s.referenceVersion,
    })),
    commands: report.commands.map((c) => ({
      id: c.id,
      effectiveStatus: c.effectiveStatus,
      implementationKind: c.implementationKind,
      referenceVersion: c.referenceVersion,
      gates: Object.fromEntries(
        Object.entries(c.gates).map(([name, g]) => [
          name,
          { applicability: g.applicability, result: g.result, testIds: g.testIds },
        ]),
      ),
    })),
    subsystems: report.subsystems
      .flatMap((s) => [s, ...(s.capabilities ?? [])])
      .map((s) => ({ id: s.id, effectiveState: s.effectiveState })),
  };
}

export function regression(previous, current, overrides = [], date = new Date()) {
  previous = debtSnapshot(previous);
  current = debtSnapshot(current);
  const changes = [];
  const violations = [];
  const approved = (id) =>
    overrides.some(
      (o) =>
        o.id === id &&
        o.reason.length >= 20 &&
        o.reference &&
        Number.isFinite(Date.parse(o.expires)) &&
        Date.parse(o.expires) > date.getTime(),
    );
  const before = new Map(previous.commands.map((c) => [c.id, c]));
  const after = new Map(current.commands.map((c) => [c.id, c]));
  for (const [id, old] of before) {
    const now = after.get(id);
    const regression =
      !now ||
      (old.effectiveStatus === 'VERIFIED' && now.effectiveStatus !== 'VERIFIED') ||
      (!['CATALOG_ONLY', 'LAUNCHER'].includes(old.implementationKind) &&
        now.implementationKind === 'CATALOG_ONLY');
    if (
      !now ||
      old.effectiveStatus !== now.effectiveStatus ||
      old.referenceVersion !== now.referenceVersion ||
      old.implementationKind !== now.implementationKind ||
      JSON.stringify(old.gates) !== JSON.stringify(now.gates)
    )
      changes.push({
        id,
        before: old.effectiveStatus,
        after: now?.effectiveStatus ?? 'REMOVED',
        versionBefore: old.referenceVersion,
        versionAfter: now?.referenceVersion,
        kindBefore: old.implementationKind,
        kindAfter: now?.implementationKind,
        gatesChanged: Object.keys(old.gates).filter(
          (name) => JSON.stringify(old.gates[name]) !== JSON.stringify(now?.gates[name]),
        ),
      });
    if (regression && !approved(id)) violations.push(`Unapproved regression ${id}`);
  }
  for (const [id] of after) if (!before.has(id)) changes.push({ id, after: 'ADDED' });
  const families = new Map(current.software.map((s) => [s.id, s]));
  for (const old of previous.software) {
    const now = families.get(old.id),
      id = `software:${old.id}`;
    if (
      !now ||
      old.effectiveStatus !== now.effectiveStatus ||
      old.referenceVersion !== now.referenceVersion
    )
      changes.push({
        id,
        before: old.effectiveStatus,
        after: now?.effectiveStatus ?? 'REMOVED',
        versionBefore: old.referenceVersion,
        versionAfter: now?.referenceVersion,
      });
    if (
      (!now || (old.effectiveStatus === 'VERIFIED' && now.effectiveStatus !== 'VERIFIED')) &&
      !approved(id)
    )
      violations.push(`Unapproved regression ${id}`);
  }
  for (const family of current.software)
    if (!previous.software.some((s) => s.id === family.id))
      changes.push({ id: `software:${family.id}`, after: 'ADDED' });
  for (const old of previous.subsystems ?? []) {
    const now = current.subsystems.find((s) => s.id === old.id);
    if (now?.effectiveState !== old.effectiveState)
      changes.push({
        id: `subsystem:${old.id}`,
        before: old.effectiveState,
        after: now?.effectiveState ?? 'REMOVED',
      });
  }
  return { changes, violations };
}
