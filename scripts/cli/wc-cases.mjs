// Parametric requests, never expectations. GNU's locked C locale is mandatory.
import { request, commonContracts } from './common-contracts.mjs';
export function wcRequests() {
  const cases = commonContracts('wc');
  const add = (id, argv, extra = {}) =>
    cases.push(
      request(
        'wc',
        id.replace(/[A-Z]/g, (c) => 'upper' + c.toLowerCase()),
        argv,
        extra,
      ),
    );
  const data = {
    empty: '',
    ascii: 'one two\nlast',
    one: 'a',
    lf: '\n',
    blank: '\n\n',
    cr: '\r',
    crlf: 'a\r\nb\r\n',
    whitespace: ' \t\n\v\f\r',
    punctuation: 'a,b.!?:b',
    nul: 'a\0b\n',
    unicode: 'é € 😀 界 é\u200b\u2060\u00a0z\n',
    nonbreaking: 'a\u00a0b\u2007c\u202fd\u2060e',
    controls: 'abc\bZ\x1b[0m\0\x7f\vQ\fABC\rD',
    tabs: 'x\tY\tZ\n\t\t',
    long: 'abc\t'.repeat(2049) + '\nlast',
  };
  for (const [id, stdin] of Object.entries(data))
    for (const flag of ['', '-c', '-m', '-l', '-w', '-L', '-lwmcL'])
      add(`count-${id}-${flag || 'default'}`, flag ? [flag] : [], { stdin });
  for (const [id, stdinHex] of Object.entries({
    all: Buffer.from(Array.from({ length: 256 }, (_, i) => i)).toString('hex'),
    invalid: 'ff8061c0af20edA08062f49080800a'.toLowerCase(),
    truncated: '61e282',
    split: '61e282ac62f09f98800d0a',
    isolated: 'ff',
    nul: '00',
  }))
    for (const flag of ['-lwmcL', '-m', '-w', '-L'])
      add(`binary-${id}-${flag}`, [flag], { stdinHex });
  for (const byte of [...Array.from({ length: 33 }, (_, i) => i), 127, 128, 160, 255])
    add(`class-byte-${byte}`, ['-lwmcL'], {
      stdinHex: Buffer.from([65, byte, 66]).toString('hex'),
    });
  for (const n of [0, 1, 7, 8, 9, 15, 16, 31])
    add(`width-tab-${n}`, ['-L'], { stdin: 'x'.repeat(n) + '\tx' });
  let seed = 97;
  const random = Buffer.from(
    Array.from({ length: 8193 }, () => {
      seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
      return seed >>> 24;
    }),
  );
  add('binary-random', ['-lwmcL'], { stdinHex: random.toString('hex') });
  const fixture = {
    files: {
      '/home/kali/a': 'A\n',
      '/home/kali/b': 'B B\n',
      '/home/kali/empty': '',
      '/home/kali/-dash': 'dash\n',
      '/home/kali/a b': 'space\n',
      '/home/kali/tab\tname': 'tab\n',
      '/home/kali/new\nline': 'newline\n',
      '/home/kali/é': 'utf8\n',
      '/home/kali/denied': 'denied\n',
    },
    directories: ['/home/kali/dir'],
    modes: { '/home/kali/denied': 0 },
    hardlinks: { '/home/kali/hard': '/home/kali/a' },
    symlinks: { '/home/kali/link': 'a', '/home/kali/broken': 'missing', '/home/kali/loop': 'loop' },
  };
  const argvCases = [
    [],
    ['a'],
    ['a', 'b'],
    ['a', 'b', 'empty'],
    ['-'],
    ['-', '-'],
    ['a', '-'],
    ['-', 'a'],
    ['empty'],
    ['missing'],
    ['dir'],
    ['denied'],
    ['broken'],
    ['loop'],
    ['a/x'],
    ['missing/x'],
    ['missing', 'a'],
    ['a', 'dir', 'denied', 'b'],
    ['hard', 'a', 'link'],
    ['a b', 'tab\tname', 'new\nline', 'é'],
    [''],
    ['', 'a'],
    ['--', '-dash'],
  ];
  for (const [i, argv] of argvCases.entries())
    for (const flags of [[], ['-c'], ['-lwmcL']])
      add(`operands-${i}-${flags.join('') || 'default'}`, [...flags, ...argv], { fixture });
  for (const mode of [
    'auto',
    'always',
    'only',
    'never',
    'a',
    'al',
    'au',
    'o',
    'n',
    '',
    'x',
    'AUTO',
  ])
    for (const files of [[], ['a', 'b']])
      add(`total-${mode || 'empty'}-${files.length}`, ['--total=' + mode, ...files], { fixture });
  for (const n of [9, 10, 99, 100, 999, 1000])
    for (const flags of [[], ['-c'], ['-mc'], ['-L']])
      add(`format-digits-${n}-${flags.join('') || 'default'}`, [...flags, 'a', 'b'], {
        fixture: { files: { '/home/kali/a': 'x'.repeat(n), '/home/kali/b': '\n' } },
      });
  const options = [
    ['-cm'],
    ['-ml'],
    ['-wlc'],
    ['-Llc'],
    ['-llwwmmccLL'],
    ['-L', '-w', '-m', '-l', '-c'],
    ['--bytes'],
    ['--chars'],
    ['--words'],
    ['--lines'],
    ['--max-line-length'],
    ['--b'],
    ['--c'],
    ['--l'],
    ['--w'],
    ['--m'],
    ['--t=only'],
    ['--to'],
    ['--files0-f'],
    ['--bytes=x'],
    ['--help=x'],
    ['--version=x'],
    ['-Z'],
    ['-é'],
    ['--no-option'],
    ['--help', '-Z'],
    ['--version', '-Z'],
    ['-Z', '--help'],
    ['a', '-c'],
    ['--total=only', '--total=never', 'a', 'b'],
  ];
  for (const [i, argv] of options.entries()) add(`options-${i}`, argv, { fixture });
  add('options-posix', ['a', '-c'], { fixture, env: { LC_ALL: 'C', POSIXLY_CORRECT: '1' } });
  add('invocation-short', ['--help'], { invocation: 'wc' });
  const lists = {
    empty: '',
    one: 'a',
    two: 'a\0b',
    trailing: 'a\0b\0',
    blank: '\0',
    mid: 'a\0\0b',
    empty2: '\0\0',
    missing: 'a\0missing\0b\0',
    dir: 'dir\0a',
    denied: 'denied\0b',
    stdin: '-\0a',
    repeat: 'a\0a\0',
    links: 'link\0hard\0broken\0loop\0',
    odd: 'a b\0tab\tname\0new\nline\0-dash\0é\0',
  };
  for (const [id, list] of Object.entries(lists))
    for (const source of ['file', 'pipe', 'redirect'])
      add(`files0-${source}-${id}`, ['--files0-from=' + (source === 'file' ? 'list' : '-')], {
        fixture: { ...fixture, files: { ...fixture.files, '/home/kali/list': list } },
        stdin: source === 'pipe' ? list : 'data\n',
        ...(source === 'redirect' ? { io: { stdinPath: '/home/kali/list' } } : {}),
      });
  for (const mode of ['auto', 'always', 'only', 'never'])
    for (const list of ['', 'a\0b\0', '\0a\0'])
      add(`files0-total-${mode}-${list.length}`, ['--files0-from=list', '--total=' + mode], {
        fixture: { ...fixture, files: { ...fixture.files, '/home/kali/list': list } },
      });
  for (const [i, argv] of [
    ['--files0-from=list', 'a'],
    ['--files0-from=missing'],
    ['--files0-from=dir'],
    ['--files0-from=denied'],
    ['--files0-from=broken'],
    ['--files0-from=list', '--files0-from=empty'],
    ['--files0-from=list', '--', '-dash'],
  ].entries())
    add(`files0-errors-${i}`, argv, {
      fixture: { ...fixture, files: { ...fixture.files, '/home/kali/list': 'a\0b\0' } },
    });
  for (const flag of ['-c', '-m', '-l', '-w', '-L', '-lwmcL']) {
    add(`pipe-${flag}`, [flag], { stdinHex: random.toString('hex'), transport: 'pipe' });
    add(`redirect-${flag}`, [flag, '-', '-'], {
      fixture,
      io: { stdinPath: '/home/kali/a', stdoutPath: '/home/kali/out' },
    });
    add(`tty-eof-${flag}`, [flag], {
      stdin: null,
      tty: { isTTY: true, columns: 80, rows: 24, ansiSupport: true, interactive: true },
      interaction: {
        schemaVersion: 1,
        steps: [
          { kind: 'write', hex: Buffer.from('é tab\tword\n').toString('hex') },
          { kind: 'eof' },
        ],
      },
    });
  }
  for (const signal of ['SIGINT', 'SIGTERM'])
    add(`tty-data-${signal}`, ['-lwmcL'], {
      stdin: null,
      tty: { isTTY: true, columns: 80, rows: 24, ansiSupport: true, interactive: true },
      interaction: {
        schemaVersion: 1,
        steps: [
          { kind: 'write', hex: '6162630a' },
          { kind: 'signal', signal },
        ],
      },
    });
  add('files0-source-loop', ['--files0-from=loop'], { fixture });
  add('files0-extra-quote', ['--files0-from=empty', "a'b"], { fixture });
  return cases;
}
