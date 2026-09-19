// DEV-only: emit requests, or adopt expectations from an independently captured
// GNU run. This never edits runtime source or promotes a compatibility status.
import { json, write } from './cli/io.mjs';
import { tailRequests } from './cli/tail-cases.mjs';
import { tailFollowRequests } from './cli/tail-follow-cases.mjs';
import { tailDirectedRequests } from './cli/tail-directed-cases.mjs';
import { capturedExpectations } from './cli/common-contracts.mjs';
import { caseSchema } from './cli/schema.mjs';

const args = process.argv.slice(2);
const cases = [...tailRequests(), ...tailFollowRequests(), ...tailDirectedRequests()];
const requestSchema = caseSchema.omit({ expected: true, reference: true, gates: true });
for (const test of cases) requestSchema.parse(test);
const value = (flag) => args[args.indexOf(flag) + 1];
if (args.includes('--prepare')) {
  const contracts = json('content/cli-compatibility/coreutils.json');
  const tail = contracts.commands.tail;
  tail.implementation = 'src-tauri/src/coreutils/tail.rs';
  tail.flags = [
    '-q',
    '-v',
    '-n',
    '-c',
    '-z',
    '-f',
    '-F',
    '-s',
    '--quiet',
    '--silent',
    '--verbose',
    '--lines',
    '--bytes',
    '--zero-terminated',
    '--follow',
    '--retry',
    '--sleep-interval',
    '--max-unchanged-stats',
    '--pid',
  ];
  tail.knownGaps = [
    'M1C.4 implementation awaits fresh canonical GNU capture, current project comparison and complete regression validation.',
  ];
  tail.subsystems = ['VFS.EVENTS', 'VFS.WATCH'];
  tail.intentionalDeviations = [
    'World-local virtual processes/files/devices; bounded 4 MiB suffix/output capture and 32 MiB byte files. C locale oracle. Discrete virtual time advances only at scheduler idle/event boundaries. General job control and additional device types remain outside the subset.',
  ];
  for (const [id, description] of Object.entries({
    invocation: 'Finite byte/line suffixes and from-start selection, stdin and ordered operands',
    options:
      'GNU short/long/combined/abbreviated options, count multipliers, overflow, historical syntax and NUL records',
    'help-version': 'Exact GNU 9.7 C-locale help and full version banner',
    headers: 'Multi-file and quiet/verbose headers with exact operand order and path quoting',
    tty: 'Canonical TTY EOF, incremental from-start output and follow after EOF',
    signals:
      'Process-local SIGINT, SIGTERM, SIGPIPE and cleanup at virtual wait/read/write boundaries',
    follow:
      'Descriptor/name follow, append, truncate, rotation, replacement, retry, permissions and links',
    scheduler:
      'Revision-based suspension, filtered wakeup, virtual intervals and process lifetime observation',
  })) {
    let contract = tail.contracts.find((c) => c.id === id);
    if (!contract) {
      contract = { id, applicability: 'REQUIRED', description, evidence: [] };
      tail.contracts.push(contract);
    }
    contract.description = description;
  }
  write('content/cli-compatibility/coreutils.json', contracts);
  const manifest = json('content/cli-compatibility/manifest.json');
  manifest.native.tail.implementation = tail.implementation;
  manifest.native.tail.vfsRequirements = [
    ...new Set([
      ...(manifest.native.head.vfsRequirements ?? manifest.software.coreutils.vfsRequirements),
      'VFS.EVENTS',
      'VFS.WATCH',
    ]),
  ];
  manifest.native.tail.capabilities = [
    ...new Set([...manifest.native.tail.capabilities, 'SIGNALS', 'TTY', 'PROCESS_READ']),
  ];
  write('content/cli-compatibility/manifest.json', manifest);
  const subsystems = json('content/cli-compatibility/subsystems.json');
  const vfs = subsystems.subsystems.find((s) => s.id === 'VFS');
  for (const id of ['VFS.EVENTS', 'VFS.WATCH']) {
    if (!vfs.capabilities.some((c) => c.id === id))
      vfs.capabilities.push({
        id,
        state: 'PARTIAL',
        reason:
          'Shared event/watch implementation exists; fresh canonical and global runtime evidence is pending.',
        requiredTests: [],
        evidenceIds: ['vfs::event_tests', 'service::cooperative_tests'],
        limitations: [
          'Subscriptions are runtime-only, capped at 1024 per VFS and canceled on session reset/load.',
        ],
      });
  }
  vfs.implementation = [...new Set([...vfs.implementation, 'src-tauri/src/vfs/events.rs'])];
  write('content/cli-compatibility/subsystems.json', subsystems);
} else if (args.includes('--requests')) {
  write(value('--requests'), cases);
} else if (args.includes('--capture') && args.includes('--output')) {
  const captures = args.flatMap((arg, index) =>
    arg === '--capture' ? [json(args[index + 1])] : [],
  );
  const declared = json('content/cli-compatibility/coreutils.json').commands.tail.flags;
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
    if (test.interaction?.steps.some((s) => ['mutate', 'mutateBatch'].includes(s.kind)))
      gates.push('SIDE_EFFECTS', 'CROSS_TOOL_CONSISTENCY');
    if (test.tty.isTTY) gates.push('TTY');
    if (
      test.interaction?.steps.some(
        (s) => s.kind === 'signal' || (s.kind === 'stopWriter' && s.signal),
      ) ||
      test.io?.closedConsumer
    )
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
        source: 'tests/cli/gnu/coreutils/9.7/tail.json',
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
    manifest.native.tail.vfsRequirements = [
      ...new Set([
        ...manifest.software.coreutils.vfsRequirements,
        ...(manifest.native.tail.vfsRequirements ?? []),
        'VFS.EVENTS',
        'VFS.WATCH',
      ]),
    ];
    manifest.native.tail.runtimeRequirements = {
      PROCESS: ['PROCESS.LIFETIME'],
      SIGNALS: ['SIGNALS.STREAMS'],
      TTY: ['TTY.CANONICAL_IO'],
    };
    manifest.native.tail.gates = Object.fromEntries(
      [...new Set(generated.flatMap((c) => c.gates))].sort().map((gate) => [
        gate,
        {
          applicability: 'REQUIRED',
          reason:
            'Current tail cases assert this contract; independent GNU 9.7 bytes are required separately.',
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
      headers: (c) => /headers|file-|follow-multiple/.test(c.id),
      tty: (c) => c.gates.includes('TTY'),
      signals: (c) => c.gates.includes('SIGNALS'),
      follow: (c) => c.interaction?.schemaVersion === 2,
      scheduler: (c) => c.interaction?.schemaVersion === 2,
    };
    for (const contract of contracts.commands.tail.contracts) {
      if (!match[contract.id]) throw new Error(`Unknown tail contract ${contract.id}`);
      contract.evidence = generated.filter(match[contract.id]).map((c) => c.id);
    }
    write(contractPath, contracts);
    const subsystemPath = 'content/cli-compatibility/subsystems.json';
    const subsystems = json(subsystemPath);
    for (const [id, matches, limitation] of [
      [
        'PROCESS.LIFETIME',
        (c) => c.interaction?.writers?.length > 0,
        'Only world-local writer lifetime observation; general process lifecycle and job control remain partial.',
      ],
      [
        'SIGNALS.STREAMS',
        (c) => c.gates.includes('SIGNALS'),
        'Only process-local stream INT/TERM/PIPE handling and cleanup; general signals and process groups remain partial.',
      ],
      [
        'TTY.CANONICAL_IO',
        (c) => c.gates.includes('TTY'),
        'Only canonical data/EOF events in virtual streams; raw terminal discipline remains partial.',
      ],
    ]) {
      const subsystem = subsystems.subsystems.find((s) => s.id === id.split('.')[0]);
      subsystem.capabilities ??= [];
      let capability = subsystem.capabilities.find((c) => c.id === id);
      if (!capability) {
        capability = {
          id,
          state: 'PARTIAL',
          evidenceIds: ['service::cooperative_tests', 'coreutils::tail_tests'],
          limitations: [limitation],
        };
        subsystem.capabilities.push(capability);
      }
      capability.readiness = 'GNU_DIFFERENTIAL';
      capability.reason =
        'Current canonical stream evidence derives readiness for this bounded capability; the general subsystem remains partial.';
      capability.requiredTests = generated.filter(matches).map((c) => c.id);
    }
    for (const capability of subsystems.subsystems
      .find((s) => s.id === 'VFS')
      .capabilities.filter((c) => ['VFS.EVENTS', 'VFS.WATCH'].includes(c.id))) {
      capability.readiness = 'GNU_DIFFERENTIAL';
      capability.reason =
        'Readiness requires current GNU differential follow evidence, linked contracts, no required gaps and Host Guard; declarations alone cannot promote it.';
      capability.requiredTests = generated
        .filter((c) => c.interaction?.schemaVersion === 2)
        .map((c) => c.id);
    }
    write(subsystemPath, subsystems);
    write('tests/cli/references/coreutils/9.7/tail-manifest.json', {
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
  throw new Error('Use --prepare, --requests PATH, or --capture GNU_CAPTURE --output CASES');
}
