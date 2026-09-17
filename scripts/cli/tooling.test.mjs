import test from 'node:test';
import assert from 'node:assert/strict';
import { manifestSchema, caseSchema, capabilitySubsystem } from './schema.mjs';
import { discover, defaults } from './discovery.mjs';
import { evaluate, metrics, queue, regression } from './verification.mjs';
import { match, normalizeDynamic, compareCase } from './matchers.mjs';
import { scanRuntime, hostGuard } from './host-guard.mjs';
import { render } from './reports.mjs';
import { json } from './io.mjs';
import { loadCases } from './pipeline.mjs';
import { coreutilsGate, coreutilsSchema, attachCoreutils } from './coreutils.mjs';
import { caseSources } from './fingerprint.mjs';
import { ReferenceEnvironment } from './reference-environment.mjs';
import { cliRuntimeBoundary } from '../../vite.config.ts';
import { build } from 'vite';

function dataset() {
  const gates = defaults();
  for (const [name, g] of Object.entries(gates))
    if (!['DISCOVERY', 'REFERENCE_PINNED', 'HOST_ISOLATION', 'STDOUT'].includes(name))
      g.applicability = 'NOT_APPLICABLE';
  gates.STDOUT = {
    applicability: 'REQUIRED',
    reason: 'Full tiny fixture contract',
    testIds: ['toy/basic'],
  };
  const command = {
    id: 'command.toy.toy',
    command: 'toy',
    softwareId: 'toy',
    implementationKind: 'NATIVE',
    requirement: 'REQUIRED',
    classification: 'REAL_COMPAT',
    referenceVersion: '1',
    requirements: ['VFS'],
    gates,
    capabilities: ['VFS_READ'],
  };
  const software = {
    id: 'toy',
    classification: 'REAL_COMPAT',
    referenceVersion: '1',
    upstream: 'https://example.invalid',
    referenceSources: [{ kind: 'MAN_PAGE' }],
    executables: [command.id],
    aliases: [],
    launchers: [],
    dependencies: [],
    priority: 'P0',
    wave: 'W1_CORE_LINUX',
    usedByMissions: [],
  };
  const testCase = {
    id: 'toy/basic',
    softwareId: 'toy',
    command: 'toy',
    reference: { softwareId: 'toy', version: '1' },
    gates: ['STDOUT'],
    expected: {
      stdout: { kind: 'EXACT', value: 'ok\n' },
      stderr: { kind: 'EXACT', value: '' },
      exitCode: 0,
      state: [],
    },
  };
  const capture = {
    cases: [{ id: 'toy/basic', stdout: 'ok\n', stderr: '', exitCode: 0, before: {}, after: {} }],
  };
  const inventory = {
    commands: [command],
    software: [software],
    launchers: [],
    subsystems: [{ id: 'VFS', state: 'READY', requiredTests: ['toy/basic'] }],
  };
  return { inventory, cases: [testCase], capture };
}
const run = (d) => evaluate(d.inventory, d.cases, d.capture, { result: 'PASS' });
test('Coreutils project cases never substitute GNU reference, and declared flags require evidence', () => {
  const c = {
    command: 'basename',
    referenceVersion: '9.7',
    coreutils: {
      flags: ['-z'],
      knownGaps: ['Missing error contract'],
      contracts: [{ id: 'paths', applicability: 'REQUIRED', evidence: ['base/path'] }],
    },
  };
  const tests = new Map([['base/path', { id: 'base/path', command: 'basename', flags: [] }]]);
  const results = new Map([['base/path', { result: 'PASS' }]]);
  assert.equal(coreutilsGate(c, 'COMMAND_CONTRACTS', tests, results).result, 'SKIPPED');
  assert.notEqual(coreutilsGate(c, 'GNU_REFERENCE', tests, results).result, 'PASS');
  tests.get('base/path').flags = ['-z'];
  assert.equal(coreutilsGate(c, 'COMMAND_CONTRACTS', tests, results).result, 'PASS');
  assert.equal(coreutilsGate(c, 'KNOWN_GAPS', tests, results).result, 'SKIPPED');
  c.coreutils.knownGaps = [];
  assert.equal(coreutilsGate(c, 'KNOWN_GAPS', tests, results).result, 'PASS');
  results.get('base/path').result = 'FAIL';
  assert.equal(coreutilsGate(c, 'COMMAND_CONTRACTS', tests, results).result, 'SKIPPED');
});
test('Coreutils inventory requires every registered executable and exact baseline', () => {
  const config = coreutilsSchema.parse(json('content/cli-compatibility/coreutils.json'));
  const inventory = { software: [{ id: 'coreutils', referenceVersion: '9.7' }], commands: [] };
  assert.throws(() => attachCoreutils(inventory, config), /Orphan/);
  inventory.software[0].referenceVersion = '8.32';
  assert.throws(() => attachCoreutils(inventory, config), /baseline/);
});
test('Byte mismatch reports bounded offset and handler fingerprints follow dependencies', () => {
  const t = dataset().cases[0];
  t.expected.stdoutHex = '00ff80';
  const a = dataset().capture.cases[0];
  a.stdoutHex = '00fe80';
  assert.match(compareCase(t, a).reason, /offset 1/);
  const sources = [
    'src-tauri/src/terminal_text.rs',
    'src-tauri/src/vfs/resolve.rs',
    'src-tauri/src/coreutils/foundation.rs',
  ];
  assert.deepEqual(
    caseSources({ softwareId: 'coreutils', command: 'basename' }, sources),
    sources.slice(1),
  );
  assert.deepEqual(
    caseSources({ softwareId: 'coreutils', command: 'basename', script: 'sort a' }, sources),
    sources,
  );
  assert.deepEqual(caseSources({ softwareId: 'coreutils', command: 'env' }, sources), sources);
});
test('all required PASS derives VERIFIED; N/A does not block', () =>
  assert.equal(run(dataset()).commands[0].effectiveStatus, 'VERIFIED'));
test('failed/skipped REQUIRED, missing reference, catalog and missing subsystem block VERIFIED', () => {
  for (const change of [
    (d) => (d.capture.cases[0].stdout = 'bad'),
    (d) => (d.capture = null),
    (d) => (d.inventory.software[0].referenceVersion = null),
    (d) => (d.inventory.commands[0].implementationKind = 'CATALOG_ONLY'),
    (d) => (d.inventory.subsystems[0].state = 'MISSING'),
  ]) {
    const d = dataset();
    change(d);
    assert.notEqual(run(d).commands[0].effectiveStatus, 'VERIFIED');
  }
});
test('manual VERIFIED with missing evidence fails integrity', () => {
  const d = dataset();
  d.capture = null;
  d.inventory.commands[0].declaredStatus = 'VERIFIED';
  const r = run(d);
  assert.equal(r.commands[0].effectiveStatus, 'PARTIAL');
  assert.match(r.verification.errors.join(), /Fake VERIFIED/);
});

test('wrong reference version cannot supply PASS evidence', () => {
  const d = dataset();
  d.cases[0].reference.version = '2';
  const r = run(d);
  assert.equal(r.verification.caseResults[0].result, 'FAIL');
  assert.notEqual(r.commands[0].effectiveStatus, 'VERIFIED');
});

test('software capabilities and dependencies block certification', () => {
  const d = dataset();
  d.inventory.software[0].requirements = ['PACKETS'];
  assert.notEqual(run(d).software[0].effectiveStatus, 'VERIFIED');
  const other = structuredClone(d.inventory.software[0]);
  other.id = 'dependency';
  other.executables = [];
  other.requirements = [];
  d.inventory.software.push(other);
  d.inventory.software[0].dependencies = ['dependency'];
  assert.notEqual(run(d).commands[0].effectiveStatus, 'VERIFIED');
});
test('optional failure is visible but policy does not block', () => {
  const d = dataset();
  d.inventory.commands[0].requirements = [];
  d.inventory.commands[0].gates.STDOUT.applicability = 'OPTIONAL';
  d.capture.cases[0].stdout = 'bad';
  const r = run(d);
  assert.equal(r.commands[0].gates.STDOUT.result, 'FAIL');
  assert.equal(r.commands[0].effectiveStatus, 'VERIFIED');
});
test('READY without test evidence is rejected', () => {
  const d = dataset();
  d.inventory.subsystems[0].requiredTests = [];
  const r = run(d);
  assert.notEqual(r.commands[0].effectiveStatus, 'VERIFIED');
  assert.match(r.verification.errors.join(), /claims READY/);
});
test('capability readiness is derived from current evidence without upgrading the subsystem', () => {
  const d = dataset();
  d.inventory.subsystems[0].state = 'PARTIAL';
  d.inventory.subsystems[0].capabilities = [
    { id: 'VFS.GLOB', state: 'READY', requiredTests: ['toy/basic'] },
  ];
  let report = run(d);
  assert.equal(report.subsystems[0].effectiveState, 'PARTIAL');
  assert.equal(report.subsystems[0].capabilities[0].effectiveState, 'READY');
  d.capture.cases[0].stdout = 'wrong';
  report = run(d);
  assert.equal(report.subsystems[0].capabilities[0].effectiveState, 'PARTIAL');
  assert.match(report.verification.errors.join(), /Capability VFS.GLOB claims READY/);
});
test('a program requires only declared shell capabilities while jobs stay partial', () => {
  const d = dataset();
  d.inventory.commands[0].requirements = ['SHELL.PIPELINES'];
  d.inventory.software[0].requirements = ['SHELL.PIPELINES'];
  d.inventory.subsystems = [
    {
      id: 'SHELL',
      state: 'PARTIAL',
      requiredTests: [],
      capabilities: [
        { id: 'SHELL.PIPELINES', state: 'READY', requiredTests: ['toy/basic'] },
        { id: 'SHELL.JOBS', state: 'PARTIAL', requiredTests: [] },
      ],
    },
  ];
  assert.equal(run(d).commands[0].effectiveStatus, 'VERIFIED');
  d.capture.cases[0].stdout = 'bad';
  assert.deepEqual(run(d).commands[0].blockedBy, ['SUBSYSTEM:SHELL.PIPELINES:PARTIAL']);
});
test('family with two verified and one partial remains PARTIAL', () => {
  const d = dataset();
  for (const n of ['b', 'c']) {
    const c = structuredClone(d.inventory.commands[0]);
    c.id += n;
    c.command = n;
    c.gates.STDOUT.testIds = [];
    if (n === 'b') c.gates.STDOUT.applicability = 'NOT_APPLICABLE';
    d.inventory.commands.push(c);
    d.inventory.software[0].executables.push(c.id);
  }
  const r = run(d);
  assert.equal(r.commands.filter((c) => c.effectiveStatus === 'VERIFIED').length, 2);
  assert.equal(r.software[0].effectiveStatus, 'PARTIAL');
});
test('aliases do not inflate real denominator; fictional separate; catalog explicit', () => {
  const r = run(dataset());
  const base = r.commands[0];
  r.commands.push(
    { ...base, id: 'alias', implementationKind: 'ALIAS' },
    { ...base, id: 'fiction', classification: 'FICTIONAL_NATIVE' },
    { ...base, id: 'catalog', implementationKind: 'CATALOG_ONLY', effectiveStatus: 'UNVERIFIED' },
  );
  const m = metrics(r);
  assert.equal(m.softwareGroups, 1);
  assert.equal(m.aliases, 1);
  assert.equal(m.fictionalNative, 1);
  assert.equal(m.catalogOnly, 1);
  assert.equal(m.commandVerification.total, 2);
  assert.equal(m.commandVerification.percent, 50);
});
test('blocked subsystem is explicit in deterministic queue', () => {
  const d = dataset();
  d.inventory.subsystems[0].state = 'MISSING';
  const r = run(d);
  assert.equal(queue(r).next.action, 'NEEDS_SUBSYSTEM');
  assert.deepEqual(queue(r).next.blockers, ['SUBSYSTEM:VFS:MISSING']);
  assert.deepEqual(queue(r), queue(r));
});
test('regression rejects lost verification/native implementation and requires contextual unexpired override', () => {
  const before = run(dataset());
  const after = structuredClone(before);
  after.commands[0].effectiveStatus = 'PARTIAL';
  assert.equal(regression(before, after).violations.length, 1);
  assert.equal(
    regression(before, after, [
      { id: before.commands[0].id, reason: 'temporary', reference: '#1', expires: '2099-01-01' },
    ]).violations.length,
    1,
  );
  assert.equal(
    regression(before, after, [
      {
        id: before.commands[0].id,
        reason: 'Documented change of the pinned upstream contract',
        reference: 'issue:123',
        expires: '2099-01-01',
      },
    ]).violations.length,
    0,
  );
});
test('matchers preserve whitespace, counts and unselected dynamic values', () => {
  assert(!match('a  12\n', { kind: 'EXACT', value: 'a 12\n' }));
  assert(match('value=2\n', { kind: 'REGEX', pattern: '^value=2\\n$', flags: '' }));
  assert(match('{"a":1}', { kind: 'STRUCTURED', value: { a: 1 } }));
  const rules = [
    {
      type: 'PID',
      prefix: 'pid=',
      suffix: ' ',
      token: '<PID>',
      reason: 'Virtual pid allocated per isolated fixture.',
    },
  ];
  assert.equal(normalizeDynamic('pid=123 total=12\n', rules), 'pid=<PID> total=12\n');
  assert.throws(() => normalizeDynamic('total=12', rules));
  assert(
    !match('pid=123 total=13\n', {
      kind: 'NORMALIZED_DYNAMIC',
      value: 'pid=<PID> total=12\n',
      rules,
    }),
  );
});

test('regression protects software verification independently of commands', () => {
  const before = run(dataset());
  const after = structuredClone(before);
  after.software[0].effectiveStatus = 'PARTIAL';
  assert.deepEqual(regression(before, after).violations, ['Unapproved regression software:toy']);
});
test('state assertions cover VFS/process/network/package values and absence', () => {
  const d = dataset();
  const c = d.cases[0];
  c.expected.state = [
    { path: '/vfs/f', absent: true },
    { path: '/processes/0/running', matcher: { kind: 'STRUCTURED', value: false } },
    { path: '/network', unchanged: true },
    { path: '/packages/p/status', matcher: { kind: 'EXACT', value: 'installed' } },
  ];
  const actual = {
    ...d.capture.cases[0],
    before: { network: { connected: true } },
    after: {
      vfs: {},
      processes: [{ running: false }],
      network: { connected: true },
      packages: { p: { status: 'installed' } },
    },
  };
  assert.equal(compareCase(c, actual).result, 'PASS');
  actual.after.processes[0].running = true;
  assert.equal(compareCase(c, actual).result, 'FAIL');
});
test('host/build guard rejects runtime processes, network and reference imports', () => {
  for (const source of [
    'std::process::Command::new("sh")',
    'use std::net::TcpStream;',
    'import x from "../../scripts/cli/reference-environment.mjs"',
    'import { spawnSync } from "node:child_process"',
  ])
    assert.equal(scanRuntime([['src/unsafe.ts', source]]).result, 'FAIL');
  assert.equal(hostGuard().result, 'PASS', JSON.stringify(hostGuard().failures));
});
test('host guard scans production after gated declarations and test blocks', () => {
  for (const source of [
    '#[cfg(test)]\nmod tests;\nfn run() { std::process::Command::new("sh"); }',
    '#[cfg(test)]\nmod tests { let text = "}"; }\nfn run() { std::net::TcpStream::connect("host"); }',
  ])
    assert.equal(scanRuntime([['src-tauri/src/example.rs', source]]).result, 'FAIL');
});
test('no host reference execution fallback', async () => {
  await assert.rejects(
    new ReferenceEnvironment().execute({
      softwareId: 'toy',
      version: '1',
      command: 'toy',
      argv: [],
      stdin: null,
      env: {},
      cwd: '/fixture',
      tty: { isTTY: false, rows: 24, columns: 80, ansiSupport: false, interactive: false },
      fixtureId: 'toy/basic',
      environmentDigest: 'pinned',
    }),
    /No host fallback/,
  );
});

test('production bundler rejects a real reference-tooling import', async () => {
  await assert.rejects(
    build({
      configFile: false,
      logLevel: 'silent',
      plugins: [cliRuntimeBoundary()],
      build: { write: false, rollupOptions: { input: 'scripts/cli/reference-environment.mjs' } },
    }),
    /cannot enter the production bundle/,
  );
});
test('schemas reject unknown capabilities, statuses and fixture fields', () => {
  const config = json('content/cli-compatibility/manifest.json');
  const name = Object.keys(config.native)[0];
  config.native[name].capabilities = ['HOST_EXEC'];
  assert.throws(() => manifestSchema.parse(config));
  const c = loadCases()[0];
  c.expected.ignoreAllWhitespace = true;
  assert.throws(() => caseSchema.parse(c));
});
test('runtime discovery, grouping, aliases and duplicate/orphan/version validation', () => {
  const runtime = json('artifacts/cli-runtime-registry.json'),
    manifest = json('content/cli-compatibility/manifest.json'),
    catalog = json('content/software/kali-default.json'),
    baseline = json('content/cli-compatibility/baseline.json'),
    subs = json('content/cli-compatibility/subsystems.json');
  const build = (m) => discover(runtime, m, catalog, baseline, subs);
  const r = build(manifest);
  assert.equal(r.commands.length, new Set(runtime.names).size);
  assert.equal(r.commands.find((c) => c.command === 'ls').implementationKind, 'NATIVE');
  assert.equal(r.commands.find((c) => c.command === 'cd').implementationKind, 'BUILTIN');
  assert.equal(r.commands.find((c) => c.command === '.').aliasOf, 'source');
  assert.equal(
    r.commands.find((c) => c.command === 'airmon-ng').implementationKind,
    'CATALOG_ONLY',
  );
  assert.equal(
    r.commands.find((c) => c.command === 'msfconsole').softwareId,
    'metasploit-framework',
  );
  assert(r.launchers.some((l) => l.kind === 'CATALOG'));
  for (const change of [
    (m) => delete m.native.ls,
    (m) => (m.native.orphan = m.native.ls),
    (m) => (m.native.ls.referenceVersion = 'other'),
    (m) => (m.native['.'].aliasOf = 'missing'),
    (m) => delete m.collisions[Object.keys(m.collisions)[0]],
  ]) {
    const m = structuredClone(manifest);
    change(m);
    assert.throws(() => build(m));
  }
  const one = build(manifest),
    two = build(manifest);
  delete one.performance;
  delete two.performance;
  assert.equal(JSON.stringify(one), JSON.stringify(two));
});
test('dashboard deterministic and derived, no independent status list', () => {
  const report = run(dataset());
  report.summary = metrics(report);
  report.queue = queue(report);
  assert.deepEqual(render(report), render(report));
  assert.match(render(report)['cli-dashboard.md'], /commandVerification/);
});
test('every capability has a typed subsystem', () => {
  for (const value of Object.values(capabilitySubsystem)) assert.equal(typeof value, 'string');
});

test('VFS dependencies and relational metadata assertions cannot bypass missing evidence', () => {
  const d = dataset();
  d.inventory.commands[0].requirements = ['VFS.INODES'];
  d.inventory.software[0].requirements = ['VFS.INODES'];
  d.inventory.subsystems[0].state = 'PARTIAL';
  d.inventory.subsystems[0].capabilities = [
    { id: 'VFS.INODES', state: 'READY', requiredTests: ['toy/basic'] },
  ];
  assert.equal(run(d).commands[0].effectiveStatus, 'VERIFIED');
  d.capture.cases = [];
  assert.equal(run(d).subsystems[0].capabilities[0].effectiveState, 'PARTIAL');
  assert.match(run(d).commands[0].blockedBy.join(), /VFS.INODES/);
  const c = {
    id: 'relation',
    expected: {
      stdout: { kind: 'EXACT', value: '' },
      stderr: { kind: 'EXACT', value: '' },
      exitCode: 0,
      state: [{ path: '/a', equalsPath: '/b' }],
    },
  };
  assert.equal(compareCase(c, { stdout: '', stderr: '', exitCode: 0, after: {} }).result, 'FAIL');
  assert.equal(
    compareCase(c, { stdout: '', stderr: '', exitCode: 0, after: { a: 7, b: 7 } }).result,
    'PASS',
  );
  c.expected.state = [{ path: '/a', differsPath: '/b' }];
  assert.equal(
    compareCase(c, { stdout: '', stderr: '', exitCode: 0, after: { a: 7 } }).result,
    'FAIL',
  );
});
