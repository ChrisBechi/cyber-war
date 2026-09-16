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
  for (const stream of ['stdout', 'stderr'])
    if (!match(actual[stream], test.expected[stream])) errors.push(`${stream} mismatch`);
  if (actual.exitCode !== test.expected.exitCode)
    errors.push(`exitCode ${actual.exitCode}, expected ${test.expected.exitCode}`);
  for (const assertion of test.expected.state) {
    const value = pointer(actual.after, assertion.path);
    const pass = assertion.absent
      ? value === undefined
      : assertion.unchanged
        ? isDeepStrictEqual(value, pointer(actual.before, assertion.path))
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
