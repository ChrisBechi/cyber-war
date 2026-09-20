// Requests only. Roundtrip list bytes are GNU emissions, not project output.
import { request, commonContracts } from './common-contracts.mjs';
import { json } from './io.mjs';

export const names = [
  'a',
  'a b',
  'tab\tname',
  'back\\slash',
  'new\nline',
  'cr\rname',
  '-dash',
  'é',
  'a)b(c',
  "a'b",
];
export const fixture = {
  files: Object.fromEntries([
    ...names.map((n) => ['/home/kali/' + n, 'abc']),
    ['/home/kali/empty', ''],
    ['/home/kali/changed', 'changed'],
    ['/home/kali/denied', 'abc'],
  ]),
  directories: ['/home/kali/dir'],
  modes: { '/home/kali/denied': 0 },
  symlinks: { '/home/kali/link': 'a', '/home/kali/broken': 'missing', '/home/kali/loop': 'loop' },
  hardlinks: { '/home/kali/hard': '/home/kali/a' },
};
export function sha256sumSeeds() {
  return [[], ['-b'], ['--tag'], ['--zero'], ['--tag', '--zero']].flatMap((flags, i) =>
    names.map((name, j) =>
      request('sha256sum', `escaping-seed-${i}-${j}`, [...flags, '--', name], { fixture }),
    ),
  );
}
export function sha256sumRequests() {
  const cases = [...commonContracts('sha256sum'), ...sha256sumSeeds()];
  const add = (id, argv, extra = {}) => cases.push(request('sha256sum', id, argv, extra));
  const records = json('tests/cli/fixtures/sha256sum-records.json');
  const record = (mode, index) =>
    Buffer.from(records.records[`escaping-seed-${mode}-${index}`], 'hex').toString();
  const good = record(0, 0),
    digest = good.slice(0, 64);
  const check = (id, list, flags = [], extra = {}) =>
    add(id, ['-c', ...flags, 'list'], {
      fixture: { ...fixture, files: { ...fixture.files, '/home/kali/list': list } },
      ...extra,
    });
  const corpus = {
    empty: '',
    ascii: 'abc',
    lines: 'a\nb\n',
    crlf: 'a\r\nb\r\n',
    nul: 'a\0b',
    unicode: 'é 😀 界\n',
    long: 'abc\0'.repeat(4097),
  };
  for (const [id, stdin] of Object.entries(corpus))
    for (const argv of [[], ['-'], ['-b'], ['-t']])
      add(`hash-basic-${id}-${argv.join('') || 'default'}`, argv, { stdin });
  for (const [id, stdinHex] of Object.entries({
    all: Buffer.from(Array.from({ length: 256 }, (_, i) => i)).toString('hex'),
    invalid: 'ff8061c0af00eda0800d0a',
    truncated: '61e282',
  }))
    add(`binary-${id}`, [], { stdinHex });
  for (const [i, argv] of [
    ['a'],
    ['a', 'empty'],
    ['a', '-'],
    ['-', 'a'],
    ['-', '-'],
    ['a', 'missing', 'empty'],
    ['missing', 'a'],
    ['dir', 'a'],
    ['denied', 'a'],
    ['broken', 'a'],
    ['loop', 'a'],
    ['a/x', 'a'],
    ['missing/x', 'a'],
    ['hard', 'link', 'a'],
    [''],
    ['--', '-dash'],
  ].entries())
    add(`multiple-files-${i}`, argv, { fixture, stdin: 'abc' });
  for (const mode of [0, 1, 2])
    for (let n = 0; n < names.length; n++)
      check(`check-valid-roundtrip-${mode}-${n}`, record(mode, n));
  check('check-valid-many', names.map((_, n) => record(0, n)).join(''));
  check('check-valid-no-lf', good.slice(0, -1));
  check('check-valid-crlf', good.replaceAll('\n', '\r\n'));
  check('check-valid-upper', digest.toUpperCase() + good.slice(64));
  const malformed = {
    short: digest.slice(1) + '  a\n',
    long: digest + '0  a\n',
    nonhex: 'z' + good.slice(1),
    noseparator: digest + 'a\n',
    marker: digest + ' ?a\n',
    noname: digest + '  \n',
    prefix: 'x' + good,
    suffix: good.trimEnd() + 'x\n',
    blank: '\n',
    space: ' \n',
    tab: '\t\n',
    comment: '# ignored\n',
    indentedcomment: ' # no\n',
    escape: '\\' + digest + '  back\\q\n',
    trailingescape: '\\' + digest + '  a\\\n',
    escapednul: '\\' + digest + '  a\0x\n',
    rawnull: digest + '  a\0ignored\n',
    reversed: digest + ' a\n',
    reversedafter: good + digest + ' a\n',
    reversedfirst: digest + ' a\n' + good,
    separatorTab: digest + '\t a\n',
    leading: ' \t' + good,
    separatorVT: digest + '\v a\n',
    tag: 'SHA256(a)=' + digest + '\n',
    tagspaces: 'SHA256  (a) = ' + digest + '\n',
    tagtrailing: 'SHA256 (a) = ' + digest + ' \n',
    tagmissing: 'SHA256 (a = ' + digest + '\n',
    tagempty: 'SHA256 () = ' + digest + '\n',
    tagcase: 'sha256 (a) = ' + digest + '\n',
    garbage: 'garbage\n',
  };
  const policies = [
    [],
    ['--warn'],
    ['--strict'],
    ['--quiet'],
    ['--status'],
    ['--strict', '--warn'],
    ['--strict', '--status'],
  ];
  for (const [id, line] of Object.entries(malformed))
    for (const [i, flags] of policies.entries())
      check(`check-malformed-${id.toLowerCase()}-${i}`, line + good, flags);
  for (const [id, list] of Object.entries({
    empty: '',
    blank: '\n\r\n',
    comments: '# comment\n',
    bad: 'bad\nxx\n',
    wrong: '0'.repeat(64) + '  a\n',
    missing: digest + '  missing\n',
    mixed: good + '0'.repeat(64) + '  a\n' + digest + '  missing\n',
    allmissing: digest + '  missing\n' + digest + '  broken\n',
    badmissing: 'bad\n' + digest + '  missing\n',
    someMissing: good + digest + '  missing\n',
  }))
    for (const [i, flags] of [
      ...policies,
      ['--ignore-missing'],
      ['--ignore-missing', '--strict'],
      ['--ignore-missing', '--status'],
      ['--ignore-missing', '--warn'],
    ].entries())
      check(`check-results-${id.toLowerCase()}-${i}`, list, flags);
  for (const [i, flags] of [
    ['--warn', '--status'],
    ['--status', '--warn'],
    ['--quiet', '--warn'],
    ['--warn', '--quiet'],
    ['--status', '--quiet'],
    ['--quiet', '--status'],
  ].entries())
    check(`options-check-order-${i}`, 'bad\n' + good, flags);
  for (const name of ['missing', 'dir', 'denied', 'broken', 'loop', 'a/x']) {
    check(`filesystem-record-${name.replace('/', '-')}`, digest + '  ' + name + '\n' + good);
    for (const flags of [[], ['--status'], ['--ignore-missing']])
      add(
        `filesystem-list-${name.replace('/', '-')}-${flags.join('') || 'default'}`,
        ['-c', ...flags, name, 'empty'],
        { fixture },
      );
  }
  for (const source of ['pipe', 'redirect', 'implicit'])
    for (const [id, list] of Object.entries({
      good,
      bad: 'bad\n' + good,
      dash: digest + '  -\n',
      mixed: good + digest + '  -\n',
    }))
      add(`stdin-check-${source}-${id}`, source === 'implicit' ? ['-c'] : ['-c', '-'], {
        fixture: { ...fixture, files: { ...fixture.files, '/home/kali/list': list } },
        stdin: list,
        ...(source === 'redirect'
          ? { io: { stdinPath: '/home/kali/list' } }
          : source === 'pipe'
            ? { transport: 'pipe' }
            : {}),
      });
  check('stdin-check-data', digest + '  -\n', [], { stdin: 'abc' });
  check('stdin-check-repeated', digest + '  -\n' + digest + '  -\n', [], { stdin: 'abc' });
  for (const [i, argv] of [
    ['list', 'list'],
    ['missing', 'list'],
    ['list', 'missing'],
    ['list', '-'],
    ['-', 'list'],
    ['-', '-'],
  ].entries())
    add(`check-multiple-${i}`, ['-c', ...argv], {
      fixture: { ...fixture, files: { ...fixture.files, '/home/kali/list': good } },
      stdin: good,
    });
  const options = [
    ['-bt'],
    ['-tb'],
    ['-bbttb'],
    ['-w'],
    ['-q'],
    ['-s'],
    ['-h'],
    ['-v'],
    ['--b'],
    ['--t'],
    ['--sta'],
    ['--st'],
    ['--str'],
    ['--c'],
    ['--zero=x'],
    ['--tag', '--text'],
    ['--text', '--tag'],
    ['--tag', '--binary'],
    ['--binary', '--tag'],
    ['--check', '--zero'],
    ['-cz'],
    ['--check', '--tag'],
    ['-cb'],
    ['-ct'],
    ['-bct'],
    ['--status'],
    ['--quiet'],
    ['--strict'],
    ['--ignore-missing'],
    ['--warn'],
    ['--tag', '--text', '--check', '--zero'],
    ['--help', '--bad'],
    ['--bad', '--help'],
    ['--version', '--bad'],
    ['--bad', '--version'],
    ['--he'],
    ['--ver'],
    ['a', '-b'],
    ['--', 'a'],
    ['-é'],
  ];
  for (const [i, argv] of options.entries()) add(`options-${i}`, argv, { fixture });
  add('options-posix', ['a', '-b'], { fixture, env: { LC_ALL: 'C', POSIXLY_CORRECT: '1' } });
  add('invocation-bare', ['--help'], { invocation: 'sha256sum' });
  add('pipe-binary', [], { stdinHex: '00ff80'.repeat(8193), transport: 'pipe' });
  add('redirect-append', ['a'], {
    fixture: { ...fixture, files: { ...fixture.files, '/home/kali/sums': 'previous\n' } },
    io: { stdoutPath: '/home/kali/sums', append: true },
  });
  check('redirect-check', good, [], {
    fixture: { ...fixture, files: { ...fixture.files, '/home/kali/list': good } },
    io: { stdoutPath: '/home/kali/result' },
  });
  for (const mode of ['hash', 'check'])
    add(`tty-eof-${mode}`, mode === 'check' ? ['-c'] : [], {
      fixture,
      stdin: null,
      tty: { isTTY: true, columns: 80, rows: 24, ansiSupport: true, interactive: true },
      interaction: {
        schemaVersion: 1,
        steps: [
          { kind: 'write', hex: Buffer.from(mode === 'check' ? good : 'abc\n').toString('hex') },
          { kind: 'eof' },
        ],
      },
    });
  for (const signal of ['SIGINT', 'SIGTERM'])
    add(`tty-check-${signal.toLowerCase()}`, ['-c'], {
      stdin: null,
      tty: { isTTY: true, columns: 80, rows: 24, ansiSupport: true, interactive: true },
      interaction: { schemaVersion: 1, steps: [{ kind: 'signal', signal }] },
    });
  check('streaming-long-whitespace', ' '.repeat(20000) + good);
  check('streaming-long-malformed', 'x'.repeat(20000) + '\n' + good, ['--warn']);
  check('streaming-long-comment', '#'.repeat(20000) + '\n' + good);
  check('streaming-many-records', good.repeat(300));
  for (const byte of [1, 9, 13, 31, 127, 128, 255]) {
    const list = Buffer.concat([
      Buffer.from(digest + '  missing'),
      Buffer.from([byte]),
      Buffer.from('\n' + good),
    ]);
    add(`binary-list-${byte}`, ['-cw', 'list'], {
      fixture: { ...fixture, bytes: { '/home/kali/list': list.toString('hex') } },
    });
  }
  for (const [i, name] of names.entries())
    check(`escaping-missing-${i}`, record(0, i).replace(name, name + 'missing'));
  return cases;
}
