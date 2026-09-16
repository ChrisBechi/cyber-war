import type { ReactNode } from 'react';
import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import type * as AppSettingsModule from './lib/app-settings';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { App } from './App';
import { Desktop } from './features/desktop/Desktop';
import type { World } from './lib/api';
import { defaultAppSettings, useAppSettings } from './lib/app-settings';
import { useGame } from './lib/game-store';
import { endSession, startSession } from './lib/session';
import { useWindows } from './lib/window-store';

const ipc = vi.hoisted(() => vi.fn<(command: string) => Promise<unknown>>());
vi.mock('@tauri-apps/api/core', () => ({ invoke: ipc, isTauri: () => true }));
vi.mock('@tauri-apps/api/event', () => ({ listen: () => Promise.resolve(() => undefined) }));
vi.mock('./lib/use-host-battery', () => ({ useHostBattery: () => null }));
vi.mock('./lib/app-settings', async (original) => ({
  ...(await original<typeof AppSettingsModule>()),
  loadAppSettings: () => Promise.resolve(),
}));
vi.mock('./lib/audio-manager', () => ({
  audioManager: {
    preload: () => Promise.resolve(),
    play: vi.fn(),
    unlock: vi.fn(),
    pause: vi.fn(),
    resume: vi.fn(),
    stopAll: vi.fn(),
  },
}));
vi.mock('./features/boot/boot-assets', () => ({
  preloadBranding: () => Promise.resolve(),
  openingSources: [],
}));
vi.mock('./features/boot/FadeTransition', () => ({
  FadeTransition: ({ stage, render }: { stage: string; render: (stage: string) => ReactNode }) =>
    render(stage),
}));
vi.mock('./features/boot/BootFlow', () => ({
  BootFlow: ({ onPlay }: { onPlay: (newGame: boolean, needsLogin?: boolean) => void }) => (
    <nav aria-label="Menu do jogo">
      <button onClick={() => onPlay(false, true)}>Carregar campanha</button>
      <button onClick={() => onPlay(true)}>Nova campanha</button>
      <button onClick={() => onPlay(false)}>Entrada direta</button>
    </nav>
  ),
}));
vi.mock('./features/files/FileManager', () => ({ FileManager: () => null }));
vi.mock('./features/files/Editor', () => ({ Editor: () => null }));
vi.mock('./features/media/ImageViewer', () => ({ ImageViewer: () => null }));
vi.mock('./features/media/MediaPlayer', () => ({ MediaPlayer: () => null }));
vi.mock('./features/terminal/TerminalWindow', () => ({ TerminalWindow: () => null }));
vi.mock('./features/desktop/Applications', async () => ({
  Saves: (await import('./features/desktop/Saves')).Saves,
  Browser: () => null,
  CodeLab: () => null,
  Forum: () => null,
  Journey: () => null,
  Messages: () => null,
  Missions: () => null,
  Processes: () => null,
  Settings: () => null,
}));

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: Error) => void;
  const promise = new Promise<T>((accept, fail) => {
    resolve = accept;
    reject = fail;
  });
  return { promise, resolve, reject };
}

let backendWorld: World;
let endReply: () => Promise<unknown>;
let startReply: () => Promise<unknown>;
let worldReply: () => Promise<unknown>;
const commands = () => ipc.mock.calls.map(([command]) => command);
const sessionCommands = () =>
  commands().filter((command) => ['end_session', 'session_start'].includes(command));

beforeEach(() => {
  vi.clearAllMocks();
  vi.stubGlobal('matchMedia', () => ({
    matches: false,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
  }));
  vi.spyOn(HTMLMediaElement.prototype, 'load').mockImplementation(() => undefined);
  backendWorld = {
    nickname: 'kali',
    hostname: 'lifeos',
    session: 1,
    money: 0,
    reputation: 0,
    playtimeSeconds: 0,
    vfs: { nodes: {} },
    network: { connected: true, gateway: '10.20.4.1', hosts: {} },
    domains: { registrations: {}, offers: [], onionServices: {} },
    terminal: { cwd: '/home/kali', user: 'kali', host: null },
    messages: [],
    contacts: [],
    inventory: [],
    techniques: [],
    flags: [],
    memoryValue: 0,
    memoryCandidates: [],
    processes: [],
    settings: { loginUsername: 'kali', loginPassword: 'secret' },
  };
  endReply = () => Promise.resolve(null); // Rust () is serialized as null by Tauri.
  startReply = () => Promise.resolve(backendWorld);
  worldReply = () => Promise.resolve(backendWorld);
  ipc.mockImplementation((command) => {
    switch (command) {
      case 'terminal_cancel_all':
        return Promise.resolve(null);
      case 'end_session':
        return endReply();
      case 'session_start':
        return startReply();
      case 'world_get':
        return worldReply();
      case 'mission_get_state':
      case 'list_checkpoints':
        return Promise.resolve([]);
      default:
        return Promise.reject(new Error(`Unexpected IPC: ${command}`));
    }
  });
  useGame.setState({
    world: structuredClone(backendWorld),
    missions: [],
    slot: 1,
    revision: 0,
    error: '',
    busy: false,
    sessionPending: false,
  });
  useWindows.getState().reset();
  useAppSettings.setState({ settings: defaultAppSettings, ready: true, error: '' });
});
afterEach(() => {
  cleanup(); // Dispose media while its explicit jsdom test double still exists.
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

type ExitPath = 'panel' | 'launcher' | 'campaign' | 'locked';
function logoutButton(path: ExitPath) {
  if (path === 'launcher') {
    fireEvent.click(screen.getByRole('button', { name: 'Abrir menu de aplicativos' }));
    return screen.getByRole('button', { name: 'Encerrar sessão e sair' });
  }
  if (path === 'campaign') {
    act(() => useWindows.getState().open('saves'));
    return screen.getByRole('button', { name: 'Encerrar sessão e voltar ao menu' });
  }
  if (path === 'locked') {
    fireEvent.click(screen.getByRole('button', { name: 'Bloquear sessão' }));
    return within(screen.getByRole('dialog', { name: 'Sessão bloqueada' })).getByRole('button', {
      name: 'Encerrar sessão',
    });
  }
  return screen.getByRole('button', { name: 'Sair da sessão' });
}

describe('desktop session exits with real store and mocked native IPC', () => {
  it.each<ExitPath>(['panel', 'launcher', 'campaign', 'locked'])(
    '%s waits for end_session AND normalized refresh before navigation',
    async (path) => {
      const end = deferred<null>();
      const refresh = deferred<World>();
      endReply = () => end.promise;
      worldReply = () => refresh.promise;
      useWindows.getState().open('editor', '/home/kali/unsaved.txt');
      const onMenu = vi.fn(() => {
        expect(useWindows.getState().windows).toEqual([]);
        expect(useGame.getState().world?.flags).toEqual(['NORMALIZED']);
      });
      render(<Desktop onMenu={onMenu} />);
      const button = logoutButton(path);
      const windows = useWindows.getState().windows;
      fireEvent.click(button);
      fireEvent.click(button);
      await waitFor(() => expect(sessionCommands()).toEqual(['end_session']));
      expect(onMenu).not.toHaveBeenCalled();
      expect(useWindows.getState().windows).toEqual(windows);
      await act(async () => {
        end.resolve(null);
        await end.promise;
      });
      expect(commands()).toContain('world_get');
      expect(onMenu).not.toHaveBeenCalled();
      expect(useWindows.getState().windows).toEqual(windows);
      await act(async () => {
        refresh.resolve({ ...backendWorld, flags: ['NORMALIZED'] });
        await refresh.promise;
      });
      await waitFor(() => expect(onMenu).toHaveBeenCalledOnce());
      expect(useGame.getState().sessionPending).toBe(false);
    },
  );

  it.each<ExitPath>(['panel', 'launcher', 'campaign', 'locked'])(
    '%s preserves the session on failure and supports retry',
    async (path) => {
      endReply = () => Promise.reject(new Error('save failed'));
      useWindows.getState().open('editor', '/home/kali/unsaved.txt');
      const onMenu = vi.fn();
      render(<Desktop onMenu={onMenu} />);
      const button = logoutButton(path);
      const windows = useWindows.getState().windows;
      const world = useGame.getState().world;
      fireEvent.click(button);
      expect(await screen.findByRole('alert')).toHaveTextContent('save failed');
      expect(onMenu).not.toHaveBeenCalled();
      expect(useWindows.getState().windows).toEqual(windows);
      expect(useGame.getState().world).toEqual(world);
      expect(commands()).not.toContain('world_get');
      expect(useGame.getState().sessionPending).toBe(false);
      endReply = () => Promise.resolve(null);
      fireEvent.click(button);
      await waitFor(() => expect(onMenu).toHaveBeenCalledOnce());
      expect(sessionCommands()).toEqual(['end_session', 'end_session']);
    },
  );

  it('blocks navigation and keeps windows if post-end refresh fails', async () => {
    worldReply = () => Promise.reject(new Error('refresh failed'));
    useWindows.getState().open('editor');
    const onMenu = vi.fn();
    render(<Desktop onMenu={onMenu} />);
    fireEvent.click(logoutButton('panel'));
    expect(await screen.findByRole('alert')).toHaveTextContent('refresh failed');
    expect(onMenu).not.toHaveBeenCalled();
    expect(useWindows.getState().windows.map((w) => w.id)).toEqual(['editor']);
  });

  it.each(['panel', 'launcher'])(
    'locks and unlocks through %s without ending or starting a session',
    (path) => {
      useWindows.getState().open('editor', '/home/kali/unsaved.txt');
      useWindows.getState().newTerminal();
      const windows = useWindows.getState().windows;
      render(<Desktop onMenu={vi.fn()} />);
      if (path === 'launcher') {
        fireEvent.click(screen.getByRole('button', { name: 'Abrir menu de aplicativos' }));
      }
      fireEvent.click(screen.getAllByRole('button', { name: 'Bloquear sessão' }).at(-1)!);
      expect(screen.getByRole('dialog', { name: 'Sessão bloqueada' })).toBeInTheDocument();
      fireEvent.click(screen.getByRole('button', { name: 'Desbloquear' }));
      expect(screen.queryByRole('dialog', { name: 'Sessão bloqueada' })).not.toBeInTheDocument();
      expect(useWindows.getState().windows).toEqual(windows);
      expect(ipc).not.toHaveBeenCalled();
    },
  );
});

async function showLogin() {
  render(<App />);
  fireEvent.click(await screen.findByRole('button', { name: 'Carregar campanha' }));
  await screen.findByRole('main', { name: 'Login do Kali Linux' });
}
function login() {
  fireEvent.change(screen.getByLabelText('Senha'), { target: { value: 'secret' } });
  fireEvent.click(screen.getByRole('button', { name: /^Entrar$/ }));
}

function finishSessionIntro() {
  fireEvent.animationEnd(screen.getByRole('dialog', { name: /^Sessão \d+:/ }));
}

describe('App session boundaries', () => {
  it.each(['Cancelar', 'Voltar ao menu do jogo'])(
    '%s waits for end_session and blocks on failure',
    async (label) => {
      await showLogin();
      const end = deferred<null>();
      endReply = () => end.promise;
      if (label !== 'Cancelar') {
        fireEvent.click(screen.getByRole('button', { name: 'Opções de sessão' }));
      }
      fireEvent.click(screen.getByRole('button', { name: label }));
      await waitFor(() => expect(sessionCommands()).toEqual(['end_session', 'end_session']));
      expect(screen.getByRole('main', { name: 'Login do Kali Linux' })).toBeInTheDocument();
      expect(screen.queryByRole('navigation', { name: 'Menu do jogo' })).not.toBeInTheDocument();
      await act(async () => {
        end.reject(new Error('cannot save'));
        await end.promise.catch(() => undefined);
      });
      expect(await screen.findByRole('alert')).toHaveTextContent('cannot save');
      expect(screen.getByLabelText('Senha')).toBeInTheDocument();
      endReply = () => Promise.resolve(null);
      fireEvent.click(screen.getByRole('button', { name: 'Cancelar' }));
      expect(await screen.findByRole('navigation', { name: 'Menu do jogo' })).toBeInTheDocument();
    },
  );

  it('requires successful end_session before navigating to another login', async () => {
    endReply = () => Promise.reject(new Error('end rejected'));
    render(<App />);
    fireEvent.click(await screen.findByRole('button', { name: 'Carregar campanha' }));
    await waitFor(() => expect(useGame.getState().error).toContain('end rejected'));
    expect(screen.getByRole('navigation', { name: 'Menu do jogo' })).toBeInTheDocument();
    expect(screen.queryByLabelText('Senha')).not.toBeInTheDocument();
    endReply = () => Promise.resolve(null);
    fireEvent.click(screen.getByRole('button', { name: 'Carregar campanha' }));
    expect(await screen.findByLabelText('Senha')).toBeInTheDocument();
  });

  it('waits for backend session_start and refresh before resetting windows and applying autostart', async () => {
    await showLogin();
    act(() => {
      useWindows.getState().open('editor', '/home/kali/stale.txt');
      useWindows.getState().newTerminal();
      useWindows.getState().newTerminal();
      useWindows.getState().update('editor', { x: 999, minimized: true, maximized: true });
      useWindows.setState({ workspace: 4 });
    });
    const windows = useWindows.getState().windows;
    const start = deferred<World>();
    const refresh = deferred<World>();
    startReply = () => start.promise;
    worldReply = () => refresh.promise;
    backendWorld.settings.autostartApps = '["files","terminal","files","unknown","forum",4]';
    login();
    login();
    await waitFor(() => expect(sessionCommands()).toEqual(['end_session', 'session_start']));
    expect(screen.queryByTestId('desktop')).not.toBeInTheDocument();
    expect(useWindows.getState().windows).toEqual(windows);
    await act(async () => {
      start.resolve(backendWorld);
      await start.promise;
    });
    expect(screen.queryByTestId('desktop')).not.toBeInTheDocument();
    expect(useWindows.getState().windows).toEqual(windows);
    await act(async () => {
      refresh.resolve(backendWorld);
      await refresh.promise;
    });
    expect(await screen.findByTestId('desktop')).toBeInTheDocument();
    expect(screen.getByRole('dialog', { name: 'Sessão 1: Script Kiddie' })).toBeInTheDocument();
    expect(screen.getByTestId('desktop')).toHaveAttribute('inert');
    fireEvent.keyDown(window, { key: 't', ctrlKey: true, altKey: true });
    expect(useWindows.getState().workspace).toBe(1);
    expect(useWindows.getState().windows.map((w) => w.id)).toEqual(['files', 'terminal']);
    expect(
      useWindows.getState().windows.every((w) => !w.path && !w.minimized && !w.maximized),
    ).toBe(true);
    finishSessionIntro();
    expect(screen.queryByRole('dialog', { name: /^Sessão \d+:/ })).not.toBeInTheDocument();
    expect(screen.getByTestId('desktop')).not.toHaveAttribute('inert');
  });

  it('preserves windows and stays at login after session_start fails, then retries', async () => {
    await showLogin();
    act(() => useWindows.getState().open('editor'));
    startReply = () => Promise.reject(new Error('start rejected'));
    login();
    expect(await screen.findByRole('alert')).toHaveTextContent('start rejected');
    expect(screen.queryByTestId('desktop')).not.toBeInTheDocument();
    expect(screen.queryByRole('dialog', { name: /^Sessão \d+:/ })).not.toBeInTheDocument();
    expect(useWindows.getState().windows.map((w) => w.id)).toEqual(['editor']);
    startReply = () => Promise.resolve(backendWorld);
    login();
    expect(await screen.findByTestId('desktop')).toBeInTheDocument();
    expect(useWindows.getState().windows).toEqual([]);
  });

  it('ends a desktop session before menu, then starts a clean session after another login', async () => {
    await showLogin();
    login();
    await screen.findByTestId('desktop');
    finishSessionIntro();
    act(() => {
      useWindows.getState().open('editor', '/home/kali/old.txt');
      useWindows.getState().newTerminal();
    });
    fireEvent.click(logoutButton('panel'));
    await screen.findByRole('navigation', { name: 'Menu do jogo' });
    expect(useWindows.getState().windows).toEqual([]);
    fireEvent.click(screen.getByRole('button', { name: 'Carregar campanha' }));
    await screen.findByLabelText('Senha');
    login();
    await screen.findByTestId('desktop');
    finishSessionIntro();
    expect(useWindows.getState().windows).toEqual([]);
    expect(sessionCommands()).toEqual([
      'end_session',
      'session_start',
      'end_session',
      'end_session',
      'session_start',
    ]);
  });

  it('routes direct desktop entry through backend session_start', async () => {
    useWindows.getState().open('editor');
    render(<App />);
    fireEvent.click(await screen.findByRole('button', { name: 'Entrada direta' }));
    expect(await screen.findByTestId('desktop')).toBeInTheDocument();
    expect(sessionCommands()).toEqual(['session_start']);
    expect(useWindows.getState().windows).toEqual([]);
  });

  it('starts a new campaign through session_start and allows retry from its narrative', async () => {
    render(<App />);
    fireEvent.click(await screen.findByRole('button', { name: 'Nova campanha' }));
    await screen.findByRole('button', { name: 'Entrar na história' });
    startReply = () => Promise.reject(new Error('start rejected'));
    const now = vi.spyOn(performance, 'now').mockReturnValue(performance.now() + 1000);
    fireEvent.click(screen.getByRole('button', { name: 'Entrar na história' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('start rejected');
    expect(screen.queryByTestId('desktop')).not.toBeInTheDocument();
    startReply = () => Promise.resolve(backendWorld);
    now.mockReturnValue(performance.now() + 1000);
    fireEvent.click(screen.getByRole('button', { name: 'Entrar na história' }));
    expect(await screen.findByTestId('desktop')).toBeInTheDocument();
    expect(sessionCommands()).toEqual(['session_start', 'session_start']);
  });
});

describe('session helper validation', () => {
  it.each([undefined, 'invalid-json', '{}', 'null', '["forum",42,null]'])(
    'starts with no windows for unsupported autostart %s',
    async (value) => {
      if (value !== undefined) {
        backendWorld.settings.autostartApps = value;
      }
      useWindows.getState().open('editor');
      const navigate = vi.fn();
      await startSession(navigate);
      expect(useWindows.getState().windows).toEqual([]);
      expect(navigate).toHaveBeenCalledOnce();
    },
  );
  it('rejects a non-unit end_session response without clearing windows or navigating', async () => {
    endReply = () => Promise.resolve(backendWorld);
    useWindows.getState().open('editor');
    const navigate = vi.fn();
    await expect(endSession(navigate)).rejects.toThrow();
    expect(navigate).not.toHaveBeenCalled();
    expect(useWindows.getState().windows.map((w) => w.id)).toEqual(['editor']);
  });
});
