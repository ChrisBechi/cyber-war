import test from 'node:test';
import assert from 'node:assert/strict';
import { tailRequests } from './tail-cases.mjs';
import { tailFollowRequests } from './tail-follow-cases.mjs';
import { tailDirectedRequests } from './tail-directed-cases.mjs';
import { referenceRequest, requestDigest, validateReference } from './coreutils-reference.mjs';
import { json } from './io.mjs';
import { scanRuntime } from './host-guard.mjs';
import { caseSchema } from './schema.mjs';

test('tail canonical evidence pins its binary and both interaction harnesses', () => {
  const command = { command: 'tail', referenceVersion: '9.7' };
  const capture = json('tests/cli/gnu/coreutils/9.7/tail.json');
  assert.equal(validateReference(command, capture), null);
  for (const mutate of [
    (c) => (c.followHash = '0'.repeat(64)),
    (c) => (c.interactionHash = '0'.repeat(64)),
    (c) => (c.environment.binaryHashes.tail.sha256 = '0'.repeat(64)),
    (c) => delete c.environment.binaryHashes.tail,
    (c) => (c.environment.network = 'enabled'),
  ]) {
    const invalid = structuredClone(capture);
    mutate(invalid);
    assert.ok(validateReference(command, invalid));
  }
});

test('tail requests carry versioned follow operations and no invented expectations', () => {
  const requests = [...tailRequests(), ...tailFollowRequests(), ...tailDirectedRequests()];
  assert.equal(new Set(requests.map((c) => c.id)).size, requests.length);
  for (const row of requests) {
    assert.equal(row.command, 'tail');
    assert.equal(row.expected, undefined);
    assert.equal(row.env.LC_ALL, 'C');
    const result = caseSchema.omit({ expected: true, reference: true, gates: true }).safeParse(row);
    assert.equal(result.success, true, row.id);
  }
  const follow = requests.find((c) => c.id.endsWith('/follow-binary'));
  assert.equal(referenceRequest(follow).interaction.schemaVersion, 2);
  const changed = structuredClone(follow);
  changed.interaction.steps.find((s) => s.kind === 'mutate').hex = '00';
  assert.notEqual(requestDigest(follow), requestDigest(changed));
  const pid = requests.find((c) => c.id.endsWith('/follow-pid-single'));
  assert.deepEqual(referenceRequest(pid).interaction.writers, ['writer']);
});

test('directed follow barriers include the initial GNU output prefix', () => {
  const cases = json('tests/cli/compat/pilot/coreutils-tail.json');
  for (const request of tailDirectedRequests().filter((c) => c.id.includes('/directed-start-'))) {
    const declared = cases.find((c) => c.id === request.id);
    assert.equal(requestDigest(request), requestDigest(declared));
    const observed = declared.expected.observations;
    const barriers = request.interaction.steps.filter((s) => ['wait', 'await'].includes(s.kind));
    assert.equal(barriers.length, observed.length);
    for (const [index, barrier] of barriers.entries()) {
      if (barrier.kind === 'await')
        assert.equal(barrier.stdoutBytes, observed[index].stdoutHex.length / 2, request.id);
    }
  }
});

test('follow protocol rejects incomplete mutations and ambiguous barriers', () => {
  const schema = caseSchema.shape.interaction;
  const valid = {
    schemaVersion: 2,
    steps: [{ kind: 'mutate', operation: 'write', path: '/home/kali/log', hex: '00' }],
  };
  assert.equal(schema.safeParse(valid).success, true);
  for (const steps of [
    [{ kind: 'mutateBatch', steps: [{ kind: 'wait' }] }],
    [{ kind: 'mutateBatch', steps: [] }],
    [{ kind: 'mutate', operation: 'write', path: '/home/kali/log' }],
    [{ kind: 'mutate', operation: 'truncate', path: '/home/kali/log', hex: '00' }],
    [{ kind: 'mutate', operation: 'unlink', path: '/home/kali/log', size: 2 }],
    [
      {
        kind: 'mutate',
        operation: 'rename',
        path: '/home/kali/log',
        target: '/home/kali/../other',
      },
    ],
    [{ kind: 'mutate', operation: 'write', path: '/home/kali/../log', hex: '' }],
    [{ kind: 'await' }],
    [{ kind: 'write', hex: '00' }],
    [{ kind: 'stopWriter', writer: 'missing' }],
  ]) {
    assert.equal(
      schema.safeParse({ schemaVersion: 2, steps }).success,
      false,
      JSON.stringify(steps),
    );
  }
  assert.equal(schema.safeParse({ ...valid, schemaVersion: 1 }).success, false);
  assert.equal(schema.safeParse({ ...valid, writers: ['same', 'same'] }).success, false);
  assert.equal(
    schema.safeParse({
      schemaVersion: 2,
      writers: ['writer'],
      steps: [
        { kind: 'stopWriter', writer: 'writer' },
        { kind: 'stopWriter', writer: 'writer' },
      ],
    }).success,
    false,
  );
});

test('host guard rejects filesystem watcher APIs without expanding exceptions', () => {
  for (const code of [
    'notify::recommended_watcher(callback)',
    'inotify::Inotify::init()',
    'ReadDirectoryChangesW(handle)',
    'FSEventStreamCreate()',
    'kqueue()',
    'fs.watch(path)',
    "import chokidar from 'chokidar'",
  ]) {
    assert.equal(scanRuntime([['src-tauri/src/bad.rs', code]]).result, 'FAIL', code);
  }
  assert.equal(
    scanRuntime([['src-tauri/src/good.rs', 'world.vfs.subscribe(WatchTarget::Inode(17));']]).result,
    'PASS',
  );
});
