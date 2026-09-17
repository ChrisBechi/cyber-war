import { commonContracts, request } from './common-contracts.mjs';

export function headRequests() {
  const cases = commonContracts('head');
  const add = (id, argv, extra) => cases.push(request('head', id, argv, extra));
  const matrix = (id, rows, extra) => rows.forEach((argv, i) => add(`${id}-${i}`, argv, extra));
  add('binary', ['-c', '4', 'data'], {
    stdin: null,
    fixture: { bytes: { '/home/kali/data': '00ff80c3a90a7a' } },
  });
  add('lines', ['-n', '2'], { stdin: 'a\nb\nc' });
  add('negative-or-start', ['-n', '-1'], { stdin: 'a\nb\nc' });
  add('headers', ['-n1', 'a', 'b'], {
    stdin: null,
    fixture: { files: { '/home/kali/a': 'A\n', '/home/kali/b': 'B\n' } },
  });
  add('invalid', ['-c', 'oops'], { stdin: null });
  add('version', ['--version'], { stdin: null });
  add('unknown-option', ['--not-implemented'], { stdin: null });
  cases.push({ ...request('head', 'stdin', ['-n', '1'], { stdin: 'a\nb\n' }), id: 'head/stdin' });
  add('zero-terminated-long', ['--zero-terminated', '-n1'], { stdinHex: '61006200' });
  add('empty', [], { stdin: '' });
  add('large-file', ['-c8193', 'large'], {
    stdin: null,
    fixture: {
      bytes: {
        '/home/kali/large': Buffer.from(Array.from({ length: 131072 }, (_, i) => i % 256)).toString(
          'hex',
        ),
      },
    },
  });
  add('directory-permission-zero', ['-n0', 'dir'], {
    fixture: { directories: ['/home/kali/dir'], modes: { '/home/kali/dir': 0 } },
  });
  matrix('mode', [
    [],
    ['-n', '0'],
    ['-n', '1'],
    ['-n', '+2'],
    ['-n', '-2'],
    ['-n', '-0'],
    ['-c', '-2'],
    ['-c', '+2'],
    ['-2'],
    ['-2c'],
    ['-2cq'],
    ['-2qv'],
    ['-2v'],
    ['-2z'],
    ['-n', '2', '-3'],
    ['-0'],
    ['--bytes=2'],
    ['--lines=-1'],
    ['-z', '-n', '2'],
    ['-n', '1', '-', '-'],
    ['-c', '1', '-', '-'],
    ['-n', '-1', '-', '-'],
  ]);
  matrix(
    'count',
    [
      '',
      '+',
      '-',
      '--1',
      '++1',
      '1.0',
      '1x',
      ' 2',
      '2 ',
      '01',
      '0x10',
      '18446744073709551615',
      '18446744073709551616',
      '-18446744073709551616',
      '999999999999999999999x',
      '2é',
      '-+2',
      '\t+2',
      ' -2',
    ].map((n) => ['-n', n]),
  );
  matrix(
    'suffix',
    [
      'b',
      'k',
      'K',
      'KB',
      'KiB',
      'kB',
      'kiB',
      'm',
      'M',
      'MB',
      'MiB',
      'g',
      'G',
      'GB',
      'GiB',
      'T',
      'P',
      'E',
      'Z',
      'Y',
      'R',
      'Q',
      'B',
      'c',
      'w',
      'Kib',
      'kib',
      'kk',
    ].map((s) => ['-c', '1' + s]),
    { stdin: 'x'.repeat(1200) },
  );
  matrix(
    'suffix-zero',
    ['E', 'Z', 'Y', 'R', 'Q'].map((s) => ['-c', '0' + s]),
  );
  matrix('parse', [
    ['--bogus'],
    ['-x'],
    ['-é'],
    ['--v'],
    ['--ver'],
    ['--h'],
    ['--l'],
    ['-n'],
    ['--bytes'],
    ['--lines='],
    ['--silent=x'],
    ['--help=x'],
    ['--bogus', '--help'],
    ['--help', '--bogus'],
    ['-n', 'bad', '--help'],
    ['-n', 'bad', '-n', '2'],
    ['-n', '2', '-c', '3'],
    ['-c', '3', '-n', '2'],
    ['-n', '--help'],
    ['-1x'],
    ['--', '-2'],
    ['--s'],
    ['--by=2'],
    ['--ze'],
    ['-zqvn2'],
    ['-2', '-n3'],
    ['-1b'],
    ['-1k'],
    ['-1m'],
    ['-1cl'],
    ['-1lc'],
    ['-1K'],
    ['-1n'],
    ['--lines=2', '--bytes=1'],
  ]);
  const fixture = {
    files: { '/home/kali/a': 'A\nB\nC', '/home/kali/b': 'D\nE\nF' },
    directories: ['/home/kali/dir'],
  };
  matrix(
    'file',
    [
      ['a', 'b'],
      ['missing', 'a'],
      ['a', 'missing', 'b'],
      ['-n0', 'missing', 'a'],
      ['-v', 'a'],
      ['-qv', 'a', 'b'],
      ['-vq', 'a', 'b'],
      ['-n1', '-', 'a', '-'],
      ['a', '-n', '1'],
      ['-1', 'a'],
      ['a', '-1'],
      ['-n0', 'dir', 'a'],
      ['dir', 'a'],
      ['--quiet', 'a', 'b'],
      ['--silent', 'a', 'b'],
      ['--verbose', 'a'],
      ['-n-1', 'a', 'b'],
      ['-c-1', 'a', 'b'],
      ['a', '-'],
      ['-c0', 'dir'],
      ['-c-0', 'a'],
    ],
    { fixture },
  );
  for (const mode of ['n', 'c'])
    add(`stdin-file-${mode}`, ['-' + mode, '1', '-', '-'], {
      fixture,
      io: { stdinPath: '/home/kali/a' },
    });
  add('self-append', ['-n1', 'a'], { fixture, io: { stdoutPath: '/home/kali/a', append: true } });
  for (const [index, flag] of ['-n-1', '-c-2'].entries())
    add(`self-negative-${index}`, [flag, 'a'], {
      stdin: null,
      fixture: { files: { '/home/kali/a': 'x\n'.repeat(3000) } },
      io: { stdoutPath: '/home/kali/a', append: true },
    });
  add('append', ['-c3', 'a'], { fixture, io: { stdoutPath: '/home/kali/b', append: true } });
  add('posix', ['a', '-n', '1'], { fixture, env: { LC_ALL: 'C', POSIXLY_CORRECT: '1' } });
  for (const [i, file] of ['a b', 'a\tb', 'a\nb', "a'b", 'a\\b', 'a\x07b', 'é日本'].entries()) {
    add(`header-${i}`, ['-v', file], { fixture: { files: { ['/home/kali/' + file]: 'X\n' } } });
    add(`missing-${i}`, [file]);
  }
  add('links', ['link', 'hard'], {
    fixture: {
      files: { '/home/kali/a': 'X\nY' },
      symlinks: { '/home/kali/link': 'a' },
      hardlinks: { '/home/kali/hard': '/home/kali/a' },
    },
  });
  add('broken-link', ['broken'], { fixture: { symlinks: { '/home/kali/broken': 'absent' } } });
  add('link-loop', ['loop'], { fixture: { symlinks: { '/home/kali/loop': 'loop' } } });
  for (const flag of ['-n', '-c'])
    for (const count of ['0', '1', '2', '-0', '-1', '-2', '50', '-50'])
      add(`binary-${flag.slice(1)}-${count.replace('-', 'minus')}`, ['-z', flag, count], {
        stdinHex: 'ff00800a004100424300',
      });
  matrix(
    'lines-suffix',
    [
      ['-n', '1K'],
      ['-n', '1KB'],
      ['-n', '1KiB'],
    ],
    { stdin: 'x\n'.repeat(1100) },
  );
  add('closed-consumer', [], { stdin: 'x\n'.repeat(100), io: { closedConsumer: true } });
  const tty = { isTTY: true, columns: 80, rows: 24, ansiSupport: true, interactive: true };
  add('legacy-utf8', ['-1é']);
  add('pipe-read-ahead', ['-qn1', '-', '-'], { stdin: 'A\n' + 'x'.repeat(1022) + 'B\nrest' });
  for (const mode of ['n', 'c'])
    for (const size of [6, 2048, 8192]) {
      const steps = [];
      const data = 'x\n'.repeat(size / 2);
      for (let i = 0; i < data.length; i += 2048)
        steps.push({ kind: 'write', hex: Buffer.from(data.slice(i, i + 2048)).toString('hex') });
      steps.push({ kind: 'signal', signal: 'SIGTERM' });
      add(`negative-${mode}-signal-${size}`, [mode === 'n' ? '-n-1' : '-c-2'], {
        stdin: null,
        tty,
        interaction: { schemaVersion: 1, steps },
      });
    }
  const write = (s) => ({ kind: 'write', hex: Buffer.from(s).toString('hex') });
  const expect = (s) => ({ kind: 'expect', stdoutHex: Buffer.from(s).toString('hex') });
  for (const [id, argv, steps] of [
    ['tty-lines', ['-n2'], [write('first\n'), expect('first\n'), write('last\n')]],
    ['tty-bytes', ['-c3'], [write('x\n'), expect('x\n'), write('y\n')]],
    ['tty-repeat-bytes', ['-qc1', '-', '-'], [write('abc\n')]],
    ['tty-partial-eof', ['-n1'], [write('abc'), { kind: 'eof' }, { kind: 'eof' }]],
    ['tty-negative', ['-n-1'], [write('a\nb\nc\n'), { kind: 'eof' }]],
    ['tty-negative-bytes', ['-c-2'], [write('abc\n'), { kind: 'eof' }]],
  ])
    add(id, argv, { stdin: null, tty, interaction: { schemaVersion: 1, steps } });
  for (const signal of ['SIGINT', 'SIGTERM'])
    add(`tty-partial-${signal.toLowerCase()}`, ['-n3'], {
      stdin: null,
      tty,
      interaction: {
        schemaVersion: 1,
        steps: [write('a\n'), expect('a\n'), { kind: 'signal', signal }],
      },
    });
  return cases;
}
