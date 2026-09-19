import { files, read } from './io.mjs';
// Fixed application infrastructure exceptions, never reachable via CLI dispatch.
const allow = [
  {
    path: 'src-tauri/src/lib.rs',
    fragment: 'std::fs::create_dir_all(&data)?;',
    reason: 'Tauri creates its own application save directory.',
  },
];
// Remove only the item annotated cfg(test), preserving later production items
// and line numbers. Braces inside Rust strings/comments are not delimiters.
export function productionSource(raw) {
  const masked = raw.replace(
    /r(#+)"[\s\S]*?"\1|"(?:\\[\s\S]|[^"\\])*"|'(?:\\.|[^'\\\r\n])'|\/\/[^\n]*|\/\*[\s\S]*?\*\//g,
    (s) => s.replace(/[^\n]/g, ' '),
  );
  const chars = raw.split('');
  for (const match of masked.matchAll(/#\[cfg\(test\)\]/g)) {
    let end = match.index + match[0].length;
    for (; end < masked.length; end++) {
      if (masked[end] === ';') {
        end++;
        break;
      }
      if (masked[end] === '{') {
        let depth = 1;
        end++;
        while (end < masked.length && depth) {
          if (masked[end] === '{') depth++;
          if (masked[end] === '}') depth--;
          end++;
        }
        break;
      }
    }
    for (let i = match.index; i < end; i++) if (chars[i] !== '\n') chars[i] = ' ';
  }
  return chars.join('');
}
export function scanRuntime(entries) {
  const failures = [];
  const exceptions = [];
  for (const [path, raw] of entries) {
    // The desktop QA reporter is outside the release compilation unit. Require
    // its own inner compiler gate, so changing/removing the parent mod gate
    // cannot accidentally ship host report IO. Ungated code is still scanned.
    if (
      path === 'src-tauri/src/virtual_web/desktop_qa.rs' &&
      /^\s*(?:\/\/[^\n]*\n\s*)*#!\[cfg\(debug_assertions\)\]/.test(raw)
    ) {
      exceptions.push({
        path,
        reason: 'Compiler excludes the entire opted-in QA reporter from release.',
      });
      continue;
    }
    // These are compiler-gated fixture/test modules, validated separately below.
    if (
      path.endsWith('_tests.rs') ||
      path.endsWith('.test.ts') ||
      path.endsWith('.test.tsx') ||
      path.endsWith('/tests.rs') ||
      path.endsWith('/cli_tooling_bridge.rs') ||
      path.endsWith('/opening_fixture.rs')
    )
      continue;
    const text = path.endsWith('.rs') ? productionSource(raw) : raw;
    for (const [index, line] of text.split('\n').entries()) {
      if (/^\s*(\/\/|\*)/.test(line)) continue;
      const rule = allow.find((a) => a.path === path && line.trim() === a.fragment);
      if (rule) {
        exceptions.push(rule);
        continue;
      }
      if (
        /(?:std|tokio)\s*::\s*(?:process|net|fs)\b|\bCommand\s*::\s*new\s*\(|child_process|node:child_process|\b(?:reqwest|ureq)\b|(?:execFile|execSync|spawnSync)\s*\(/.test(
          line,
        )
      )
        failures.push({
          path,
          line: index + 1,
          reason: 'Host process/filesystem/network API in runtime',
        });
      if (
        /(?:import|require|include_str!|include_bytes!|\bmod\b).*?(?:scripts[\\/]cli|reference-environment|cli-case-actual|cli-evidence)/.test(
          line,
        )
      )
        failures.push({
          path,
          line: index + 1,
          reason: 'Development verification dependency in runtime',
        });
      if (
        /\b(?:notify|inotify|chokidar)\s*::|\b(?:inotify_init\w*|inotify_add_watch|ReadDirectoryChanges[AW]?|FSEventStream\w*)\b|\b(?:kqueue|kevent)\s*\(|\bfs\s*\.\s*watch\s*\(|from\s+['"](?:chokidar|node:fs)['"]/.test(
          line,
        )
      )
        failures.push({ path, line: index + 1, reason: 'Host filesystem watcher API in runtime' });
      if (/(?:powershell|cmd\.exe|wsl\.exe)/i.test(line))
        failures.push({ path, line: index + 1, reason: 'Host shell reference in runtime' });
    }
  }
  return { result: failures.length ? 'FAIL' : 'PASS', failures, exceptions };
}
export function hostGuard() {
  const entries = [...files('src-tauri/src'), ...files('src')]
    .filter((p) => /\.(rs|tsx?|m?js)$/.test(p))
    .map((p) => [p, read(p)]);
  const result = scanRuntime(entries);
  for (const name of ['cli_tooling_bridge', 'cli_compat_tests', 'opening_fixture']) {
    if (
      !new RegExp(`#\\[cfg\\(test\\)\\]\\s*(?:pub\\s+)?mod ${name};`).test(
        read('src-tauri/src/lib.rs'),
      )
    )
      result.failures.push({
        path: 'src-tauri/src/lib.rs',
        reason: `${name} must be compiler test-only`,
      });
  }
  result.result = result.failures.length ? 'FAIL' : 'PASS';
  result.note =
    'Source/build boundary guard, not an OS sandbox proof. Virtual command tests exercise the actual dispatch; there is no host-executor implementation in production.';
  return result;
}
