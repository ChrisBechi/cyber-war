import test from 'node:test';
import assert from 'node:assert/strict';
import { json, evidenceSources } from './io.mjs';
import { base64Requests } from './base64-cases.mjs';
import { capturedExpectations } from './common-contracts.mjs';
import { requestDigest, validateReference } from './coreutils-reference.mjs';
import { caseSources } from './fingerprint.mjs';

test('base64 expectations and interactions are bound to the locked GNU executable', () => {
  const capture = json('tests/cli/gnu/coreutils/9.7/base64.json');
  const command = { command: 'base64', referenceVersion: '9.7' };
  assert.equal(validateReference(command, capture), null);
  const requests = base64Requests();
  const cases = json('tests/cli/compat/pilot/coreutils-base64.json');
  assert.equal(new Set(requests.map((c) => c.id)).size, requests.length);
  assert.equal(cases.length, requests.length);
  for (const request of requests) {
    assert.equal(request.expected, undefined);
    const declared = cases.find((c) => c.id === request.id);
    assert.equal(requestDigest(request), requestDigest(declared));
    const observed = capturedExpectations(request, capture);
    for (const key of ['stdoutHex', 'stderrHex', 'exitCode', 'termination', 'observations'])
      assert.deepEqual(declared.expected[key], observed[key], request.id);
  }
  for (const mutate of [
    (c) => (c.interactionHash = '0'.repeat(64)),
    (c) => (c.environment.binaryHashes.base64.sha256 = '0'.repeat(64)),
    (c) => (c.provenance = 'GNU_PROBE'),
  ]) {
    const changed = structuredClone(capture);
    mutate(changed);
    assert.ok(validateReference(command, changed));
  }
  const changed = structuredClone(requests.find((c) => c.id.includes('interrupt')));
  const original = requestDigest(changed);
  changed.interaction.steps.at(-1).signal = 'SIGTERM';
  assert.notEqual(requestDigest(changed), original);
});

test('base64 fingerprints retain its stream dispatcher and exclude unrelated leaf implementations', () => {
  assert.ok(
    evidenceSources().every((path) => !path.includes('/__pycache__/') && !path.endsWith('.pyc')),
  );
  const paths = ['base64.rs', 'base64_tests.rs', 'cat.rs', 'head.rs'].map(
    (p) => 'src-tauri/src/coreutils/' + p,
  );
  const included = caseSources(base64Requests()[0], paths);
  assert.deepEqual(included, paths.slice(0, 3));
  const native = json('content/cli-compatibility/manifest.json').native.base64;
  assert.equal(native.resultContract, 'STRUCTURED');
  for (const gate of ['TTY', 'SIGNALS', 'SIDE_EFFECTS', 'PIPE', 'REDIRECTION']) {
    assert.equal(native.gates[gate].applicability, 'REQUIRED');
    assert.ok(native.gates[gate].testIds.length);
  }
});
