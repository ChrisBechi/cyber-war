// DEV only. Adoption validates the exact request and two observed GNU runs.
import { wcRequests } from './cli/wc-cases.mjs';
import { capturedExpectations } from './cli/common-contracts.mjs';
import { caseSchema } from './cli/schema.mjs';
import { json, write } from './cli/io.mjs';

const args = process.argv.slice(2);
const value = (flag) => args[args.indexOf(flag) + 1];
const requests = wcRequests();
for (const test of requests)
  caseSchema.omit({ expected: true, reference: true, gates: true }).parse(test);
if (args.includes('--requests')) {
  write(value('--requests'), requests);
} else if (args.includes('--capture') && args.includes('--output')) {
  const capture = json(value('--capture'));
  const flags = [
    '-c',
    '-m',
    '-l',
    '-w',
    '-L',
    '--bytes',
    '--chars',
    '--lines',
    '--words',
    '--max-line-length',
    '--files0-from',
    '--total',
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
    if (test.argv.some((a) => /^-[lwmcL]{2,}$/.test(a))) gates.push('COMBINED_FLAGS');
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
        source: 'tests/cli/gnu/coreutils/9.7/wc.json',
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
    const spec = config.commands.wc;
    spec.implementation = 'src-tauri/src/coreutils/wc.rs';
    spec.flags = flags;
    spec.knownGaps = [
      'Canonical GNU capture, independent verification and differential closure pending.',
    ];
    spec.intentionalDeviations = [
      'Certified locale C only: byte characters, C whitespace and printable columns. Existing UTF-8 character/word profile is not GNU-certified; display width is the C profile. Virtual UTF-8 paths, bounded resources; no host execution.',
    ];
    const contracts = {
      invocation: (c) => /operands|invocation/.test(c.id),
      options: (c) => /options|total-/.test(c.id),
      errors: (c) => c.errorCase,
      'help-version': (c) => c.gates.includes('HELP') || c.gates.includes('VERSION'),
      integration: (c) => c.gates.includes('CROSS_TOOL_CONSISTENCY'),
      bytes: (c) => /binary|class-byte/.test(c.id),
      streaming: (c) => /tty-eof|pipe-/.test(c.id),
      counting: (c) => /count-|class-byte/.test(c.id),
      'word-classification': (c) => /class-byte|unicode|whitespace|binary/.test(c.id),
      'display-width': (c) => /width-tab|controls|unicode|binary/.test(c.id),
      formatting: (c) => /format-digits|operands/.test(c.id),
      totals: (c) => /total-/.test(c.id),
      'files0-from': (c) => /files0-/.test(c.id),
      signals: (c) => c.gates.includes('SIGNALS'),
      tty: (c) => c.gates.includes('TTY'),
    };
    spec.contracts = Object.entries(contracts).map(([id, match]) => ({
      id,
      applicability: 'REQUIRED',
      description: `GNU 9.7 wc ${id} observable behavior`,
      evidence: generated.filter(match).map((c) => c.id),
    }));
    write('content/cli-compatibility/coreutils.json', config);
    const manifest = json('content/cli-compatibility/manifest.json');
    const native = manifest.native.wc;
    native.implementation = spec.implementation;
    native.resultContract = 'STRUCTURED';
    native.capabilities = [...new Set([...native.capabilities, 'TTY', 'SIGNALS'])];
    native.runtimeRequirements = { SIGNALS: ['SIGNALS.STREAMS'], TTY: ['TTY.CANONICAL_IO'] };
    native.gates = Object.fromEntries(
      [...new Set(generated.flatMap((c) => c.gates))].sort().map((gate) => [
        gate,
        {
          applicability: 'REQUIRED',
          reason: 'GNU wc observable contract; canonical evidence required.',
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
          ? data.filter((c) => c.command !== 'wc')
          : { ...data, caseIds: data.caseIds.filter((id) => !id.startsWith('coreutils/wc/')) },
      );
    }
    write('tests/cli/references/coreutils/9.7/wc-manifest.json', {
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
        capture.cases.find((c) => c.id === `coreutils/wc/common-${kind}`).stdoutHex,
        'hex',
      ).toString();
      write(
        `src-tauri/src/coreutils/messages/wc-${kind}.txt`,
        bytes.replaceAll('/usr/bin/wc', '{invocation}').replaceAll('9.7', '{version}'),
      );
    }
  }
  console.log(`${generated.length} GNU observations adopted; status remains pipeline-derived.`);
} else throw new Error('Use --requests PATH or --capture PATH --output PATH [--link]');
