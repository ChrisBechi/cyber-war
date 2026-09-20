// DEV adoption: separate command goldens/contracts over a shared path matrix.
import { filesystemRequests, filesystemCommands } from './cli/filesystem-cases.mjs';
import { capturedExpectations } from './cli/common-contracts.mjs';
import { caseSchema } from './cli/schema.mjs';
import { json, write } from './cli/io.mjs';
const args = process.argv.slice(2),
  value = (flag) => args[args.indexOf(flag) + 1];
const command = value('--command');
if (!filesystemCommands.includes(command)) throw new Error('Select a filesystem wave command');
const requests = filesystemRequests(command);
for (const t of requests)
  caseSchema.omit({ expected: true, reference: true, gates: true }).parse(t);
if (args.includes('--requests')) write(value('--requests'), requests);
else if (args.includes('--capture') && args.includes('--output')) {
  const capture = json(value('--capture'));
  const flags = {
    ln: [
      '-b',
      '-d',
      '-F',
      '-f',
      '-i',
      '-L',
      '-n',
      '-P',
      '-r',
      '-s',
      '-S',
      '-t',
      '-T',
      '-v',
      '--backup',
      '--directory',
      '--no-dereference',
      '--no-target-directory',
      '--force',
      '--interactive',
      '--suffix',
      '--target-directory',
      '--logical',
      '--physical',
      '--relative',
      '--symbolic',
      '--verbose',
    ],
    rmdir: ['-p', '-v', '--parents', '--path', '--verbose', '--ignore-fail-on-non-empty'],
    mkdir: ['-p', '-m', '-v', '-Z', '--parents', '--mode', '--verbose', '--context'],
    realpath: [
      '-e',
      '-m',
      '-L',
      '-P',
      '-q',
      '-s',
      '-z',
      '--canonicalize-existing',
      '--canonicalize-missing',
      '--relative-to',
      '--relative-base',
      '--quiet',
      '--strip',
      '--no-symlinks',
      '--zero',
      '--logical',
      '--physical',
    ],
    readlink: [
      '-e',
      '-f',
      '-m',
      '-n',
      '-q',
      '-s',
      '-v',
      '-z',
      '--canonicalize',
      '--canonicalize-existing',
      '--canonicalize-missing',
      '--no-newline',
      '--quiet',
      '--silent',
      '--verbose',
      '--zero',
    ],
  }[command].concat(['--help', '--version']);
  const generated = requests.map((test) => {
    const expected = capturedExpectations(test, capture);
    expected.state.push(
      ...['cwd', 'user', 'processes'].map((p) => ({ path: '/' + p, unchanged: true })),
    );
    const gates = [
      'STDOUT',
      'STDERR',
      'EXIT_CODE',
      'PARSER',
      'POSITIONAL_ARGS',
      'STATE_CONSISTENCY',
    ];
    if (test.argv.some((a) => /^-[^-]/.test(a))) gates.push('SHORT_FLAGS');
    if (test.argv.some((a) => a.startsWith('--'))) gates.push('LONG_FLAGS');
    if (test.argv.some((a) => /^-[a-zA-Z]{2,}$/.test(a))) gates.push('COMBINED_FLAGS');
    if (test.argv.includes('--help')) gates.push('HELP');
    if (test.argv.includes('--version')) gates.push('VERSION');
    if (expected.exitCode !== 0) gates.push('ERRORS');
    if (Object.keys(test.fixture.modes ?? {}).length) gates.push('PERMISSIONS');
    if (test.transport === 'pipe') gates.push('PIPE');
    if (test.io) gates.push('REDIRECTION');
    if (test.tty.isTTY) gates.push('TTY');
    if (command === 'ln' && /interactive|tty|stdin-file|integration-pipe/.test(test.id))
      gates.push('STDIN');
    if (
      test.interaction?.steps.some((s) => s.kind === 'signal') ||
      expected.termination?.kind === 'signal'
    )
      gates.push('SIGNALS');
    if (
      test.io ||
      test.transport === 'pipe' ||
      Object.keys(test.fixture.symlinks ?? {}).length ||
      Object.keys(test.fixture.hardlinks ?? {}).length
    )
      gates.push('CROSS_TOOL_CONSISTENCY');
    const row = capture.cases.find((c) => c.id === test.id);
    if (JSON.stringify(row.before) !== JSON.stringify(row.after)) gates.push('SIDE_EFFECTS');
    return caseSchema.parse({
      ...test,
      reference: {
        softwareId: 'coreutils',
        version: capture.version,
        kind: 'REFERENCE_ENVIRONMENT',
        source: `tests/cli/gnu/coreutils/9.7/${command}.json`,
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
    const config = json('content/cli-compatibility/coreutils.json'),
      spec = config.commands[command];
    spec.implementation =
      'src-tauri/src/coreutils/' +
      (['mkdir', 'rmdir'].includes(command)
        ? 'directories.rs'
        : command === 'ln'
          ? 'links.rs'
          : 'pathnames.rs');
    spec.flags = flags;
    spec.knownGaps = [
      'Canonical GNU capture, independent verification and differential closure pending.',
    ];
    spec.intentionalDeviations = [
      'Locked C locale and UTF-8 VFS names. Virtual paths are bounded to 4096 bytes; stored filesystem components to 255 bytes. Canonical missing-mode may describe nonexistent longer components. Traversal has a 10000-component and 64 KiB pending-expansion safety budget; no host execution.',
    ];
    if (['readlink', 'realpath'].includes(command))
      spec.intentionalDeviations.push(
        'Growing-target symlink cycles (e.g. grow -> grow/child) exceed GNU reference resource limits (SIGKILL/timeout); gameplay rejects cycles with bounded traversal. Canonical GNU evidence excludes these nonterminating resource-bound probes.',
      );
    const contracts = {
      invocation: (c) => /invocation|multiple-operands/.test(c.id),
      options: (c) => /options|unknown|missing-operands/.test(c.id),
      errors: (c) => c.errorCase,
      'help-version': (c) => c.gates.includes('HELP') || c.gates.includes('VERSION'),
      integration: (c) => /integration/.test(c.id),
      'raw-target': (c) => /paths-0-|special-filenames/.test(c.id),
      canonicalization: (c) => /paths-[234]-/.test(c.id),
      'missing-policy': (c) => /paths-[234]-(12|13|25|26|27|28)$/.test(c.id),
      'link-resolution': (c) =>
        /paths-[234]-(6|7|8|9|10|14|15|16|17|18|19)$|symlink-chain/.test(c.id),
      'output-delimiters': (c) => /multiple|options-single|permutation/.test(c.id),
      diagnostics: (c) => /special-errors|unknown|component-limit/.test(c.id),
      state: (c) => c.gates.includes('CROSS_TOOL_CONSISTENCY'),
    };
    if (command === 'realpath') {
      delete contracts['raw-target'];
      contracts.canonicalization = (c) => /paths-/.test(c.id);
      contracts['missing-policy'] = (c) => /paths-[012]-(12|13|25|26|27|28)$/.test(c.id);
      contracts['lexical-normalization'] = (c) => /paths-[35]-/.test(c.id);
      contracts['logical-physical'] = (c) => /paths-[04]-|options-multiple/.test(c.id);
      contracts['relative-output'] = (c) => /relative-/.test(c.id);
    }
    if (command === 'mkdir') {
      for (const key of [
        'raw-target',
        'canonicalization',
        'missing-policy',
        'link-resolution',
        'output-delimiters',
      ])
        delete contracts[key];
      Object.assign(contracts, {
        creation: (c) => /paths-0-|special-filenames/.test(c.id),
        parents: (c) => /paths-[12]-|parents-/.test(c.id),
        modes: (c) => /mode|setgid/.test(c.id),
        umask: (c) => /modes-|umask/.test(c.id),
        'partial-effects': (c) => /paths-[12]-(4[0-9]|26|27)|options-(9|10)$/.test(c.id),
        'link-resolution': (c) =>
          /paths-[012]-(5|6|7|8|9|10|11|12|13|14|15|16|17|18|19)$/.test(c.id),
        permissions: (c) => /permissions|paths-[012]-(31|32)$/.test(c.id),
        verbose: (c) => /paths-2-|special-filenames/.test(c.id),
        context: (c) => c.argv.some((a) => a.startsWith('--context') || a === '-Z'),
      });
    }
    if (command === 'rmdir') {
      for (const key of [
        'raw-target',
        'canonicalization',
        'missing-policy',
        'link-resolution',
        'output-delimiters',
      ])
        delete contracts[key];
      Object.assign(contracts, {
        removal: (c) => /paths-0-|special-filenames/.test(c.id),
        parents: (c) => /paths-[13]-/.test(c.id),
        'ignore-nonempty': (c) => /paths-[23]-|permissions-nonempty/.test(c.id),
        'partial-effects': (c) => /paths-[13]-|options-[01]$/.test(c.id),
        'link-resolution': (c) => /paths-[0123]-(5|6|7|8|9|1[0-9]|4[0-4]|58|59|60)$/.test(c.id),
        permissions: (c) => /permissions/.test(c.id),
        verbose: (c) => /paths-[13]-|special-/.test(c.id),
      });
    }
    if (command === 'ln') {
      for (const key of [
        'raw-target',
        'canonicalization',
        'missing-policy',
        'link-resolution',
        'output-delimiters',
      ])
        delete contracts[key];
      Object.assign(contracts, {
        hardlinks: (c) => /sources-[02]-|same-file-|collision-/.test(c.id),
        symlinks: (c) => /sources-[13]-|component-target|target-byte-limit/.test(c.id),
        destinations: (c) => /destinations-|same-file-|options-(1[4-9]|2[0123])$/.test(c.id),
        relative: (c) => /relative-|sources-3-/.test(c.id),
        backup: (c) => /backup|collision-/.test(c.id),
        interactive: (c) => /interactive-|tty-/.test(c.id),
        signals: (c) => c.gates.includes('SIGNALS'),
        permissions: (c) => /permissions-|directory-root/.test(c.id),
        'partial-effects': (c) => /ordering-regression|collision-|options-[56789]$/.test(c.id),
        'same-file': (c) => /same-file-|interactive-order/.test(c.id),
      });
    }
    spec.contracts = Object.entries(contracts).map(([id, match]) => ({
      id,
      applicability: 'REQUIRED',
      description: `GNU 9.7 ${command} ${id} observable behavior`,
      evidence: generated.filter(match).map((c) => c.id),
    }));
    write('content/cli-compatibility/coreutils.json', config);
    const manifest = json('content/cli-compatibility/manifest.json'),
      native = manifest.native[command];
    native.implementation = spec.implementation;
    native.resultContract = 'STRUCTURED';
    if (['readlink', 'realpath'].includes(command))
      native.capabilities = native.capabilities.filter((c) => !['STDIN', 'VFS_WRITE'].includes(c));
    if (command === 'ln') {
      native.implementation = 'src-tauri/src/coreutils/links.rs';
      spec.implementation = native.implementation;
      native.capabilities = [...new Set([...native.capabilities, 'TTY', 'SIGNALS'])];
      native.runtimeRequirements = { SIGNALS: ['SIGNALS.STREAMS'], TTY: ['TTY.CANONICAL_IO'] };
    }
    native.gates = Object.fromEntries(
      [...new Set(generated.flatMap((c) => c.gates))].sort().map((g) => [
        g,
        {
          applicability: 'REQUIRED',
          reason: `GNU ${command} observable contract; canonical evidence required.`,
          testIds: generated.filter((c) => c.gates.includes(g)).map((c) => c.id),
        },
      ]),
    );
    if (command !== 'ln')
      for (const gate of ['STDIN', 'TTY', 'SIGNALS'])
        native.gates[gate] = {
          applicability: 'NOT_APPLICABLE',
          reason:
            'Finite pathname operation with no stdin, terminal-interaction or asynchronous signal contract; shared shell behavior is tested independently.',
          testIds: [],
        };
    write('content/cli-compatibility/manifest.json', manifest);
    for (const path of [
      'tests/cli/compat/pilot/coreutils-foundation.json',
      'tests/cli/references/coreutils/9.7/foundation-manifest.json',
    ]) {
      const data = json(path);
      write(
        path,
        Array.isArray(data)
          ? data.filter((c) => c.command !== command)
          : {
              ...data,
              caseIds: data.caseIds.filter((id) => !id.startsWith(`coreutils/${command}/`)),
            },
      );
    }
    write(`tests/cli/references/coreutils/9.7/${command}-manifest.json`, {
      schemaVersion: 2,
      softwareId: 'coreutils',
      version: capture.version,
      provenance: 'DECLARED_EXPECTATIONS',
      environmentDigest: null,
      caseIds: generated.map((c) => c.id).sort(),
      notes: `Expectations adopted from ${capture.provenance}; certification requires fresh GNU_REFERENCE and independent verification.`,
    });
    for (const kind of ['help', 'version'])
      write(
        `src-tauri/src/coreutils/messages/${command}-${kind}.txt`,
        Buffer.from(
          capture.cases.find((c) => c.id === `coreutils/${command}/common-${kind}`).stdoutHex,
          'hex',
        )
          .toString()
          .replaceAll('/usr/bin/' + command, '{invocation}')
          .replaceAll('9.7', '{version}'),
      );
    const lock = json('content/cli-compatibility/coreutils-environment.json');
    lock.binaryHashes[command] = capture.environment.binaryHashes[command].sha256;
    write('content/cli-compatibility/coreutils-environment.json', lock);
  }
  console.log(`${command}: ${generated.length} GNU observations adopted; status remains derived.`);
} else
  throw new Error(
    'Use --command NAME with --requests PATH or --capture PATH --output PATH [--link]',
  );
