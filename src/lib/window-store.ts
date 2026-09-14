import { create } from 'zustand';
import { softwareCatalog } from './software-catalog';

export type BuiltinAppId =
  | 'terminal'
  | 'files'
  | 'editor'
  | 'browser'
  | 'tor-browser'
  | 'messages'
  | 'forum'
  | 'missions'
  | 'codelab'
  | 'settings'
  | 'saves'
  | 'processes'
  | 'journey'
  | 'vigilia'
  | 'media-player'
  | 'image-viewer';
export type AppId = BuiltinAppId | `tool:${string}`;
export type WindowId = AppId | `terminal:${number}` | `files:${number}` | `editor:${number}`;
export const windowApp = (id: WindowId): AppId =>
  id.startsWith('terminal:')
    ? 'terminal'
    : id.startsWith('files:')
      ? 'files'
      : id.startsWith('editor:')
        ? 'editor'
        : (id as AppId);
export const apps: Record<AppId, { title: string }> = {
  ...Object.fromEntries(
    softwareCatalog.entries.map((entry) => [`tool:${entry.id}`, { title: entry.name }]),
  ),
  terminal: { title: 'Terminal' },
  files: { title: 'Arquivos' },
  editor: { title: 'HackPad' },
  browser: { title: 'Navegador' },
  'tor-browser': { title: 'Tor Browser' },
  messages: { title: 'Mensagens' },
  forum: { title: 'Fórum' },
  missions: { title: 'Trabalhos' },
  codelab: { title: 'CodeLab' },
  settings: { title: 'Configurações' },
  saves: { title: 'Campanha' },
  processes: { title: 'Task Manager' },
  journey: { title: 'Technical Journey' },
  vigilia: { title: 'SECTOR IX — Protocolo Zero' },
  'media-player': { title: 'Parole Media Player' },
  'image-viewer': { title: 'Ristretto Image Viewer' },
};
export type WindowState = {
  id: WindowId;
  x: number;
  y: number;
  width: number;
  height: number;
  z: number;
  minimized: boolean;
  maximized: boolean;
  fullscreen?: boolean;
  workspace: number;
  path?: string;
  asRoot?: boolean;
  cwd?: string;
  navigationId?: number;
};
type WindowStore = {
  windows: WindowState[];
  top: number;
  workspace: number;
  open: (id: WindowId, path?: string) => void;
  newTerminal: (options?: { cwd?: string; asRoot?: boolean }) => WindowId;
  newInstance: (app: 'files' | 'editor', path: string, asRoot?: boolean) => WindowId;
  close: (id: WindowId) => void;
  focus: (id: WindowId) => void;
  update: (id: WindowId, patch: Partial<WindowState>) => void;
  reset: () => void;
};
export const useWindows = create<WindowStore>((set, get) => ({
  windows: [],
  top: 1,
  workspace: 1,
  newTerminal: (options) => {
    const windows = get().windows;
    let id: WindowId = 'terminal';
    let number = 2;
    while (windows.some((window) => window.id === id)) {
      id = `terminal:${number++}`;
    }
    get().open(id);
    if (options) {
      get().update(id, options);
    }
    return id;
  },
  newInstance: (app, path, asRoot = false) => {
    let number = 2;
    let id: WindowId = `${app}:${number}`;
    while (get().windows.some((window) => window.id === id)) {
      id = `${app}:${++number}`;
    }
    get().open(id, path);
    get().update(id, { asRoot });
    return id;
  },
  open: (id, path) =>
    set((s) => ({
      top: s.top + 1,
      windows: s.windows.some((w) => w.id === id)
        ? s.windows.map((w) =>
            w.id === id
              ? {
                  ...w,
                  minimized: false,
                  workspace: s.workspace,
                  z: s.top + 1,
                  ...(path ? { path, navigationId: s.top + 1 } : {}),
                }
              : w,
          )
        : [
            ...s.windows,
            {
              id,
              x: 140 + (s.windows.length % 5) * 35,
              y: 60 + (s.windows.length % 5) * 25,
              width: windowApp(id) === 'processes' ? 300 : windowApp(id) === 'vigilia' ? 1120 : 780,
              height: windowApp(id) === 'processes' ? 460 : windowApp(id) === 'vigilia' ? 720 : 490,
              z: s.top + 1,
              minimized: false,
              maximized: false,
              workspace: s.workspace,
              path,
              navigationId: path ? s.top + 1 : undefined,
            },
          ],
    })),
  close: (id) => set((s) => ({ windows: s.windows.filter((w) => w.id !== id) })),
  focus: (id) =>
    set((s) => ({
      top: s.top + 1,
      windows: s.windows.map((w) => (w.id === id ? { ...w, z: s.top + 1, minimized: false } : w)),
    })),
  update: (id, patch) =>
    set((s) => ({ windows: s.windows.map((w) => (w.id === id ? { ...w, ...patch } : w)) })),
  reset: () => set({ windows: [], top: 1, workspace: 1 }),
}));
