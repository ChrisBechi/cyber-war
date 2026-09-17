import { readFileSync, writeFileSync, readdirSync, mkdirSync, existsSync } from 'node:fs';
import { resolve, relative, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
export const root = fileURLToPath(new URL('../../', import.meta.url));
export const read = (p) => readFileSync(resolve(root, p), 'utf8');
export const json = (p) => JSON.parse(read(p));
export function files(dir) {
  return readdirSync(resolve(root, dir), { withFileTypes: true })
    .flatMap((e) => (e.isDirectory() ? files(`${dir}/${e.name}`) : [`${dir}/${e.name}`]))
    .sort();
}
export function write(p, value) {
  const path = resolve(root, p);
  mkdirSync(dirname(path), { recursive: true });
  const text = typeof value === 'string' ? value : JSON.stringify(value, null, 2) + '\n';
  if (existsSync(path) && readFileSync(path, 'utf8') === text) return;
  // Windows preview/indexing can briefly hold a generated report open.
  for (let attempt = 0; ; attempt++) {
    try {
      writeFileSync(path, text);
      return;
    } catch (error) {
      if (
        process.platform !== 'win32' ||
        attempt >= 5 ||
        !['UNKNOWN', 'EBUSY', 'EPERM'].includes(error.code)
      )
        throw error;
      Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 100 * (attempt + 1));
    }
  }
}
export function digest(paths) {
  const hash = createHash('sha256');
  for (const p of [...paths].sort())
    hash.update(p + '\0' + read(p).replaceAll('\r\n', '\n') + '\0');
  return hash.digest('hex');
}
export const registrySources = () =>
  [
    ...files('src-tauri/src'),
    ...files('content/software'),
    'src-tauri/Cargo.toml',
    'src-tauri/Cargo.lock',
  ].filter((p) => !p.endsWith('cli_compat_tests.rs'));
export const evidenceSources = () => [
  ...registrySources(),
  ...files('content/cli-compatibility'),
  ...files('tests/cli'),
  ...files('scripts/cli'),
  'scripts/cli-inventory.mjs',
  'scripts/cli-compat.mjs',
  'scripts/cli-verify.mjs',
  'vite.config.ts',
];
export function cargo(args, extra = {}) {
  const env = { ...process.env, ...extra };
  let executable = 'cargo';
  const local = resolve(root, '.tools/cargo/bin/cargo.exe');
  if (process.platform === 'win32' && existsSync(local)) {
    executable = local;
    const key = Object.keys(env).find((k) => k.toLowerCase() === 'path') ?? 'PATH';
    env[key] =
      `${resolve(root, '.tools/llvm-mingw-20260908-msvcrt-x86_64/bin')};${dirname(local)};${env[key] ?? ''}`;
    Object.assign(env, {
      RUSTUP_HOME: resolve(root, '.tools/rustup'),
      CARGO_HOME: resolve(root, '.tools/cargo'),
      CC: 'x86_64-w64-mingw32-clang',
      AR: 'llvm-ar',
      CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER: 'x86_64-w64-mingw32-clang',
      RUSTFLAGS: '-C link-self-contained=yes',
    });
  }
  const result = spawnSync(executable, args, { cwd: root, env, shell: false, stdio: 'inherit' });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`Cargo exited ${result.status}`);
}
export function runtimeRegistry() {
  const hash = digest(registrySources());
  const cache = 'artifacts/cli-runtime-registry.json';
  if (
    !existsSync(resolve(root, cache)) ||
    !existsSync(resolve(root, 'artifacts/cli-registry.hash')) ||
    read('artifacts/cli-registry.hash') !== hash
  ) {
    cargo([
      'test',
      '--offline',
      '--manifest-path',
      'src-tauri/Cargo.toml',
      '--lib',
      'cli_tooling_registry_export',
      '--',
      '--ignored',
      '--nocapture',
    ]);
    write('artifacts/cli-registry.hash', hash);
  }
  return json(cache);
}
export const relativePath = (p) => relative(root, p).replaceAll('\\', '/');
