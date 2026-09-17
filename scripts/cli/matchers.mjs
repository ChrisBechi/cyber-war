import { isDeepStrictEqual } from 'node:util';
export function pointer(value, path) {
  return path
    .split('/')
    .slice(1)
    .reduce((v, k) => v?.[k.replaceAll('~1', '/').replaceAll('~0', '~')], value);
}
const escape = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
const dynamic = {
  PID: '[0-9]+',
  TIMESTAMP: '[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9:.]+Z',
  LATENCY: '[0-9]+(?:\\.[0-9]+)?',
  TRANSFER_RATE: '[0-9]+(?:\\.[0-9]+)?',
  IDENTIFIER: '[a-fA-F0-9-]{8,}',
};
export function normalizeDynamic(text, rules) {
  for (const rule of rules) {
    const regex = new RegExp(
      `(${escape(rule.prefix)})${dynamic[rule.type]}(?=${escape(rule.suffix)})`,
      'g',
    );
    if (!regex.test(text)) throw new Error(`Dynamic field not found: ${rule.type} ${rule.prefix}`);
    regex.lastIndex = 0;
    text = text.replace(regex, (_, prefix) => prefix + rule.token);
  }
  return text;
}
export function match(actual, matcher) {
  try {
    if (matcher.kind === 'STRUCTURED')
      return isDeepStrictEqual(
        typeof actual === 'string' ? JSON.parse(actual) : actual,
        matcher.value,
      );
    if (typeof actual !== 'string') return false;
    if (matcher.kind === 'EXACT') return actual === matcher.value;
    if (matcher.kind === 'REGEX') return new RegExp(matcher.pattern, matcher.flags).test(actual);
    return normalizeDynamic(actual, matcher.rules) === matcher.value;
  } catch {
    return false;
  }
}
export function compareCase(test, actual) {
  const errors = [];
  for (const stream of ['stdout', 'stderr']) {
    const key = stream + 'Hex';
    if (test.expected[key] === undefined || actual[key] === test.expected[key]) continue;
    const expected = Buffer.from(test.expected[key], 'hex');
    const observed = Buffer.from(actual[key] ?? '', 'hex');
    let offset = 0;
    while (
      offset < expected.length &&
      offset < observed.length &&
      expected[offset] === observed[offset]
    )
      offset++;
    errors.push(
      `${stream} bytes differ at offset ${offset}: expected ${expected.subarray(offset, offset + 16).toString('hex')}, actual ${observed.subarray(offset, offset + 16).toString('hex')}`,
    );
  }
  for (const stream of ['stdout', 'stderr'])
    if (
      test.expected[stream + 'Hex'] === undefined &&
      !match(actual[stream], test.expected[stream])
    )
      errors.push(
        `${stream} mismatch: expected ${JSON.stringify(test.expected[stream]).slice(0, 300)}, actual ${JSON.stringify(actual[stream]).slice(0, 300)}`,
      );
  if (actual.exitCode !== test.expected.exitCode)
    errors.push(`exitCode ${actual.exitCode}, expected ${test.expected.exitCode}`);
  for (const field of ['termination', 'observations'])
    if (
      test.expected[field] !== undefined &&
      !isDeepStrictEqual(test.expected[field], actual[field])
    )
      errors.push(
        `${field} mismatch: expected ${JSON.stringify(test.expected[field])}, actual ${JSON.stringify(actual[field])}`,
      );
  for (const assertion of test.expected.state) {
    const value = pointer(actual.after, assertion.path);
    const pass = assertion.absent
      ? value === undefined
      : assertion.unchanged
        ? isDeepStrictEqual(value, pointer(actual.before, assertion.path))
        : assertion.equalsPath
          ? value !== undefined &&
            isDeepStrictEqual(value, pointer(actual.after, assertion.equalsPath))
          : assertion.differsPath
            ? value !== undefined &&
              pointer(actual.after, assertion.differsPath) !== undefined &&
              !isDeepStrictEqual(value, pointer(actual.after, assertion.differsPath))
            : match(value, assertion.matcher);
    if (!pass) errors.push(`state ${assertion.path} mismatch`);
  }
  return {
    id: test.id,
    result: errors.length ? 'FAIL' : 'PASS',
    reason: errors.join('; ') || 'All declared stream/status/state assertions passed',
    errors,
  };
}
