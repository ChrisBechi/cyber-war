import test from 'node:test';
import assert from 'node:assert/strict';
import { json } from './io.mjs';
import { teeRequests } from './tee-cases.mjs';
import { capturedExpectations } from './common-contracts.mjs';
import { referenceRequest, requestDigest, validateReference } from './coreutils-reference.mjs';
import { caseSources } from './fingerprint.mjs';
import { caseSchema } from './schema.mjs';

test('tee expectations require reproducible GNU and cover stream/error/signal contracts', () => {
  const requests = teeRequests();
  const capture = json('tests/cli/gnu/coreutils/9.7/tee.json');
  const declared = json('tests/cli/compat/pilot/coreutils-tee.json');
  assert.equal(validateReference({ command: 'tee', referenceVersion: '9.7' }, capture), null);
  assert.equal(new Set(requests.map((c) => c.id)).size, requests.length);
  assert.equal(declared.length, requests.length);
  for (const request of requests) {
    assert.equal(request.expected, undefined);
    const actual = declared.find((c) => c.id === request.id);
    caseSchema.parse(actual);
    const observed = capturedExpectations(request, capture);
    assert.equal(requestDigest(actual), requestDigest(request));
    for (const key of ['stdoutHex', 'stderrHex', 'exitCode', 'termination', 'observations'])
      assert.deepEqual(actual.expected[key], observed[key], request.id);
  }
  for (const field of ['harnessHash', 'interactionHash']) {
    const changed = structuredClone(capture);
    changed[field] = '0'.repeat(64);
    assert.ok(validateReference({ command: 'tee', referenceVersion: '9.7' }, changed));
  }
  const native = json('content/cli-compatibility/manifest.json').native.tee;
  for (const gate of ['TTY', 'SIGNALS', 'SIDE_EFFECTS', 'PERMISSIONS', 'PIPE', 'REDIRECTION']) {
    assert.equal(native.gates[gate].applicability, 'REQUIRED');
    assert.ok(native.gates[gate].testIds.length);
  }
});

test('tee request fingerprints bind output failures, alias identity and signal synchronization', () => {
  for (const [id, mutate] of [
    [
      'mode-3-closed',
      (c) => {
        c.io.closedConsumer = false;
      },
    ],
    [
      'large-alias-true',
      (c) => {
        c.fixture.hardlinks = {};
      },
    ],
    [
      'backpressure-int',
      (c) => {
        delete c.interaction.steps[0].waitFor;
      },
    ],
  ]) {
    const request = teeRequests().find((c) => c.id.endsWith('/' + id));
    const changed = structuredClone(request);
    mutate(changed);
    assert.notDeepEqual(referenceRequest(request), referenceRequest(changed));
    assert.notEqual(requestDigest(request), requestDigest(changed));
  }
  const paths = ['tee.rs', 'tee_tests.rs', 'cat.rs', 'base64.rs'].map(
    (p) => 'src-tauri/src/coreutils/' + p,
  );
  assert.deepEqual(caseSources(teeRequests()[0], paths), paths.slice(0, 3));
  const invalid = structuredClone(teeRequests().find((c) => c.id.endsWith('/backpressure-int')));
  invalid.interaction.schemaVersion = 2;
  assert.equal(
    caseSchema.omit({ expected: true, reference: true, gates: true }).safeParse(invalid).success,
    false,
  );
});
