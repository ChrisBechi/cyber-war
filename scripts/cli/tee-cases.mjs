// DEV only. Requests contain no expected results; the locked GNU binary supplies them.
import { commonContracts, request } from './common-contracts.mjs';

export function teeRequests() {
  const cases = commonContracts('tee');
  const add = (id, argv, extra = {}) =>
    cases.push(request('tee', id, argv, { stdin: 'abc\n', ...extra }));
  const file = { files: { '/home/kali/a': 'old\n', '/home/kali/b': 'B' } };
  const modes = [
    [],
    ['-p'],
    ['--output-error'],
    ...['warn', 'warn-nopipe', 'exit', 'exit-nopipe'].map((m) => ['--output-error=' + m]),
  ];
  for (const size of [0, 1, 2, 3, 7, 31, 255, 256, 4095, 4096, 4097, 32771, 131075]) {
    const data = Buffer.from(Array.from({ length: size }, (_, i) => (i * 73 + 17) & 255));
    add(`bytes-${size}`, ['a', 'b', 'c'], { stdinHex: data.toString('hex') });
  }
  for (const [i, text] of ['no newline', 'a\nb\nlast', '\r\n', '\0\u00ff'].entries())
    add(`text-${i}`, ['a'], { stdin: text });
  for (const [id, argv, extra] of [
    ['truncate', ['a', 'b', 'c'], { fixture: file }],
    ['repeat', ['a', 'a'], { fixture: file }],
    ['dash', ['-', 'a'], {}],
    ['dash-name', ['--', '-a', '--help'], {}],
    ['permutation', ['a', '-a', 'b'], { fixture: file }],
    ['posix', ['a', '-a', 'b'], { fixture: file, env: { LC_ALL: 'C', POSIXLY_CORRECT: '1' } }],
    ['symlink', ['link', 'a'], { fixture: { ...file, symlinks: { '/home/kali/link': 'a' } } }],
    ['broken-symlink', ['link'], { fixture: { symlinks: { '/home/kali/link': 'new' } } }],
    [
      'hardlink',
      ['a', 'alias'],
      { fixture: { ...file, hardlinks: { '/home/kali/alias': '/home/kali/a' } } },
    ],
    [
      'stdin-file',
      ['a'],
      {
        fixture: { files: { '/home/kali/input': 'source\n' } },
        io: { stdinPath: '/home/kali/input' },
      },
    ],
    ['stdin-alias', ['a'], { fixture: file, io: { stdinPath: '/home/kali/a' } }],
    ['redirect', ['a', 'b'], { fixture: file, io: { stdoutPath: '/home/kali/c' } }],
    [
      'redirect-append',
      ['a', 'b'],
      { fixture: file, io: { stdoutPath: '/home/kali/b', append: true } },
    ],
    ['stdout-alias', ['a', 'a'], { fixture: file, io: { stdoutPath: '/home/kali/a' } }],
    ['pipe', ['a', 'b'], { transport: 'pipe' }],
  ])
    add(id, argv, extra);
  for (const flag of ['-a', '--append']) {
    for (const [id, argv, fixture] of [
      ['existing', ['a', 'b', 'c'], file],
      ['empty', ['a'], { files: { '/home/kali/a': '' } }],
      ['repeat', ['a', 'a'], file],
      ['hardlink', ['a', 'alias'], { ...file, hardlinks: { '/home/kali/alias': '/home/kali/a' } }],
    ])
      add(`append-${flag.slice(1)}-${id}`, [flag, ...argv], { fixture, stdinHex: '00ff800d0a01' });
  }
  for (const [i, flags] of modes.entries()) {
    for (const [id, operands, fixture] of [
      ['first-open', ['dir', 'a', 'b'], { directories: ['/home/kali/dir'] }],
      ['middle-open', ['a', 'dir', 'b'], { directories: ['/home/kali/dir'] }],
      [
        'multiple-open',
        ['missing/a', 'locked', 'a'],
        { files: { '/home/kali/locked': 'unchanged' }, modes: { '/home/kali/locked': 0o444 } },
      ],
      ['write', ['a', '/dev/full', 'b'], {}],
      ['multiple-write', ['/dev/full', 'a', '/dev/full', 'b'], {}],
    ])
      add(`mode-${i}-${id}`, [...flags, ...operands], { fixture });
    add(`mode-${i}-closed`, [...flags, 'a', 'b'], { io: { closedConsumer: true } });
    add(`mode-${i}-closed-only`, flags, { io: { closedConsumer: true } });
    add(`mode-${i}-closed-empty`, [...flags, 'a'], { stdin: '', io: { closedConsumer: true } });
  }
  for (const [i, argv] of [
    ['-ai'],
    ['-ipa'],
    ['--app'],
    ['--ignore-i'],
    ['--output-e=warn'],
    ['--output-error=w'],
    ['--output-error=e'],
    ['--output-error=warn-n'],
    ['--output-error=bad'],
    ['--output-error='],
    ['--output-error', 'warn'],
    ['-pwarn'],
    ['--append=yes'],
    ['--help=x'],
    ['--version=x'],
    ['--help', '--bad'],
    ['--bad', '--help'],
    ['--output-error=bad', '--help'],
    ['--output-error=exit', '-p'],
    ['-p', '--output-error=exit'],
    ['--ignore-interrupts'],
    ['-é'],
    ['--he'],
    ['--ver'],
  ].entries())
    add(`options-${i}`, argv);
  for (const invocation of ['tee', '/usr/bin/tee']) {
    add(`invocation-${invocation === 'tee' ? 'bare' : 'path'}`, ['--help'], { invocation });
  }
  const tty = { isTTY: true, columns: 80, rows: 24, ansiSupport: true, interactive: true };
  for (const [id, flags, steps] of [
    [
      'stream-eof',
      [],
      [
        { kind: 'write', hex: '610a620a' },
        { kind: 'expect', stdoutHex: '610a620a' },
        { kind: 'eof' },
      ],
    ],
    [
      'stream-int',
      [],
      [
        { kind: 'write', hex: '61620a' },
        { kind: 'signal', signal: 'SIGINT' },
      ],
    ],
    [
      'stream-term',
      ['-i'],
      [
        { kind: 'write', hex: '61620a' },
        { kind: 'signal', signal: 'SIGTERM' },
      ],
    ],
    ...['-i', '--ignore-interrupts', '-ip'].map((flag, i) => [
      `ignore-${i}`,
      [flag],
      [
        { kind: 'signal', signal: 'SIGINT' },
        { kind: 'write', hex: '61620a' },
        { kind: 'expect', stdoutHex: '61620a' },
        { kind: 'eof' },
      ],
    ]),
  ])
    add(`tty-${id}`, [...flags, 'a', 'b'], {
      stdin: null,
      tty,
      interaction: { schemaVersion: 1, steps },
    });
  for (const [i, flags] of modes.entries()) {
    add(`stdout-full-${i}`, [...flags, 'a', 'b'], {
      fixture: { symlinks: { '/home/kali/out': '/dev/full' } },
      io: { stdoutPath: '/home/kali/out' },
    });
    add(`open-and-closed-${i}`, [...flags, 'missing/a', 'a', 'b'], {
      io: { closedConsumer: true },
    });
  }
  const large = Buffer.from(Array.from({ length: 16387 }, (_, i) => i % 251)).toString('hex');
  for (const append of [false, true])
    add(`large-alias-${append}`, [...(append ? ['-a'] : []), 'a', 'alias'], {
      stdin: null,
      fixture: {
        bytes: { '/home/kali/input': large },
        files: { '/home/kali/a': 'old' },
        hardlinks: { '/home/kali/alias': '/home/kali/a' },
      },
      io: { stdinPath: '/home/kali/input' },
    });
  for (const [id, argv, signals] of [
    ['int', [], ['SIGINT']],
    ['term', [], ['SIGTERM']],
    ['ignore-term', ['-i'], ['SIGINT', 'SIGTERM']],
    ['warn-int', ['--output-error=warn'], ['SIGINT']],
  ])
    add(`backpressure-${id}`, [...argv, 'a', 'b'], {
      stdin: null,
      fixture: { files: { '/home/kali/input': 'abcd'.repeat(65536) } },
      io: { stdinPath: '/home/kali/input' },
      interaction: {
        schemaVersion: 1,
        steps: signals.map((signal) => ({ kind: 'signal', signal, waitFor: 'output' })),
      },
    });
  return cases;
}
