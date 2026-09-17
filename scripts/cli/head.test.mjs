import test from 'node:test';
import assert from 'node:assert/strict';
import { json } from './io.mjs';
import { headRequests } from './head-cases.mjs';
import { capturedExpectations, commonContracts } from './common-contracts.mjs';
import { requestDigest, validateReference } from './coreutils-reference.mjs';
import { waveDecision } from './coreutils-wave.mjs';

test('generated requests preserve case IDs and bind every expectation to reproducible GNU bytes', () => {
  const requests = headRequests();
  const capture = json('tests/cli/gnu/coreutils/9.7/head.json');
  const cases = json('tests/cli/compat/pilot/coreutils-head.json');
  assert.equal(validateReference({ command: 'head', referenceVersion: '9.7' }, capture), null);
  assert.equal(new Set(requests.map((c) => c.id)).size, requests.length);
  assert.equal(cases.length, requests.length);
  for (const request of requests) {
    const declared = cases.find((c) => c.id === request.id);
    assert.equal(requestDigest(request), requestDigest(declared));
    const expected = capturedExpectations(request, capture);
    for (const key of ['stdoutHex', 'stderrHex', 'exitCode', 'termination', 'observations'])
      assert.deepEqual(expected[key], declared.expected[key]);
  }
  assert.throws(
    () => capturedExpectations({ ...requests[0], argv: ['changed'] }, capture),
    /mismatch/,
  );
  assert.throws(
    () => capturedExpectations(requests[0], { ...capture, provenance: 'DECLARED_EXPECTATIONS' }),
    /incompatible/,
  );
  assert.throws(
    () => capturedExpectations(requests[0], { ...capture, harnessHash: '0'.repeat(64) }),
    /Stale/,
  );
  assert.throws(() => capturedExpectations(requests[0], { ...capture, cases: [] }), /mismatch/);
});

test('common contracts generate independent requests without inventing expected output', () => {
  for (const command of ['head', 'cat', 'tail', 'wc']) {
    const cases = commonContracts(command);
    assert.ok(cases.length >= 12);
    for (const c of cases) {
      assert.equal(c.command, command);
      assert.equal(c.env.LC_ALL, 'C');
      assert.equal(c.expected, undefined);
    }
  }
});

test('wave selection follows exact queue, refuses skipping and respects architecture/count boundaries', () => {
  const head = { command: 'head', area: 'reading', blockers: ['GAP:count multipliers'] };
  assert.equal(waveDecision({ next: head }).classification, 'SMALL_SHARED_EXTENSION');
  const same = { ...head, command: 'wc', blockers: [] };
  assert.equal(waveDecision({ next: same }, ['head']).classification, 'SAME_WAVE');
  const tail = {
    ...head,
    command: 'tail',
    blockers: ['GAP:Follow/retry requires process/filesystem notifications'],
  };
  const decision = waveDecision({ next: tail }, ['head']);
  assert.equal(decision.next, tail);
  assert.equal(decision.classification, 'ARCHITECTURAL_BOUNDARY');
  assert.equal(decision.continue, false);
  assert.equal(
    waveDecision({ next: { ...head, area: 'filesystem' } }, ['head']).classification,
    'OUT_OF_SCOPE_FAMILY',
  );
  assert.equal(
    waveDecision({ next: head }, ['head', ...Array.from({ length: 11 }, (_, i) => 'cmd' + i)])
      .stopReason,
    'EXECUTABLE_LIMIT',
  );
  assert.throws(() => waveDecision({ next: head }, ['wc']), /begin with head/);
  assert.throws(() => waveDecision({ next: head }, ['head', 'head']), /count/);
});
