// DEV requests only; expected bytes must come from the locked GNU 9.7 binary.
import { commonContracts, request } from './common-contracts.mjs';

export function base64Requests() {
  const cases = commonContracts('base64');
  const add = (id, argv = [], extra = {}) =>
    cases.push(request('base64', id, argv, { stdin: 'foo', ...extra }));
  add('encode');
  add('decode', ['-d'], { stdin: 'Zm9v' });
  add('wrap', ['-w2']);
  add('garbage', ['-di'], { stdin: '!Z m9v?' });
  add('invalid', ['-d'], { stdin: 'Zm9v!' });
  add('unknown-option', ['--unknown']);
  add('version', ['--version']);
  for (const size of [
    0, 1, 2, 3, 4, 5, 56, 57, 58, 59, 75, 76, 77, 255, 256, 4095, 4096, 4097, 8193, 65539,
  ]) {
    const bytes = Buffer.from(Array.from({ length: size }, (_, i) => i % 256));
    add(`encode-binary-${size}`, [], { stdinHex: bytes.toString('hex') });
    add(`decode-binary-${size}`, ['--decode'], { stdin: bytes.toString('base64') });
  }
  for (const wrap of [
    '0',
    '1',
    '2',
    '3',
    '4',
    '5',
    '75',
    '76',
    '77',
    '1024',
    '+4',
    '-1',
    '-0',
    ' 4',
    '4 ',
    '',
    'x',
    '1k',
    '0x10',
    '01',
    '18446744073709551615',
    '18446744073709551616',
    '999999999999999999999999999',
  ]) {
    add(`wrap-parse-${cases.length}`, ['--wrap=' + wrap], { stdin: 'foobar'.repeat(30) });
  }
  for (const [i, argv] of [
    ['-w'],
    ['--wrap'],
    ['--wr'],
    ['--wr=4'],
    ['-w4'],
    ['-w', '4'],
    ['-iw4'],
    ['-diw0'],
    ['--decode=true'],
    ['--ignore-garbage=x'],
    ['-D'],
    ['-h'],
    ['-v'],
    ['--d'],
    ['--i'],
    ['--he'],
    ['--ver'],
    ['--help', '--bad'],
    ['--bad', '--help'],
    ['--help=x'],
    ['--version=x'],
    ['--wrap=bad', '--help'],
    ['--decode', '--wrap=bad'],
    ['--wrap=2', '--wrap=0'],
    ['-w0', '-w2'],
    ['--', '-'],
    ['a', 'b'],
    ['-', '-'],
    ['missing', 'a'],
    ['a', '-w2'],
    ['a', '--decode'],
    ['--', '-w2'],
    ['--', 'missing space'],
    ['--', "missing'quote"],
    ['--', 'missing\nline'],
  ].entries())
    add(`options-${i}`, argv, { fixture: { files: { '/home/kali/a': 'Zm9v' } } });
  add('posix-order', ['a', '-w2'], {
    env: { LC_ALL: 'C', POSIXLY_CORRECT: '1' },
    fixture: { files: { '/home/kali/a': 'foo' } },
  });
  for (const [i, input] of [
    '',
    'Zg==',
    'Zm8=',
    'Zm9v',
    'Z',
    'Zg',
    'Zm8',
    '=',
    '==',
    '===',
    '====',
    '=AAA',
    'A=AA',
    'AA=A',
    'AAA=',
    'AA==',
    'A===',
    'Zg=',
    'Zg===',
    'Zg==A',
    'Zg==Zg==',
    'Zm8=Zm8=',
    'Zm9vZg==',
    'Zm9vZ',
    'Zm9vZg',
    'Zm9vZg=',
    'Zm9v!',
    '!Zm9v',
    'Zm!9v',
    'Zm9v====',
    'Zh==',
    'Zm9=',
    'Zg==\n',
    'Z\ng==',
    'Zg=\n=',
    'Zg==\nZg==',
    'Zm9v\r\n',
  ].entries())
    for (const ignore of [false, true])
      add(`decode-matrix-${i}-${ignore ? 'ignore' : 'strict'}`, [ignore ? '-di' : '-d'], {
        stdin: input,
      });
  for (const byte of [0, 9, 10, 11, 12, 13, 32, 33, 45, 95, 127, 128, 255])
    for (const ignore of [false, true]) {
      const bytes = Buffer.from([90, 109, byte, 57, 118]);
      add(`decode-separator-${byte}-${ignore ? 'ignore' : 'strict'}`, [ignore ? '-di' : '-d'], {
        stdinHex: bytes.toString('hex'),
      });
    }
  for (const [i, argv, fixture] of [
    [0, ['a'], { files: { '/home/kali/a': 'foo' } }],
    [1, ['a'], { files: { '/home/kali/a': '' } }],
    [2, ['a'], { directories: ['/home/kali/a'] }],
    [3, ['a'], { directories: ['/home/kali/a'], modes: { '/home/kali/a': 0 } }],
    [4, ['link'], { files: { '/home/kali/a': 'foo' }, symlinks: { '/home/kali/link': 'a' } }],
    [5, ['link'], { symlinks: { '/home/kali/link': 'missing' } }],
    [6, ['link'], { symlinks: { '/home/kali/link': 'link' } }],
  ])
    add(`file-${i}`, argv, { fixture });
  add('stdin-file', [], {
    fixture: { files: { '/home/kali/a': 'foo' } },
    io: { stdinPath: '/home/kali/a' },
  });
  add('partial-redirect', ['-d'], {
    stdin: 'Zm9vZg!',
    io: { stdoutPath: '/home/kali/out', stderrPath: '/home/kali/errors' },
  });
  add('append', ['-w0'], {
    fixture: { files: { '/home/kali/out': 'prefix:' } },
    io: { stdoutPath: '/home/kali/out', append: true },
  });
  add('closed-consumer', [], { stdin: 'a'.repeat(16384), io: { closedConsumer: true } });
  for (const [i, argv, text] of [
    [0, [], 'foo\n'],
    [1, ['-d'], 'Zm9v\n'],
    [2, ['-d'], 'Zm9v!\n'],
    [3, ['-di'], '!Zm9v\n'],
  ])
    add(`tty-${i}`, argv, {
      stdin: null,
      tty: { isTTY: true, columns: 80, rows: 24, ansiSupport: true, interactive: true },
      interaction: {
        schemaVersion: 1,
        steps: [{ kind: 'write', hex: Buffer.from(text).toString('hex') }, { kind: 'eof' }],
      },
    });
  for (const invocation of ['base64'])
    for (const arg of ['--help', '--version', '--bad'])
      add(`alias-${cases.length}`, [arg], { invocation });
  // A bounded deterministic corpus, with GNU as the oracle for invalid strings.
  let state = 0x1c500;
  for (let i = 0; i < 24; i++) {
    const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=!?\n\r ';
    const value = Array.from({ length: 5 + i * 3 }, () => {
      state = (Math.imul(state, 1664525) + 1013904223) >>> 0;
      return alphabet[state % alphabet.length];
    }).join('');
    add(`decode-fuzz-${i}`, [i % 2 ? '-di' : '-d'], { stdin: value });
  }
  for (const size of [6, 7, 8, 63, 64, 65, 257, 30719, 30720, 30721, 46079, 46080, 46081]) {
    const bytes = Buffer.from(Array.from({ length: size }, (_, i) => i % 256));
    add(`encode-binary-${size}`, [], { stdinHex: bytes.toString('hex') });
    add(`decode-binary-${size}`, ['-d'], { stdin: bytes.toString('base64') });
  }
  add('hardlink', ['linked'], {
    fixture: {
      files: { '/home/kali/a': 'same inode\n' },
      hardlinks: { '/home/kali/linked': '/home/kali/a' },
    },
  });
  for (const size of [4095, 4096, 4097, 16385]) {
    add(`decode-late-invalid-${size}`, ['-d'], { stdin: 'Zm9v'.repeat(size) + 'Zg!' });
  }
  add('decode-invalid-append', ['-d'], {
    stdin: 'Zm9vZg!',
    fixture: { files: { '/home/kali/out': 'prefix:' } },
    io: { stdoutPath: '/home/kali/out', append: true },
  });
  for (const decode of [false, true]) {
    for (const size of [4, 4096, 8192, 30720, 46080]) {
      // Short canonical lines avoid the kernel's per-line truncation limit.
      const line = decode ? 'Zm9v'.repeat(15) + '\n' : 'a'.repeat(60) + '\n';
      const input = line.repeat(Math.floor(size / line.length)) + '\n';
      add(`tty-${decode ? 'decode' : 'encode'}-interrupt-${size}`, decode ? ['-d'] : [], {
        stdin: null,
        tty: { isTTY: true, columns: 80, rows: 24, ansiSupport: true, interactive: true },
        interaction: {
          schemaVersion: 1,
          steps: [
            ...Array.from({ length: Math.ceil(input.length / 4000) }, (_, i) => ({
              kind: 'write',
              hex: Buffer.from(input.slice(i * 4000, (i + 1) * 4000)).toString('hex'),
            })),
            { kind: 'signal', signal: 'SIGINT' },
          ],
        },
      });
    }
  }
  return cases;
}
