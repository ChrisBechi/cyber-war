import test from 'node:test';
import assert from 'node:assert/strict';
import { json } from './io.mjs';
import {
  referenceRequest,
  requestDigest,
  validateReference,
  referenceDifference,
} from './coreutils-reference.mjs';
import { caseSources } from './fingerprint.mjs';
import { compareCase } from './matchers.mjs';
import { scope } from './scope.mjs';
import { coreutilsGate } from './coreutils.mjs';

test('GNU evidence refuses wrong provenance, baseline, stale harness and changed input', () => {
  const command = {
    command: 'basename',
    referenceVersion: json('content/cli-compatibility/manifest.json').software.coreutils
      .referenceVersion,
  };
  const capture = json(`tests/cli/gnu/coreutils/${command.referenceVersion}/basename.json`);
  assert.equal(validateReference(command, capture), null);
  for (const mutate of [
    (c) => (c.provenance = 'DECLARED_EXPECTATIONS'),
    (c) => (c.version = '8.32'),
    (c) => (c.harnessHash = '0'.repeat(64)),
    (c) => (c.environment.lockHash = '0'.repeat(64)),
    (c) => (c.environment.binaryHashes.basename.sha256 = 'invalid'),
    (c) => (c.environment.binaryHashes.basename.sha256 = '1'.repeat(64)),
    (c) => (c.environment.network = 'enabled'),
    (c) => c.cases.push(c.cases[0]),
  ]) {
    const invalid = structuredClone(capture);
    mutate(invalid);
    assert.ok(validateReference(command, invalid));
  }
  const row = capture.cases[0],
    input = row.request;
  const actual = { ...row, before: { vfs: {} }, after: { vfs: {} } };
  const commandWithContract = {
    ...command,
    coreutils: { area: 'foundation', contracts: [], flags: [] },
  };
  const byCase = new Map([[input.id, input]]),
    results = new Map([[input.id, { result: 'PASS' }]]);
  assert.equal(
    coreutilsGate(
      commandWithContract,
      'GNU_REFERENCE',
      byCase,
      results,
      new Map([[input.id, actual]]),
    ).result,
    'PASS',
  );
  assert.equal(
    coreutilsGate(
      commandWithContract,
      'GNU_REFERENCE',
      byCase,
      results,
      new Map([[input.id, { ...actual, stdoutHex: 'ff' }]]),
    ).result,
    'FAIL',
  );
  assert.equal(referenceDifference(input, actual, row), null);
  assert.match(
    referenceDifference({ ...input, script: 'echo hidden' }, actual, row),
    /does not cover/,
  );
  assert.notEqual(requestDigest({ ...input, argv: [...input.argv, 'changed'] }), row.requestDigest);
  assert.match(
    referenceDifference({ ...input, env: { ...input.env, NEW: 'value' } }, actual, row),
    /fingerprint/,
  );
  assert.match(referenceDifference(input, { ...actual, stderrHex: 'ff' }, row), /stderr byte/);
  assert.match(referenceDifference(input, { ...actual, exitCode: 99 }, row), /status/);
  assert.match(
    referenceDifference(
      input,
      { ...actual, after: { vfs: { '/unexpected': { content: 'x' } } } },
      row,
    ),
    /filesystem mutation/,
  );
  assert.equal(referenceRequest(input).stdinHex, input.stdinHex);
});

test('stderr comparison is byte exact even when lossy display is identical', () => {
  const expected = {
    stdout: { kind: 'EXACT', value: '' },
    stderr: { kind: 'EXACT', value: '�' },
    stderrHex: 'c3',
    exitCode: 1,
    state: [],
  };
  assert.equal(
    compareCase(
      { id: 'bytes', expected },
      { stdout: '', stderr: '�', stderrHex: 'c3', exitCode: 1 },
    ).result,
    'PASS',
  );
  assert.equal(
    compareCase(
      { id: 'bytes', expected },
      { stdout: '', stderr: '�', stderrHex: 'efbfbd', exitCode: 1 },
    ).result,
    'FAIL',
  );
});

test('scoped fingerprints preserve unrelated handlers and bind shared dependencies', () => {
  const files = [
    'src-tauri/src/coreutils/foundation.rs',
    'src-tauri/src/coreutils/foundation_options.rs',
    'src-tauri/src/coreutils/messages/basename-help.txt',
    'src-tauri/src/coreutils/messages/dirname-help.txt',
    'src-tauri/src/terminal_transfer.rs',
    'src-tauri/src/shell_pipeline.rs',
    'src-tauri/src/vfs.rs',
  ];
  const select = (command) => caseSources({ softwareId: 'coreutils', command }, files);
  assert.ok(select('basename').includes(files[2]));
  assert.ok(!select('dirname').includes(files[2]));
  assert.ok(!select('cp').includes(files[0]));
  assert.ok(!select('cp').includes(files[1]));
  for (const name of ['basename', 'dirname', 'cp'])
    for (const shared of files.slice(5)) assert.ok(select(name).includes(shared));
  assert.deepEqual(scope(['--wave', 'foundation']).names.sort(), [
    'basename',
    'dirname',
    'printenv',
    'whoami',
  ]);
  assert.throws(() => scope(['--command', 'basename', '--wave', 'foundation']));
});

test('cat GNU evidence binds PTY descriptors, signal termination and observation barriers', () => {
  const command = { command: 'cat', referenceVersion: '9.7' };
  const capture = json('tests/cli/gnu/coreutils/9.7/cat.json');
  assert.equal(validateReference(command, capture), null);
  const stale = structuredClone(capture);
  stale.interactionHash = '0'.repeat(64);
  assert.match(validateReference(command, stale), /interaction/);
  const testCase = json('tests/cli/compat/pilot/coreutils-cat.json').find(
    (c) => c.id === 'coreutils/cat/tty-sigint',
  );
  const row = capture.cases.find((r) => r.id === testCase.id);
  const actual = {
    stdoutHex: row.stdoutHex,
    stderrHex: row.stderrHex,
    exitCode: row.exitCode,
    termination: row.termination,
    observations: row.observations,
    before: { vfs: {} },
    after: { vfs: {} },
    files: {},
  };
  assert.equal(referenceDifference(testCase, actual, row), null);
  assert.match(
    referenceDifference(testCase, { ...actual, termination: { kind: 'exit', code: 130 } }, row),
    /termination/,
  );
  assert.match(referenceDifference(testCase, { ...actual, observations: [] }, row), /observations/);
  assert.match(
    referenceDifference(testCase, actual, {
      ...row,
      endpoints: { ...row.endpoints, stdin: 'pipe' },
    }),
    /descriptor/,
  );
  assert.match(
    referenceDifference(testCase, actual, { ...row, termination: undefined }),
    /termination/,
  );
  assert.notEqual(
    compareCase(testCase, { ...actual, termination: { kind: 'exit', code: 130 } }).result,
    'PASS',
  );
  const changed = structuredClone(testCase);
  changed.interaction.steps[0].hex = '61';
  assert.notEqual(requestDigest(changed), requestDigest(testCase));
});
