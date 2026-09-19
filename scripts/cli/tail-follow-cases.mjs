import { request } from './common-contracts.mjs';
const hex = (s) => Buffer.from(s).toString('hex');
const wait = { kind: 'wait' };
const end = { kind: 'signal', signal: 'SIGTERM' };
const awaitBytes = (stdoutBytes) => ({ kind: 'await', stdoutBytes });
const mutate = (operation, name, extra = {}) => ({
  kind: 'mutate',
  operation,
  path: '/home/kali/' + name,
  ...extra,
});
const append = (name, s) => mutate('append', name, { hex: hex(s) });
const write = (name, s) => mutate('write', name, { hex: hex(s) });
const rename = (name, target) => mutate('rename', name, { target: '/home/kali/' + target });

export function tailFollowRequests() {
  const cases = [];
  const add = (id, argv, steps, extra = {}) =>
    cases.push(
      request('tail', 'follow-' + id, argv, {
        stdin: null,
        fixture: { files: { '/home/kali/log': 'old\n' } },
        interaction: { schemaVersion: 2, steps },
        ...extra,
      }),
    );
  for (const follow of ['-f', '--follow=descriptor', '--follow=name', '-F']) {
    const id = follow === '-F' ? 'name-retry-short' : follow.replaceAll(/[^a-z]/g, '');
    add(
      id + '-append',
      ['-n1', follow, 'log'],
      [awaitBytes(4), wait, append('log', 'new\n'), awaitBytes(8), wait, end],
    );
    add(
      id + '-truncate',
      ['-c2', follow, 'log'],
      [
        awaitBytes(2),
        wait,
        mutate('truncate', 'log', { size: 0 }),
        { kind: 'await', stderrBytes: 1 },
        wait,
        append('log', 'x\n'),
        awaitBytes(4),
        wait,
        end,
      ],
    );
  }
  add(
    'descriptor-rename',
    ['-n1', '-f', 'log'],
    [
      awaitBytes(4),
      wait,
      rename('log', 'old'),
      write('log', 'replacement\n'),
      append('old', 'kept\n'),
      awaitBytes(9),
      wait,
      end,
    ],
  );
  add(
    'name-replace',
    ['-n1', '--follow=name', 'log'],
    [
      awaitBytes(4),
      wait,
      write('temp', 'new\n'),
      rename('temp', 'log'),
      awaitBytes(8),
      wait,
      append('log', 'last\n'),
      awaitBytes(13),
      wait,
      end,
    ],
  );
  add(
    'retry-create',
    ['-n1', '-F', '--sleep-interval=.01', 'log'],
    [{ kind: 'await', stderrBytes: 1 }, wait, write('log', 'created\n'), awaitBytes(8), wait, end],
    { fixture: { files: {} } },
  );
  add(
    'retry-recreate',
    ['-n1', '-F', '--sleep-interval=.01', 'log'],
    [
      awaitBytes(4),
      wait,
      mutate('unlink', 'log'),
      { kind: 'await', stderrBytes: 1 },
      wait,
      write('log', 'again\n'),
      awaitBytes(10),
      wait,
      end,
    ],
  );
  add(
    'hardlink',
    ['-n1', '-f', 'log'],
    [awaitBytes(4), wait, append('alias', 'alias\n'), awaitBytes(10), wait, end],
    {
      fixture: {
        files: { '/home/kali/log': 'old\n' },
        hardlinks: { '/home/kali/alias': '/home/kali/log' },
      },
    },
  );
  add(
    'stdin-file',
    ['-n1', '-f'],
    [awaitBytes(4), wait, append('log', 'new\n'), awaitBytes(8), wait, end],
    { io: { stdinPath: '/home/kali/log' } },
  );
  add(
    'nul',
    ['-zn1', '-f', 'log'],
    [awaitBytes(2), wait, append('log', 'c\0'), awaitBytes(4), wait, end],
    {
      fixture: { bytes: { '/home/kali/log': hex('a\0b\0') } },
    },
  );
  add(
    'binary',
    ['-c2', '-f', 'log'],
    [awaitBytes(2), wait, mutate('append', 'log', { hex: '00ff800a' }), awaitBytes(6), wait, end],
    {
      fixture: { bytes: { '/home/kali/log': '00ff800a' } },
    },
  );
  add(
    'long-interval-event',
    ['-n1', '-f', '--sleep-interval=9999', '--max-unchanged-stats=9999', 'log'],
    [awaitBytes(4), wait, append('log', 'new\n'), awaitBytes(8), wait, end],
  );
  add('sigint', ['-n1', '-f', 'log'], [awaitBytes(4), wait, { kind: 'signal', signal: 'SIGINT' }]);
  const multi = { files: { '/home/kali/log': 'old\n', '/home/kali/other': 'two\n' } };
  const first = '==> log <==\nold\n\n==> other <==\ntwo\n';
  const second = first + '\n==> log <==\nnew\n';
  add(
    'multiple',
    ['-n1', '-f', 'log', 'other'],
    [
      awaitBytes(first.length),
      wait,
      append('log', 'new\n'),
      awaitBytes(second.length),
      wait,
      append('other', 'last\n'),
      awaitBytes((second + '\n==> other <==\nlast\n').length),
      wait,
      end,
    ],
    { fixture: multi },
  );
  add(
    'descriptor-unlink',
    ['-n1', '-f', 'log'],
    [
      awaitBytes(4),
      wait,
      mutate('unlink', 'log'),
      append('alias', 'kept\n'),
      awaitBytes(9),
      wait,
      end,
    ],
    {
      fixture: {
        files: { '/home/kali/log': 'old\n' },
        hardlinks: { '/home/kali/alias': '/home/kali/log' },
      },
    },
  );
  for (const mode of ['descriptor', 'name']) {
    add(
      'symlink-' + mode,
      ['-n1', '--follow=' + mode, '-s.01', '--max-unchanged-stats=1', 'link'],
      [
        awaitBytes(4),
        wait,
        append('log', 'new\n'),
        awaitBytes(8),
        wait,
        write('replacement', 'replaced\n'),
        mutate('symlink', 'newlink', { target: 'replacement' }),
        rename('newlink', 'link'),
        ...(mode === 'name' ? [awaitBytes(17)] : [append('log', 'kept\n'), awaitBytes(13)]),
        wait,
        end,
      ],
      {
        fixture: { files: { '/home/kali/log': 'old\n' }, symlinks: { '/home/kali/link': 'log' } },
      },
    );
  }
  add(
    'permission-retry',
    ['-n1', '-F', '-s.01', 'log'],
    [
      { kind: 'await', stderrBytes: 1 },
      wait,
      mutate('chmod', 'log', { mode: 420 }),
      awaitBytes(4),
      wait,
      end,
    ],
    {
      fixture: { files: { '/home/kali/log': 'old\n' }, modes: { '/home/kali/log': 0 } },
    },
  );
  add(
    'descriptor-permission',
    ['-n1', '-f', 'log'],
    [
      awaitBytes(4),
      wait,
      mutate('chmod', 'log', { mode: 128 }),
      append('alias', 'new\n'),
      awaitBytes(8),
      wait,
      end,
    ],
    {
      fixture: {
        files: { '/home/kali/log': 'old\n' },
        hardlinks: { '/home/kali/alias': '/home/kali/log' },
      },
    },
  );
  add(
    'remove-no-retry',
    ['-n1', '--follow=name', 'log'],
    [awaitBytes(4), wait, mutate('unlink', 'log')],
  );
  for (const both of [false, true]) {
    const writers = both ? ['writer', 'second'] : ['writer'];
    add(
      'pid-' + (both ? 'multiple' : 'single'),
      ['-n1', '-f', '-s.01', ...writers.map((w) => '--pid={' + w + '}'), 'log'],
      [],
      {
        interaction: {
          schemaVersion: 2,
          writers,
          steps: [
            awaitBytes(4),
            wait,
            append('log', 'new\n'),
            awaitBytes(8),
            wait,
            { kind: 'stopWriter', writer: 'writer' },
            ...(both
              ? [
                  append('log', 'still\n'),
                  awaitBytes(14),
                  wait,
                  { kind: 'stopWriter', writer: 'second' },
                ]
              : []),
          ],
        },
      },
    );
  }
  return cases;
}
