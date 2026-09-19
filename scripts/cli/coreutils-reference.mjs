// DEV evidence validation. This module only reads captures; it never invokes GNU.
import { createHash } from 'node:crypto';
import { existsSync } from 'node:fs';
import { resolve } from 'node:path';
import { json, read, root } from './io.mjs';
export const canonical = (v) =>
  JSON.stringify(
    v && typeof v === 'object'
      ? Array.isArray(v)
        ? v.map((x) => JSON.parse(canonical(x)))
        : Object.fromEntries(
            Object.keys(v)
              .filter((k) => v[k] !== undefined)
              .sort()
              .map((k) => [k, JSON.parse(canonical(v[k]))]),
          )
      : v,
  );
const hash = (s) => createHash('sha256').update(s).digest('hex');
export const sourceHash = (p) => hash(read(p).replaceAll('\r\n', '\n'));
export function referenceRequest(c) {
  const request = {
    id: c.id,
    command: c.command,
    invocation: c.invocation ?? `/usr/bin/${c.command}`,
    argv: c.argv,
    stdinHex: c.stdinHex ?? Buffer.from(c.stdin ?? '').toString('hex'),
    env: c.env,
    cwd: c.cwd,
    fixture: Object.fromEntries(
      ['files', 'bytes', 'directories', 'modes', 'setup'].map((k) => [
        k,
        c.fixture?.[k] ?? (['directories', 'setup'].includes(k) ? [] : {}),
      ]),
    ),
    process: c.process ?? null,
    transport: c.transport ?? 'direct',
  };
  if (['cat', 'head', 'tail'].includes(c.command)) {
    request.io = c.io ?? null;
    request.interaction = c.interaction ?? null;
    for (const key of ['hardlinks', 'symlinks']) request.fixture[key] = c.fixture?.[key] ?? {};
  }
  return request;
}
export const requestDigest = (c) => hash(canonical(referenceRequest(c)));
export function referenceCapture(command) {
  const path = `tests/cli/gnu/coreutils/${command.referenceVersion}/${command.command}.json`;
  return existsSync(resolve(root, path)) ? json(path) : null;
}
export function validateReference(command, capture) {
  if (!capture) return 'No GNU reference capture';
  const binary = capture.environment?.binaryHashes?.[command.command];
  const lock = json('content/cli-compatibility/coreutils-environment.json');
  if (
    capture.schemaVersion !==
      (command.command === 'tail' ? 4 : ['cat', 'head'].includes(command.command) ? 3 : 2) ||
    capture.provenance !== 'GNU_REFERENCE' ||
    capture.version !== command.referenceVersion ||
    capture.command !== command.command ||
    capture.locale !== 'C' ||
    !Number.isFinite(Date.parse(capture.capturedAt))
  )
    return 'Reference provenance/baseline/schema/locale mismatch';
  if (
    capture.harnessHash !== sourceHash('scripts/cli/coreutils-reference.py') ||
    capture.environment?.lockHash !==
      sourceHash('content/cli-compatibility/coreutils-environment.json')
  )
    return 'Reference harness/environment fingerprint stale';
  if (
    ['cat', 'head', 'tail'].includes(command.command) &&
    capture.interactionHash !== sourceHash('scripts/cli/coreutils_interaction.py')
  )
    return 'Reference interaction harness fingerprint stale';
  if (
    command.command === 'tail' &&
    capture.followHash !== sourceHash('scripts/cli/coreutils_follow.py')
  )
    return 'Reference follow harness fingerprint stale';
  if (
    capture.environment?.os !== 'Linux' ||
    capture.environment?.id !== lock.id ||
    capture.environment?.architecture !== 'x86_64' ||
    capture.environment?.locale !== 'C' ||
    capture.environment?.network !== 'disabled' ||
    capture.environment?.isolation !== 'bubblewrap/unshare-all' ||
    !binary?.sha256?.match(/^[a-f0-9]{64}$/) ||
    binary.sha256 !== lock.binaryHashes?.[command.command] ||
    binary.versionLine !== `${command.command} (GNU coreutils) ${command.referenceVersion}`
  )
    return 'Reference binary/environment identity invalid';
  if (
    !Array.isArray(capture.cases) ||
    new Set(capture.cases.map((c) => c.id)).size !== capture.cases.length
  )
    return 'Duplicate/missing reference cases';
  return null;
}
export function referenceDifference(test, actual, row) {
  if (
    test.script ||
    test.fixture?.setup?.length ||
    test.inputEvents?.length ||
    test.roundtrip ||
    (test.tty?.isTTY && !test.interaction)
  )
    return 'GNU structured reference does not cover scripts/events/roundtrip/TTY execution';
  if (!row || row.reproducible !== true) return 'Missing reproducible GNU case';
  if (
    row.requestDigest !== requestDigest(test) ||
    canonical(row.request) !== canonical(referenceRequest(test))
  )
    return 'GNU request fingerprint mismatch';
  if (!actual) return 'No current project execution';
  const errors = [];
  for (const stream of ['stdout', 'stderr']) {
    const expected = row[stream + 'Hex'],
      observed = actual[stream + 'Hex'] ?? Buffer.from(actual[stream] ?? '').toString('hex');
    if (typeof expected !== 'string' || !/^(?:[a-f0-9]{2})*$/.test(expected)) {
      errors.push(`Invalid GNU ${stream} bytes`);
      continue;
    }
    if (expected !== observed) {
      const a = Buffer.from(expected, 'hex'),
        b = Buffer.from(observed, 'hex');
      let offset = 0;
      while (offset < a.length && offset < b.length && a[offset] === b[offset]) offset++;
      errors.push(
        `${stream} byte ${offset}: GNU=${a.subarray(offset, offset + 24).toString('hex')} project=${b.subarray(offset, offset + 24).toString('hex')}`,
      );
    }
  }
  if (row.exitCode !== actual.exitCode)
    errors.push(`status GNU=${row.exitCode} project=${actual.exitCode}`);
  if (['cat', 'head', 'tail'].includes(test.command)) {
    const io = test.io ?? {};
    const endpoints = {
      stdin:
        test.interaction?.schemaVersion === 2
          ? io.stdinPath
            ? 'file'
            : 'null'
          : test.interaction
            ? 'pty-canonical-echo-off'
            : io.stdinPath
              ? 'file'
              : 'pipe',
      stdout:
        io.stdoutPath || test.transport === 'redirect'
          ? 'file'
          : io.closedConsumer
            ? 'closed-pipe'
            : 'pipe',
      stderr: io.stderrPath ? 'file' : 'pipe',
    };
    if (canonical(row.endpoints) !== canonical(endpoints))
      errors.push('GNU descriptor provenance mismatch');
    const terminationStatus =
      row.termination?.kind === 'exit'
        ? row.termination.code
        : row.termination?.kind === 'signal'
          ? { SIGINT: 130, SIGPIPE: 141, SIGTERM: 143 }[row.termination.signal]
          : undefined;
    if (terminationStatus === undefined || terminationStatus !== row.exitCode)
      errors.push('Invalid GNU termination result');
    for (const field of ['termination', 'observations'])
      if (canonical(row[field]) !== canonical(actual[field]))
        errors.push(`${field} differs from GNU`);
    const nodes = actual.after?.vfs ?? {};
    for (const relative of Object.keys(row.before)) {
      if (!Object.hasOwn(row.after, relative) && Object.hasOwn(nodes, '/home/kali/' + relative))
        errors.push(`Path removed by GNU still exists: ${relative}`);
    }
    const pairs = [];
    for (const [relative, expected] of Object.entries(row.after)) {
      const path = '/home/kali/' + relative,
        observed = nodes[path];
      if (!observed) {
        errors.push(`Missing VFS path ${path}`);
        continue;
      }
      for (const key of ['mode', 'nlink'])
        if (observed[key] !== expected[key]) errors.push(`${path}/${key} differs from GNU`);
      if (observed.kind !== expected.type) errors.push(`${path}/type differs from GNU`);
      if (expected.type === 'file' && actual.files?.[path] !== expected.contentHex)
        errors.push(`${path}/bytes differ from GNU`);
      if (expected.type === 'symlink' && observed.content !== expected.target)
        errors.push(`${path}/target differs from GNU`);
      for (const [group, ino] of pairs)
        if ((group === expected.inodeGroup) !== (ino === observed.ino))
          errors.push(`${path}/inode relationship differs from GNU`);
      pairs.push([expected.inodeGroup, observed.ino]);
    }
    const stable = (nodes) =>
      Object.fromEntries(
        Object.entries(nodes)
          .filter(
            ([p]) =>
              (!Object.hasOwn(row.after, p.slice('/home/kali/'.length)) &&
                !Object.hasOwn(row.before, p.slice('/home/kali/'.length))) ||
              !p.startsWith('/home/kali/'),
          )
          .map(([p, n]) => [
            p,
            {
              kind: n.kind,
              content: n.content,
              blob: n.blob ?? null,
              mode: n.mode,
              ino: n.ino,
              nlink: n.nlink,
            },
          ]),
      );
    if (canonical(stable(actual.before.vfs)) !== canonical(stable(nodes)))
      errors.push('Unexpected project filesystem mutation');
    return errors.join('; ') || null;
  }
  if (test.transport === 'redirect') {
    const output = actual.after?.vfs?.['/home/kali/reference-output'];
    const bytes = output?.blob ? null : Buffer.from(output?.content ?? '').toString('hex');
    if (!output || bytes !== row.after?.['reference-output']?.contentHex)
      errors.push('Redirected VFS bytes differ from GNU');
    if (
      output?.mode !== row.after?.['reference-output']?.mode ||
      output?.nlink !== row.after?.['reference-output']?.nlink
    )
      errors.push('Redirected VFS metadata differs from GNU');
  } else if (canonical(row.before) !== canonical(row.after))
    errors.push('Unexpected GNU filesystem mutation');
  // Foundation has no filesystem writes beyond the declared stdout redirect.
  const before = actual.before?.vfs ?? {},
    after = actual.after?.vfs ?? {};
  const selected = (nodes) =>
    Object.fromEntries(
      Object.entries(nodes)
        .filter(([p]) => test.transport !== 'redirect' || p !== '/home/kali/reference-output')
        .map(([p, n]) => [
          p,
          {
            kind: n.kind,
            content: n.content,
            blob: n.blob ?? null,
            mode: n.mode,
            uid: n.uid,
            gid: n.gid,
            ino: n.ino,
            nlink: n.nlink,
          },
        ]),
    );
  if (canonical(selected(before)) !== canonical(selected(after)))
    errors.push('Unexpected project filesystem mutation');
  return errors.join('; ') || null;
}
