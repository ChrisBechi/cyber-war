import { request } from './common-contracts.mjs';
const hex = (text) => Buffer.from(text).toString('hex');
const wait = { kind: 'wait' };
const signal = { kind: 'signal', signal: 'SIGTERM' };
const append = (data) => ({
  kind: 'mutate',
  operation: 'append',
  path: '/home/kali/log',
  hex: hex(data),
});
export function tailDirectedRequests() {
  const cases = [];
  for (const [id, args, initial, parts] of [
    ['lines', ['-n+5'], 'a\n', ['b\n', 'c\nd\ne\n', 'f\n']],
    ['partial', ['-n+5'], 'a', ['b', '\nc\nd\ne\nf\n', 'g\n']],
    ['bytes', ['-c+8'], 'ab', ['cd', 'efghij', 'kl']],
    ['binary', ['-c+8'], '\0a', ['\0b', '\0c\0d\0e', '\0f']],
    ['nul', ['-zn+5'], 'a\0', ['b\0', 'c\0d\0e\0', 'f\0']],
  ]) {
    // GNU's +byte follow probes first emit the short initial file after the
    // seek/truncation diagnostic. Include that already-observed prefix.
    let observedBytes = args[0].startsWith('-c') ? Buffer.byteLength(initial) : 0;
    cases.push(
      request('tail', 'directed-start-' + id, [...args, '-f', 'log'], {
        stdin: null,
        fixture: { bytes: { '/home/kali/log': hex(initial) } },
        interaction: {
          schemaVersion: 2,
          // The published GNU capture emits every appended byte in these
          // cases. Wait for that output: a kernel wait alone may still be the
          // old inotify read, before the append has been consumed.
          steps: [
            wait,
            ...parts.flatMap((data) => {
              observedBytes += Buffer.byteLength(data);
              return [append(data), { kind: 'await', stdoutBytes: observedBytes }];
            }),
            signal,
          ],
        },
      }),
    );
  }
  const tty = { isTTY: true, columns: 80, rows: 24, ansiSupport: true, interactive: true };
  cases.push(
    request('tail', 'directed-tty-repeated-eof', ['-n1', '-f'], {
      stdin: null,
      tty,
      interaction: {
        schemaVersion: 1,
        steps: [
          { kind: 'write', hex: hex('a\n') },
          { kind: 'eof' },
          { kind: 'expect', stdoutHex: hex('a\n') },
          { kind: 'write', hex: hex('b\n') },
          { kind: 'expect', stdoutHex: hex('a\nb\n') },
          { kind: 'eof' },
          { kind: 'write', hex: hex('c\n') },
          { kind: 'expect', stdoutHex: hex('a\nb\nc\n') },
          signal,
        ],
      },
    }),
  );
  for (const size of [2, 9])
    cases.push(
      request('tail', 'directed-truncate-batch-' + size, ['-c2', '-f', 'log'], {
        stdin: null,
        fixture: { files: { '/home/kali/log': 'old-data' } },
        interaction: {
          schemaVersion: 2,
          steps: [
            { kind: 'await', stdoutBytes: 2 },
            wait,
            {
              kind: 'mutateBatch',
              steps: [
                { kind: 'mutate', operation: 'truncate', path: '/home/kali/log', size: 0 },
                append('x'.repeat(size)),
              ],
            },
            { kind: 'await', stdoutBytes: 3 },
            wait,
            signal,
          ],
        },
      }),
    );
  cases.push(
    request('tail', 'directed-headers-aba', ['-n0', '-f', 'a', 'b'], {
      stdin: null,
      fixture: { files: { '/home/kali/a': '', '/home/kali/b': '' } },
      interaction: {
        schemaVersion: 2,
        steps: [
          { kind: 'await', stdoutBytes: 21 },
          wait,
          ...['a', 'b', 'a'].flatMap((name, index) => [
            { kind: 'mutate', operation: 'append', path: '/home/kali/' + name, hex: hex('x\n') },
            { kind: 'await', stdoutBytes: 34 + index * 13 },
            wait,
          ]),
          signal,
        ],
      },
    }),
  );
  for (const interval of ['.01', '.1'])
    cases.push(
      request(
        'tail',
        'directed-writers-interval-' + interval.slice(1),
        [
          '-n0',
          '-f',
          '-s' + interval,
          '--max-unchanged-stats=1',
          '--pid={writer}',
          '--pid={second}',
          'log',
        ],
        {
          stdin: null,
          fixture: { files: { '/home/kali/log': '' } },
          interaction: {
            schemaVersion: 2,
            writers: ['writer', 'second'],
            steps: [
              wait,
              append('a\n'),
              { kind: 'await', stdoutBytes: 2 },
              wait,
              { kind: 'stopWriter', writer: 'writer', signal: 'SIGTERM' },
              append('b\n'),
              { kind: 'await', stdoutBytes: 4 },
              wait,
              { kind: 'stopWriter', writer: 'second', signal: 'SIGINT' },
            ],
          },
        },
      ),
    );
  return cases;
}
