import test from 'node:test';
import assert from 'node:assert/strict';
import { sha256sumRequests, sha256sumSeeds } from './sha256sum-cases.mjs';
import { capturedExpectations } from './common-contracts.mjs';
import { requestDigest, validateReference } from './coreutils-reference.mjs';
import { caseSources } from './fingerprint.mjs';
import { json } from './io.mjs';
import { caseSchema } from './schema.mjs';

test('sha256sum CLI expectations and roundtrip inputs are observed GNU bytes', () => {
  const requests = sha256sumRequests();
  const declared = json('tests/cli/compat/pilot/coreutils-sha256sum.json');
  const capture = json('tests/cli/gnu/coreutils/9.7/sha256sum.json');
  assert.equal(validateReference({ command: 'sha256sum', referenceVersion: '9.7' }, capture), null);
  assert.equal(new Set(requests.map((c) => c.id)).size, requests.length);
  assert.equal(declared.length, requests.length);
  for (const request of requests) {
    assert.equal(request.expected, undefined);
    const c = declared.find((c) => c.id === request.id);
    caseSchema.parse(c);
    assert.equal(requestDigest(c), requestDigest(request));
    const expected = capturedExpectations(request, capture);
    for (const field of ['stdoutHex', 'stderrHex', 'exitCode', 'termination', 'observations'])
      assert.deepEqual(c.expected[field], expected[field], c.id);
  }
  const inputs = json('tests/cli/fixtures/sha256sum-records.json');
  for (const seed of sha256sumSeeds())
    assert.equal(
      inputs.records[seed.id.split('/').at(-1)],
      capture.cases.find((c) => c.id === seed.id).stdoutHex,
    );
  const spec = json('content/cli-compatibility/coreutils.json').commands.sha256sum;
  for (const id of [
    'hashing',
    'checksum-output',
    'filename-escaping',
    'zero-output',
    'check-parser',
    'check-results',
    'malformed-input',
    'warn-strict',
    'stdin-check',
    'signals',
    'tty',
  ])
    assert.ok(spec.contracts.find((c) => c.id === id)?.evidence.length, id);
});

test('sha256sum request identity binds lists, bytes and flags; local leaves stay granular', () => {
  const request = sha256sumRequests().find((c) => c.id.endsWith('/check-valid-many'));
  for (const mutate of [
    (c) => {
      c.fixture.files['/home/kali/list'] += 'bad\n';
    },
    (c) => {
      c.argv.push('--strict');
    },
    (c) => {
      c.stdinHex = 'ff';
    },
  ]) {
    const changed = structuredClone(request);
    mutate(changed);
    assert.notEqual(requestDigest(changed), requestDigest(request));
  }
  const paths = [
    'src-tauri/src/coreutils/sha256sum.rs',
    'src-tauri/src/coreutils/sha256sum_tests.rs',
    'src-tauri/src/shell_pipeline/streams/sha256sum.rs',
    'src-tauri/src/coreutils/cat.rs',
  ];
  assert.deepEqual(caseSources(request, paths), paths);
  for (const command of ['cat', 'base64', 'tee', 'wc'])
    assert.deepEqual(caseSources({ ...request, command }, paths), [paths[3]]);
  const ref = json('tests/cli/gnu/coreutils/9.7/sha256sum.json');
  for (const field of ['harnessHash', 'interactionHash']) {
    const changed = structuredClone(ref);
    changed[field] = '0'.repeat(64);
    assert.ok(validateReference({ command: 'sha256sum', referenceVersion: '9.7' }, changed));
  }
});
