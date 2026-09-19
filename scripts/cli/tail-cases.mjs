// Requests only. Expected bytes are adopted from the locked GNU executable.
import { headRequests } from './head-cases.mjs';
import { request } from './common-contracts.mjs';

export function tailRequests() {
  const cases = headRequests()
    .filter((c) => !c.interaction && !c.id.includes('/self-'))
    .map((c) => ({
      ...c,
      id: c.id.replaceAll('head/', 'tail/'),
      command: 'tail',
      invocation: '/usr/bin/tail',
    }));
  const add = (id, argv, extra = {}) => cases.push(request('tail', id, argv, extra));
  for (const [i, argv] of [
    ['+2'],
    ['+2c'],
    ['-2f', '--help'],
    ['--follow=bad'],
    ['--follow='],
    ['--follow=d', '--help'],
    ['--follow=n', '--help'],
    ['--follow=descriptor', 'missing'],
    ['--follow=name', 'missing'],
    ['--retry'],
    ['--retry', 'missing'],
    ['--sleep-interval=bad'],
    ['--sleep-interval=-1'],
    ['--sleep-interval=nan'],
    ['--sleep-interval=inf'],
    ['--sleep-interval=0'],
    ['--sleep-interval=.1'],
    ['--sleep-interval=1e2'],
    ['--sleep-interval='],
    ['-s'],
    ['--max-unchanged-stats=0'],
    ['--max-unchanged-stats=bad'],
    ['--max-unchanged-stats=-1'],
    ['--max-unchanged-stats=18446744073709551616'],
    ['--pid=bad'],
    ['--pid=-1'],
    ['--pid=0'],
    ['--pid=2147483648'],
    ['--p'],
    ['--s'],
    ['-F', '--help'],
    ['-f', '--help'],
    ['-n', '+0'],
    ['-c', '+0'],
    ['-n', '-+2'],
    ['-n', ' +2'],
    ['+2', 'a'],
    ['a', '+2'],
    ['-2v'],
    ['-2q'],
    ['-2z'],
    ['-2b'],
    ['-2l'],
  ].entries())
    add(`tail-parse-${i}`, argv);
  const tty = { isTTY: true, columns: 80, rows: 24, ansiSupport: true, interactive: true };
  const write = (text) => ({ kind: 'write', hex: Buffer.from(text).toString('hex') });
  for (const [id, argv, steps] of [
    ['tty-lines', ['-n2'], [write('a\nb\nc\n'), { kind: 'eof' }]],
    ['tty-bytes', ['-c3'], [write('abcd\n'), { kind: 'eof' }]],
    ['tty-start-lines', ['-n+2'], [write('a\nb\nc\n'), { kind: 'eof' }]],
    ['tty-start-bytes', ['-c+2'], [write('abcd\n'), { kind: 'eof' }]],
    [
      'tty-follow',
      ['-n1', '-f'],
      [
        write('a\n'),
        { kind: 'eof' },
        { kind: 'expect', stdoutHex: '610a' },
        write('b\n'),
        { kind: 'expect', stdoutHex: '610a620a' },
        { kind: 'signal', signal: 'SIGTERM' },
      ],
    ],
    ...['SIGINT', 'SIGTERM'].map((signal) => [
      `tty-${signal.toLowerCase()}`,
      [],
      [{ kind: 'signal', signal }],
    ]),
  ])
    add(id, argv, { stdin: null, tty, interaction: { schemaVersion: 1, steps } });
  for (const [i, argv] of [
    ['-n1', 'a'],
    ['-c1', 'a'],
    ['-n+2', 'a'],
    ['-n0', '-f', 'a'],
    ['-n0', '--follow=name', 'a'],
    ['-f', '--pid=2147483647', 'a'],
    ['--pid=2147483647', '-s.01', '-f', 'a'],
    ['-n1', '-f'],
    ['-n1', '-f', '-'],
    ['--follow=name', '-'],
    ['--retry', '-f'],
    ['-2', 'a', 'b'],
    ['+2', 'a', 'b'],
    ['-2', '-'],
    ['+2', '--help'],
  ].entries())
    add(`extended-${i}`, argv, {
      fixture: { files: { '/home/kali/a': 'first\nlast\n', '/home/kali/b': 'other\n' } },
      ...(i < 5 ? { io: { stdoutPath: '/home/kali/a', append: true } } : {}),
      ...(i === 3 || i === 4
        ? {
            stdin: null,
            interaction: {
              schemaVersion: 2,
              steps: [{ kind: 'wait' }, { kind: 'signal', signal: 'SIGTERM' }],
            },
          }
        : {}),
    });
  for (const [i, flag] of ['-n+0', '-n+1', '-n+2', '-n+3', '-c+2', '-n+300'].entries())
    add(`self-start-${i}`, [flag, 'a'], {
      fixture: { files: { '/home/kali/a': 'first\nlast\n' } },
      io: { stdoutPath: '/home/kali/a', append: true },
    });
  add('follow-closed-empty', ['-n0', '-f', 'a'], {
    fixture: { files: { '/home/kali/a': 'x\n' } },
    io: { closedConsumer: true },
  });
  return cases;
}
