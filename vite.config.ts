import { defineConfig, type Plugin } from 'vite';
import react from '@vitejs/plugin-react';

const host = process.env.TAURI_DEV_HOST;

export function cliRuntimeBoundary(): Plugin {
  return {
    name: 'cli-runtime-boundary',
    apply: 'build',
    load(id) {
      const normalized = id.replaceAll('\\', '/');
      if (/\/scripts\/cli\/|cli_tooling_bridge|cli-case-actual|cli-evidence/.test(normalized)) {
        this.error('CLI development/reference tooling cannot enter the production bundle.');
      }
    },
  };
}

export default defineConfig({
  plugins: [cliRuntimeBoundary(), react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
    watch: { ignored: ['**/src-tauri/**', '**/artifacts/**'] },
  },
});
