import test from 'node:test';
import assert from 'node:assert/strict';
import { filesystemRequests } from './filesystem-cases.mjs';
import { capturedExpectations } from './common-contracts.mjs';
import {
  requestDigest,
  referenceRequest,
  validateReference,
  referenceDifference,
} from './coreutils-reference.mjs';
import { json } from './io.mjs';
import { caseSchema } from './schema.mjs';
import { caseSources, caseFingerprint, caseFingerprints } from './fingerprint.mjs';
import { scope, differentialProviders } from './scope.mjs';

for (const command of ['readlink', 'realpath', 'mkdir', 'rmdir', 'ln'])
  test(`${command} filesystem expectations are individually captured GNU observations`, () => {
    const requests = filesystemRequests(command),
      declared = json(`tests/cli/compat/pilot/coreutils-${command}.json`),
      capture = json(`tests/cli/gnu/coreutils/9.7/${command}.json`);
    assert.equal(validateReference({ command, referenceVersion: '9.7' }, capture), null);
    assert.equal(new Set(requests.map((c) => c.id)).size, requests.length);
    assert.equal(declared.length, requests.length);
    for (const r of requests) {
      assert.equal(r.expected, undefined);
      const c = declared.find((c) => c.id === r.id);
      caseSchema.parse(c);
      assert.equal(requestDigest(c), requestDigest(r));
      const expected = capturedExpectations(r, capture);
      for (const key of ['stdoutHex', 'stderrHex', 'exitCode', 'termination', 'observations'])
        assert.deepEqual(c.expected[key], expected[key], c.id);
    }
    for (const contract of json('content/cli-compatibility/coreutils.json').commands[command]
      .contracts)
      assert.ok(contract.evidence.length, contract.id);
  });

test('filesystem differential detects unexpected parent changes and hardlink identity corruption', () => {
  const test = { command: 'mkdir', transport: 'direct' },
    row = {
      reproducible: true,
      request: referenceRequest(test),
      requestDigest: requestDigest(test),
      stdoutHex: '',
      stderrHex: '',
      exitCode: 0,
      termination: { kind: 'exit', code: 0 },
      observations: [],
      endpoints: { stdin: 'pipe', stdout: 'pipe', stderr: 'pipe' },
      rootLinkDelta: 1,
      before: {},
      after: { a: { type: 'directory', mode: 493, nlink: 2, inodeGroup: 0 } },
    },
    before = {
      vfs: { '/home/kali': { kind: 'directory', mode: 493, nlink: 4, ino: 1, content: '' } },
    },
    actual = {
      stdoutHex: '',
      stderrHex: '',
      exitCode: 0,
      termination: row.termination,
      observations: [],
      before,
      after: {
        vfs: {
          '/home/kali': { ...before.vfs['/home/kali'], nlink: 5 },
          '/home/kali/a': { kind: 'directory', mode: 493, nlink: 2, ino: 2, content: '' },
        },
      },
    };
  assert.equal(referenceDifference(test, actual, row), null);
  const changed = structuredClone(actual);
  changed.after.vfs['/home/kali'].nlink++;
  assert.match(referenceDifference(test, changed, row), /link-count delta|Unexpected project/);
  changed.after.vfs['/home/kali'].nlink--;
  changed.after.vfs['/home/kali'].mode = 511;
  assert.match(referenceDifference(test, changed, row), /Unexpected project/);
  row.after.b = { ...row.after.a, type: 'file', inodeGroup: 1, contentHex: '78', nlink: 2 };
  row.after.c = { ...row.after.b };
  for (const [n, ino] of [
    ['b', 3],
    ['c', 4],
  ])
    actual.after.vfs['/home/kali/' + n] = { kind: 'file', mode: 493, nlink: 2, ino, content: 'x' };
  actual.files = { '/home/kali/b': '78', '/home/kali/c': '78' };
  assert.match(referenceDifference(test, actual, row), /inode relationship/);
});

test('filesystem fingerprints retain shared VFS dependencies and focused runs cannot claim global scope', () => {
  const paths = [
    'src-tauri/src/vfs/canonical.rs',
    'src-tauri/src/coreutils/pathnames.rs',
    'src-tauri/src/coreutils/foundation.rs',
    'src-tauri/src/coreutils/sha256sum.rs',
  ];
  assert.deepEqual(
    caseSources({ softwareId: 'coreutils', command: 'readlink' }, paths),
    paths.slice(0, 3),
  );
  assert.deepEqual(caseSources({ softwareId: 'coreutils', command: 'sha256sum' }, paths), [
    paths[3],
  ]);
  assert.throws(() => scope(['--focused']), /requires --command/);
  const focused = scope(['--command', 'readlink', '--focused']);
  assert.equal(focused.focused, true);
  assert.deepEqual(focused.names, ['readlink']);
  assert.equal(focused.includes({ softwareId: 'coreutils', command: 'realpath' }), false);
});

test('batched fingerprints equal independently computed fingerprints with no persistent configuration cache', () => {
  const requests = [filesystemRequests('readlink')[0], filesystemRequests('realpath')[1]];
  const batched = caseFingerprints(requests);
  for (const r of requests) assert.equal(batched[r.id], caseFingerprint(r));
  const changed = structuredClone(requests[0]);
  changed.argv.push('--zero');
  assert.notEqual(caseFingerprints([changed])[changed.id], batched[changed.id]);
});

test('focused scope includes full GNU providers only for committed runtime dependencies', () => {
  const native = { ln: { runtimeRequirements: { TTY: ['TTY.CANONICAL_IO'] } }, mkdir: {} };
  const subsystems = [
    {
      capabilities: [
        { id: 'TTY.CANONICAL_IO', readiness: 'GNU_DIFFERENTIAL', requiredTests: ['tail/tty'] },
        { id: 'OTHER', readiness: 'GNU_DIFFERENTIAL', requiredTests: ['wc/input'] },
      ],
    },
  ];
  const cases = [
    { id: 'tail/tty', command: 'tail' },
    { id: 'tail/file', command: 'tail' },
    { id: 'wc/input', command: 'wc' },
  ];
  assert.deepEqual([...differentialProviders(['ln'], native, subsystems, cases)], ['tail']);
  assert.equal(differentialProviders(['mkdir'], native, subsystems, cases).size, 0);
});
