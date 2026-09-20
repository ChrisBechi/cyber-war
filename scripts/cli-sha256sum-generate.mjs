// DEV only. Adoption validates the exact request and two observed GNU runs.
import { sha256sumRequests } from './cli/sha256sum-cases.mjs';
import { capturedExpectations } from './cli/common-contracts.mjs';
import { caseSchema } from './cli/schema.mjs';
import { json, write } from './cli/io.mjs';

const args = process.argv.slice(2);
const value = (flag) => args[args.indexOf(flag) + 1];
const requests = sha256sumRequests();
for (const test of requests)
  caseSchema.omit({ expected: true, reference: true, gates: true }).parse(test);
if (args.includes('--requests')) {
  write(value('--requests'), requests);
} else if (args.includes('--capture') && args.includes('--output')) {
  const capture = json(value('--capture'));
  const flags = [
    '-b',
    '-t',
    '-c',
    '-w',
    '-z',
    '--binary',
    '--text',
    '--check',
    '--warn',
    '--zero',
    '--tag',
    '--strict',
    '--quiet',
    '--status',
    '--ignore-missing',
    '--help',
    '--version',
  ];
  const generated = requests.map((test) => {
    const expected = capturedExpectations(test, capture);
    expected.state.push(
      ...['cwd', 'user', 'processes'].map((path) => ({ path: '/' + path, unchanged: true })),
    );
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
    if (test.argv.some((a) => /^-[bctwz]{2,}$/.test(a))) gates.push('COMBINED_FLAGS');
    if (test.argv.includes('--help')) gates.push('HELP');
    if (test.argv.includes('--version')) gates.push('VERSION');
    if (expected.exitCode !== 0) gates.push('ERRORS');
    if (Object.keys(test.fixture.modes ?? {}).length) gates.push('PERMISSIONS');
    if (test.transport === 'pipe' || test.io?.closedConsumer) gates.push('PIPE');
    if (test.io && !test.io.closedConsumer) gates.push('REDIRECTION');
    if (test.tty.isTTY) gates.push('TTY');
    if (test.io?.closedConsumer || test.interaction?.steps.some((s) => s.kind === 'signal'))
      gates.push('SIGNALS');
    const row = capture.cases.find((row) => row.id === test.id);
    if (JSON.stringify(row.before) !== JSON.stringify(row.after)) gates.push('SIDE_EFFECTS');
    if (
      test.io ||
      test.transport === 'pipe' ||
      Object.keys(test.fixture.hardlinks ?? {}).length ||
      Object.keys(test.fixture.symlinks ?? {}).length
    )
      gates.push('CROSS_TOOL_CONSISTENCY');
    return caseSchema.parse({
      ...test,
      reference: {
        softwareId: 'coreutils',
        version: capture.version,
        kind: 'REFERENCE_ENVIRONMENT',
        source: 'tests/cli/gnu/coreutils/9.7/sha256sum.json',
      },
      gates: [...new Set(gates)],
      flags: flags.filter((f) =>
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
    const config = json('content/cli-compatibility/coreutils.json');
    const spec = config.commands.sha256sum;
    spec.implementation = 'src-tauri/src/coreutils/sha256sum.rs';
    spec.flags = flags;
    spec.knownGaps = [
      'Canonical GNU capture, independent verification and differential closure pending.',
    ];
    spec.intentionalDeviations = [
      'Certified locked C locale and UTF-8 VFS paths. Bounded checksum records (8320 bytes; twice the VFS path limit plus header); no host execution.',
    ];
    const contracts = {
      invocation: (c) => /multiple-files|invocation/.test(c.id),
      options: (c) => /options|common-double-dash/.test(c.id),
      errors: (c) => c.errorCase,
      'help-version': (c) => c.gates.includes('HELP') || c.gates.includes('VERSION'),
      integration: (c) => c.gates.includes('CROSS_TOOL_CONSISTENCY'),
      bytes: (c) => /binary|hash-basic-nul/.test(c.id),
      streaming: (c) => /pipe-|tty-eof/.test(c.id),
      hashing: (c) => /hash-basic|binary/.test(c.id),
      'checksum-output': (c) => /escaping-seed|multiple-files/.test(c.id),
      'filename-escaping': (c) => /escaping-seed|check-valid-roundtrip/.test(c.id),
      'zero-output': (c) => /escaping-seed-[34]-/.test(c.id),
      'check-parser': (c) => /check-valid|check-malformed/.test(c.id),
      'check-results': (c) => /check-results|filesystem/.test(c.id),
      'malformed-input': (c) => /check-malformed/.test(c.id),
      'warn-strict': (c) => /check-malformed|options-check-order/.test(c.id),
      'stdin-check': (c) => /stdin-check|check-multiple/.test(c.id),
      signals: (c) => c.gates.includes('SIGNALS'),
      tty: (c) => c.gates.includes('TTY'),
    };
    spec.contracts = Object.entries(contracts).map(([id, match]) => ({
      id,
      applicability: 'REQUIRED',
      description: `GNU 9.7 sha256sum ${id} observable behavior`,
      evidence: generated.filter(match).map((c) => c.id),
    }));
    write('content/cli-compatibility/coreutils.json', config);
    const manifest = json('content/cli-compatibility/manifest.json');
    const native = manifest.native.sha256sum;
    native.implementation = spec.implementation;
    native.resultContract = 'STRUCTURED';
    native.capabilities = [...new Set([...native.capabilities, 'TTY', 'SIGNALS'])];
    native.runtimeRequirements = { SIGNALS: ['SIGNALS.STREAMS'], TTY: ['TTY.CANONICAL_IO'] };
    native.gates = Object.fromEntries(
      [...new Set(generated.flatMap((c) => c.gates))].sort().map((gate) => [
        gate,
        {
          applicability: 'REQUIRED',
          reason: 'GNU sha256sum observable contract; canonical evidence required.',
          testIds: generated.filter((c) => c.gates.includes(gate)).map((c) => c.id),
        },
      ]),
    );
    write('content/cli-compatibility/manifest.json', manifest);
    for (const path of [
      'tests/cli/compat/pilot/coreutils-foundation.json',
      'tests/cli/references/coreutils/9.7/foundation-manifest.json',
    ]) {
      const data = json(path);
      write(
        path,
        Array.isArray(data)
          ? data.filter((c) => c.command !== 'sha256sum')
          : {
              ...data,
              caseIds: data.caseIds.filter((id) => !id.startsWith('coreutils/sha256sum/')),
            },
      );
    }
    write('tests/cli/references/coreutils/9.7/sha256sum-manifest.json', {
      schemaVersion: 2,
      softwareId: 'coreutils',
      version: capture.version,
      provenance: 'DECLARED_EXPECTATIONS',
      environmentDigest: null,
      caseIds: generated.map((c) => c.id).sort(),
      notes: `Expectations adopted from ${capture.provenance}; certification requires fresh GNU_REFERENCE and independent verification.`,
    });
    for (const kind of ['help', 'version']) {
      const bytes = Buffer.from(
        capture.cases.find((c) => c.id === `coreutils/sha256sum/common-${kind}`).stdoutHex,
        'hex',
      ).toString();
      write(
        `src-tauri/src/coreutils/messages/sha256sum-${kind}.txt`,
        bytes.replaceAll('/usr/bin/sha256sum', '{invocation}').replaceAll('9.7', '{version}'),
      );
    }
  }
  console.log(`${generated.length} GNU observations adopted; status remains pipeline-derived.`);
} else throw new Error('Use --requests PATH or --capture PATH --output PATH [--link]');
