import test from 'node:test';
import assert from 'node:assert/strict';
import { json } from './io.mjs';
import { wcRequests } from './wc-cases.mjs';
import { capturedExpectations } from './common-contracts.mjs';
import { requestDigest, validateReference, verificationFresh } from './coreutils-reference.mjs';
import { caseSources } from './fingerprint.mjs';
import { caseSchema } from './schema.mjs';

test('wc canonical expectations come exclusively from reproducible locked GNU observations', () => {
  const requests = wcRequests(),
    declared = json('tests/cli/compat/pilot/coreutils-wc.json');
  const capture = json('tests/cli/gnu/coreutils/9.7/wc.json');
  assert.equal(validateReference({ command: 'wc', referenceVersion: '9.7' }, capture), null);
  assert.equal(new Set(requests.map((c) => c.id)).size, requests.length);
  assert.equal(declared.length, requests.length);
  for (const request of requests) {
    assert.equal(request.expected, undefined);
    assert.equal(request.env.LC_ALL, 'C');
    const c = declared.find((c) => c.id === request.id);
    caseSchema.parse(c);
    assert.equal(requestDigest(c), requestDigest(request));
    const expected = capturedExpectations(request, capture);
    for (const field of ['stdoutHex', 'stderrHex', 'exitCode', 'termination', 'observations'])
      assert.deepEqual(c.expected[field], expected[field], c.id);
  }
  for (const field of ['harnessHash', 'interactionHash']) {
    const changed = structuredClone(capture);
    changed[field] = '0'.repeat(64);
    assert.ok(validateReference({ command: 'wc', referenceVersion: '9.7' }, changed));
  }
  const spec = json('content/cli-compatibility/coreutils.json').commands.wc;
  for (const id of [
    'counting',
    'word-classification',
    'display-width',
    'formatting',
    'totals',
    'files0-from',
    'signals',
    'tty',
  ])
    assert.ok(spec.contracts.find((c) => c.id === id)?.evidence.length);
});

test('wc fingerprints bind list bytes and isolate counters from sibling byte handlers', () => {
  const c = wcRequests().find((c) => c.id.endsWith('/files0-file-odd'));
  const changed = structuredClone(c);
  changed.fixture.files['/home/kali/list'] += 'a\0';
  assert.notEqual(requestDigest(c), requestDigest(changed));
  const paths = [
    'src-tauri/src/coreutils/wc.rs',
    'src-tauri/src/coreutils/wc_tests.rs',
    'src-tauri/src/shell_pipeline/streams/wc.rs',
    'src-tauri/src/coreutils/cat.rs',
  ];
  assert.deepEqual(caseSources(c, paths), paths);
  for (const command of ['cat', 'base64', 'tee'])
    assert.deepEqual(caseSources({ ...c, command }, paths), [paths[3]]);
});

test('verified provenance cannot refresh changed golden observations or identities', () => {
  for (const command of [
    'basename',
    'dirname',
    'printenv',
    'whoami',
    'cat',
    'head',
    'tail',
    'base64',
    'tee',
  ]) {
    const capture = json(`tests/cli/gnu/coreutils/9.7/${command}.json`);
    assert.equal(verificationFresh(command, capture), true, command);
    for (const mutate of [
      (c) => {
        c.cases[0].stdoutHex += '00';
      },
      (c) => {
        c.environment.binaryHashes[command].sha256 = '0'.repeat(64);
      },
      (c) => {
        c.cases[0].request.argv.push('tampered');
      },
    ]) {
      const changed = structuredClone(capture);
      mutate(changed);
      assert.equal(verificationFresh(command, changed), false, command);
      assert.ok(validateReference({ command, referenceVersion: '9.7' }, changed));
    }
  }
});
