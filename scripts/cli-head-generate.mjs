// DEV-only: emit requests, or adopt expectations from an independently captured
// GNU run. This never edits runtime source or promotes a compatibility status.
import { json, write } from './cli/io.mjs';
import { headRequests } from './cli/head-cases.mjs';
import { capturedExpectations } from './cli/common-contracts.mjs';
import { caseSchema } from './cli/schema.mjs';

const args = process.argv.slice(2);
const cases = headRequests();
const value = (flag) => args[args.indexOf(flag) + 1];
if (args.includes('--requests')) {
  write(value('--requests'), cases);
} else if (args.includes('--capture') && args.includes('--output')) {
  const captures = args.flatMap((arg, index) =>
    arg === '--capture' ? [json(args[index + 1])] : [],
  );
  const declared = json('content/cli-compatibility/coreutils.json').commands.head.flags;
  const generated = cases.map((test) => {
    const matching = captures.filter((capture) => capture.cases.some((c) => c.id === test.id));
    if (matching.length !== 1) throw new Error(`Expected exactly one GNU oracle for ${test.id}`);
    const expected = capturedExpectations(test, matching[0]);
    const gates = ['STDOUT', 'STDERR', 'EXIT_CODE', 'PARSER', 'POSITIONAL_ARGS', 'STDIN'];
    if (test.argv.some((a) => /^-[^-]/.test(a))) gates.push('SHORT_FLAGS');
    if (test.argv.some((a) => a.startsWith('--'))) gates.push('LONG_FLAGS');
    if (test.argv.some((a) => /^-[a-zA-Z]{2}/.test(a))) gates.push('COMBINED_FLAGS');
    if (test.argv.includes('--help')) gates.push('HELP');
    if (test.argv.includes('--version')) gates.push('VERSION');
    if (expected.exitCode !== 0) gates.push('ERRORS');
    if (Object.keys(test.fixture.modes ?? {}).length) gates.push('PERMISSIONS');
    if (test.transport === 'pipe' || test.io?.closedConsumer) gates.push('PIPE');
    if (test.io && !test.io.closedConsumer) gates.push('REDIRECTION');
    if (test.interaction) gates.push('TTY');
    if (test.interaction?.steps.some((s) => s.kind === 'signal') || test.io?.closedConsumer)
      gates.push('SIGNALS');
    gates.push('STATE_CONSISTENCY');
    expected.state.push(
      ...['cwd', 'user', 'processes'].map((p) => ({ path: '/' + p, unchanged: true })),
    );
    if (
      test.io ||
      test.transport === 'pipe' ||
      Object.keys(test.fixture.hardlinks ?? {}).length ||
      Object.keys(test.fixture.symlinks ?? {}).length
    ) {
      gates.push('CROSS_TOOL_CONSISTENCY');
    }
    if (test.io?.stdoutPath || test.io?.stderrPath) {
      gates.push('SIDE_EFFECTS');
      for (const path of [test.io.stdoutPath, test.io.stderrPath].filter(Boolean))
        expected.state.push({
          path: '/vfs/' + path.replaceAll('~', '~0').replaceAll('/', '~1') + '/kind',
          matcher: { kind: 'EXACT', value: 'file' },
        });
    }
    const flags = declared.filter((f) =>
      test.argv.some((a) =>
        f.startsWith('--')
          ? a === f || a.startsWith(f + '=')
          : a.startsWith('-') && !a.startsWith('--') && a.slice(1).includes(f.slice(1)),
      ),
    );
    return caseSchema.parse({
      ...test,
      reference: {
        softwareId: 'coreutils',
        version: '9.7',
        kind: 'REFERENCE_ENVIRONMENT',
        source: 'tests/cli/gnu/coreutils/9.7/head.json',
      },
      gates: [...new Set(gates)],
      flags,
      errorCase: expected.exitCode !== 0,
      expected,
    });
  });
  write(value('--output'), generated);
  if (args.includes('--link')) {
    const manifestPath = 'content/cli-compatibility/manifest.json';
    const manifest = json(manifestPath);
    manifest.native.head.gates = Object.fromEntries(
      [...new Set(generated.flatMap((c) => c.gates))].sort().map((gate) => [
        gate,
        {
          applicability: 'REQUIRED',
          reason:
            'Current head cases assert this contract; independent GNU 9.7 bytes are required separately.',
          testIds: generated.filter((c) => c.gates.includes(gate)).map((c) => c.id),
        },
      ]),
    );
    write(manifestPath, manifest);
    const contractPath = 'content/cli-compatibility/coreutils.json';
    const contracts = json(contractPath);
    const match = {
      invocation: (c) => /file-|\/lines$|stdin-file/.test(c.id),
      options: (c) => /\/(mode-|count-|suffix|parse-|posix|legacy-|zero-terminated)/.test(c.id),
      errors: (c) => c.errorCase,
      'help-version': (c) => c.gates.includes('HELP') || c.gates.includes('VERSION'),
      integration: (c) => c.gates.includes('CROSS_TOOL_CONSISTENCY'),
      bytes: (c) => /binary|large-file|\/empty$/.test(c.id),
      headers: (c) => /header|file-/.test(c.id),
      tty: (c) => c.gates.includes('TTY'),
      signals: (c) => c.gates.includes('SIGNALS'),
    };
    for (const contract of contracts.commands.head.contracts) {
      if (!match[contract.id]) throw new Error(`Unknown head contract ${contract.id}`);
      contract.evidence = generated.filter(match[contract.id]).map((c) => c.id);
    }
    write(contractPath, contracts);
    write('tests/cli/references/coreutils/9.7/head-manifest.json', {
      schemaVersion: 2,
      softwareId: 'coreutils',
      version: '9.7',
      provenance: 'DECLARED_EXPECTATIONS',
      environmentDigest: null,
      caseIds: generated.map((c) => c.id).sort(),
      notes:
        'Declaration index only; independent GNU 9.7 reference captures are required for certification.',
    });
  }
} else {
  throw new Error('Use --requests PATH, or --capture GNU_CAPTURE --output CASES');
}
