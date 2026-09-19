// DEV only. Generate requests or adopt exact observations from the locked GNU oracle.
import { base64Requests } from './cli/base64-cases.mjs';
import { capturedExpectations } from './cli/common-contracts.mjs';
import { caseSchema } from './cli/schema.mjs';
import { json, write } from './cli/io.mjs';

const args = process.argv.slice(2);
const value = (flag) => args[args.indexOf(flag) + 1];
const requests = base64Requests();
for (const test of requests)
  caseSchema.omit({ expected: true, reference: true, gates: true }).parse(test);
if (args.includes('--requests')) {
  write(value('--requests'), requests);
} else if (args.includes('--capture') && args.includes('--output')) {
  const capture = json(value('--capture'));
  const selected = args.includes('--allow-partial')
    ? requests.filter((test) => capture.cases.some((row) => row.id === test.id))
    : requests;
  if (!selected.length) throw new Error('No captured requests');
  const contractPath = 'content/cli-compatibility/coreutils.json';
  const config = json(contractPath);
  const spec = config.commands.base64;
  const generated = selected.map((test) => {
    const expected = capturedExpectations(test, capture);
    const gates = [
      'STDOUT',
      'STDERR',
      'EXIT_CODE',
      'PARSER',
      'POSITIONAL_ARGS',
      'STDIN',
      'STATE_CONSISTENCY',
    ];
    if (test.argv.some((a) => /^-[^-]/.test(a))) gates.push('SHORT_FLAGS');
    if (test.argv.some((a) => a.startsWith('--'))) gates.push('LONG_FLAGS');
    if (test.argv.some((a) => /^-[diw]{2}/.test(a))) gates.push('COMBINED_FLAGS');
    if (test.argv.includes('--help')) gates.push('HELP');
    if (test.argv.includes('--version')) gates.push('VERSION');
    if (expected.exitCode !== 0) gates.push('ERRORS');
    if (Object.keys(test.fixture.modes ?? {}).length) gates.push('PERMISSIONS');
    if (test.transport === 'pipe' || test.io?.closedConsumer) gates.push('PIPE');
    if (test.io && !test.io.closedConsumer) gates.push('REDIRECTION');
    if (test.tty.isTTY) gates.push('TTY');
    if (test.io?.closedConsumer || test.interaction?.steps.some((s) => s.kind === 'signal'))
      gates.push('SIGNALS');
    if (
      test.io ||
      test.transport === 'pipe' ||
      Object.keys(test.fixture.symlinks ?? {}).length ||
      Object.keys(test.fixture.hardlinks ?? {}).length
    )
      gates.push('CROSS_TOOL_CONSISTENCY');
    expected.state.push(
      ...['cwd', 'user', 'processes'].map((path) => ({ path: '/' + path, unchanged: true })),
    );
    if (test.io?.stdoutPath || test.io?.stderrPath) {
      gates.push('SIDE_EFFECTS');
      for (const path of [test.io.stdoutPath, test.io.stderrPath].filter(Boolean))
        expected.state.push({
          path: '/vfs/' + path.replaceAll('~', '~0').replaceAll('/', '~1') + '/kind',
          matcher: { kind: 'EXACT', value: 'file' },
        });
    }
    return caseSchema.parse({
      ...test,
      reference: {
        softwareId: 'coreutils',
        version: capture.version,
        kind: 'REFERENCE_ENVIRONMENT',
        source: 'tests/cli/gnu/coreutils/9.7/base64.json',
      },
      gates: [...new Set(gates)],
      flags: spec.flags.filter((f) =>
        test.argv.some((a) =>
          f.startsWith('--')
            ? a === f || a.startsWith(f + '=')
            : a.startsWith('-') && !a.startsWith('--') && a.slice(1).includes(f.slice(1)),
        ),
      ),
      errorCase: expected.exitCode !== 0,
      expected,
    });
  });
  write(value('--output'), generated);
  if (args.includes('--link')) {
    spec.implementation = 'src-tauri/src/coreutils/base64.rs';
    if (capture.provenance !== 'GNU_REFERENCE')
      spec.knownGaps = [
        'Canonical GNU capture and independent verification pending; finite GNU_PROBE observations are not certification.',
        'TTY buffering/timing and process-local SIGINT/SIGTERM require successful GNU interaction capture and differential validation.',
      ];
    spec.intentionalDeviations = [
      'World-local virtual byte streams/files/identities; C locale, bounded virtual resources and output capture. No host execution.',
    ];
    const matches = {
      invocation: (c) => /\/encode$|\/decode$|file-|stdin|alias/.test(c.id),
      options: (c) => /options-|wrap|unknown|double-dash|posix/.test(c.id),
      errors: (c) => c.errorCase,
      'help-version': (c) => c.gates.includes('HELP') || c.gates.includes('VERSION'),
      integration: (c) => c.gates.includes('CROSS_TOOL_CONSISTENCY'),
      bytes: (c) => /binary|separator/.test(c.id),
      encoding: (c) => /encode/.test(c.id),
      wrapping: (c) => /wrap/.test(c.id),
      decode: (c) => /decode/.test(c.id),
      'ignore-garbage': (c) => /ignore|garbage/.test(c.id),
      'invalid-input': (c) => c.errorCase && (c.argv.includes('-d') || c.argv.includes('-di')),
      'partial-output': (c) =>
        c.errorCase && (c.expected.stdoutHex.length > 0 || c.id.endsWith('/partial-redirect')),
      tty: (c) => c.gates.includes('TTY'),
      signals: (c) => c.gates.includes('SIGNALS'),
    };
    for (const [id, match] of Object.entries(matches)) {
      let contract = spec.contracts.find((c) => c.id === id);
      if (!contract) {
        contract = {
          id,
          applicability: 'REQUIRED',
          description: `GNU 9.7 base64 ${id} observable behavior`,
          evidence: [],
        };
        spec.contracts.push(contract);
      }
      if (id === 'help-version')
        contract.description = 'Exact GNU 9.7 C-locale help and full version banner';
      contract.evidence = generated.filter(match).map((c) => c.id);
    }
    write(contractPath, config);
    const manifestPath = 'content/cli-compatibility/manifest.json';
    const manifest = json(manifestPath);
    const native = manifest.native.base64;
    native.implementation = spec.implementation;
    native.resultContract = 'STRUCTURED';
    native.capabilities = [...new Set([...native.capabilities, 'SIGNALS', 'TTY'])];
    native.runtimeRequirements = { SIGNALS: ['SIGNALS.STREAMS'], TTY: ['TTY.CANONICAL_IO'] };
    native.gates = Object.fromEntries(
      [...new Set([...generated.flatMap((c) => c.gates), 'TTY', 'SIGNALS'])].sort().map((gate) => [
        gate,
        {
          applicability: 'REQUIRED',
          reason: 'GNU base64 observable contract; missing canonical evidence remains a blocker.',
          testIds: generated.filter((c) => c.gates.includes(gate)).map((c) => c.id),
        },
      ]),
    );
    write(manifestPath, manifest);
    // Replace the old seven project-only declarations, preserving every other executable.
    for (const path of [
      'tests/cli/compat/pilot/coreutils-foundation.json',
      'tests/cli/references/coreutils/9.7/foundation-manifest.json',
    ]) {
      const data = json(path);
      write(
        path,
        Array.isArray(data)
          ? data.filter((c) => c.command !== 'base64')
          : { ...data, caseIds: data.caseIds.filter((id) => !id.startsWith('coreutils/base64/')) },
      );
    }
    write('tests/cli/references/coreutils/9.7/base64-manifest.json', {
      schemaVersion: 2,
      softwareId: 'coreutils',
      version: capture.version,
      provenance: 'DECLARED_EXPECTATIONS',
      environmentDigest: null,
      caseIds: generated.map((c) => c.id).sort(),
      notes: `Expectations adopted from ${capture.provenance}; canonical GNU_REFERENCE and independent verification remain required.`,
    });
  }
  console.log(
    `${generated.length}/${requests.length} requests adopted from ${capture.provenance}; certification is derived separately.`,
  );
} else {
  throw new Error(
    'Use --requests PATH or --capture GNU_CAPTURE --output CASES [--allow-partial] [--link]',
  );
}
