// Request generation is independent of expectations. Only captured GNU bytes
// may become golden expectations; each command still needs its own reference.
import {
  canonical,
  requestDigest,
  referenceRequest,
  sourceHash,
  verificationFresh,
} from './coreutils-reference.mjs';

export function request(command, id, argv, extra = {}) {
  return {
    schemaVersion: 2,
    id: `coreutils/${command}/${id}`,
    softwareId: 'coreutils',
    command,
    invocation: `/usr/bin/${command}`,
    argv,
    stdin: '1\n2\n3\n4\nlast',
    cwd: '/home/kali',
    env: { LC_ALL: 'C' },
    tty: { isTTY: false, columns: 80, rows: 24, ansiSupport: true, interactive: false },
    fixture: {
      files: {},
      bytes: {},
      directories: [],
      modes: {},
      hardlinks: {},
      symlinks: {},
      setup: [],
    },
    ...extra,
  };
}

export function commonContracts(command) {
  const make = (id, argv, extra) => request(command, id, argv, extra);
  const file = { files: { '/home/kali/a': 'A\nB\nC\n' } };
  return [
    make('common-help', ['--help']),
    make('common-version', ['--version']),
    make('common-unknown-short', ['-?']),
    make('common-unknown-long', ['--no-such-option']),
    make('common-double-dash', ['--', '-a'], { fixture: { files: { '/home/kali/-a': 'X\n' } } }),
    make('common-stdin-dash', ['-']),
    make('common-missing', ['missing']),
    make('common-permission', ['a'], { fixture: { ...file, modes: { '/home/kali/a': 0 } } }),
    make('common-binary', [], { stdinHex: '00ff800a01fe00' }),
    make('common-pipe', ['a'], { fixture: file, transport: 'pipe' }),
    make('common-redirect', ['a'], { fixture: file, io: { stdoutPath: '/home/kali/out' } }),
    make('common-stderr', ['missing', 'a'], {
      fixture: file,
      io: { stderrPath: '/home/kali/errors' },
    }),
    ...['SIGINT', 'SIGTERM'].map((signal) =>
      make(`common-${signal.toLowerCase()}`, [], {
        stdin: null,
        tty: { isTTY: true, columns: 80, rows: 24, ansiSupport: true, interactive: true },
        interaction: { schemaVersion: 1, steps: [{ kind: 'signal', signal }] },
      }),
    ),
  ];
}

export function capturedExpectations(test, capture) {
  if (
    !['GNU_PROBE', 'GNU_REFERENCE'].includes(capture.provenance) ||
    capture.version !== '9.7' ||
    capture.command !== test.command ||
    capture.locale !== 'C' ||
    (capture.harnessHash !== sourceHash('scripts/cli/coreutils-reference.py') &&
      !verificationFresh(test.command, capture))
  )
    throw new Error('Stale or incompatible GNU capture (fingerprint mismatch)');
  const row = capture.cases.find((r) => r.id === test.id);
  if (
    !row?.reproducible ||
    row.requestDigest !== requestDigest(test) ||
    canonical(row.request) !== canonical(referenceRequest(test))
  )
    throw new Error(`GNU request mismatch: ${test.id}`);
  return {
    stdout: { kind: 'EXACT', value: Buffer.from(row.stdoutHex, 'hex').toString() },
    stderr: { kind: 'EXACT', value: Buffer.from(row.stderrHex, 'hex').toString() },
    stdoutHex: row.stdoutHex,
    stderrHex: row.stderrHex,
    exitCode: row.exitCode,
    ...(row.termination ? { termination: row.termination, observations: row.observations } : {}),
    state: [],
  };
}
