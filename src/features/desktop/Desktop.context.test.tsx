import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { VfsNode, World } from '../../lib/api';
import { useGame } from '../../lib/game-store';
import { useWindows, windowApp } from '../../lib/window-store';
import { pasteVfs, useVfsClipboard } from '../../lib/vfs-clipboard';
import { openVfsNode, scriptCommand } from '../../lib/file-open';
import { Desktop } from './Desktop';
import { DesktopContextMenu } from './DesktopContextMenu';

const ipc = vi.hoisted(() =>
  vi.fn<(command: string, args: Record<string, unknown>) => Promise<unknown>>(),
);
vi.mock('@tauri-apps/api/core', () => ({ invoke: ipc, isTauri: () => true }));
vi.mock('../../lib/audio-manager', () => ({
  audioManager: {
    play: vi.fn(),
    getVolumes: () => ({ master: 70, music: 70, voice: 70, sfx: 70 }),
    setVolumes: vi.fn(),
  },
}));
vi.mock('../files/FileManager', () => ({
  FileManager: ({ asRoot }: { asRoot?: boolean }) => <div>Arquivos {asRoot ? 'root' : 'kali'}</div>,
}));
vi.mock('../files/Editor', () => ({ Editor: () => null }));
vi.mock('../media/ImageViewer', () => ({ ImageViewer: () => null }));
vi.mock('../media/MediaPlayer', () => ({ MediaPlayer: () => null }));
vi.mock('../terminal/TerminalWindow', () => ({ TerminalWindow: () => null }));
vi.mock('./Applications', () => ({
  Saves: () => null,
  Browser: () => null,
  CodeLab: () => null,
  Forum: () => null,
  Journey: () => null,
  Messages: () => null,
  Missions: () => null,
  Processes: () => null,
  Settings: () => null,
}));

const desktopPath = '/home/kali/Desktop';
const node = (name: string, parent = desktopPath): VfsNode => ({
  id: `${parent}/${name}`,
  parentId: parent,
  name,
  kind: 'file',
  content: '',
  owner: 'kali',
  group: 'kali',
  mode: 0o644,
  modifiedAt: 1,
  metadata: {},
});
let world: World;
beforeEach(() => {
  vi.clearAllMocks();
  world = {
    nickname: 'kali',
    hostname: 'lifeos',
    session: 1,
    money: 0,
    reputation: 0,
    playtimeSeconds: 0,
    vfs: {
      nodes: Object.fromEntries(
        [node('Zeta.txt'), node('Alfa.txt'), node('origem.txt', '/home/kali/Documents')].map(
          (item) => [item.id, item],
        ),
      ),
    },
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
    settings: {},
  };
  ipc.mockImplementation((command, args) => {
    if (command === 'world_get') {
      return Promise.resolve(structuredClone(world));
    }
    if (command === 'mission_get_state') {
      return Promise.resolve([]);
    }
    if (command === 'setting_update') {
      world.settings[String(args.key)] = String(args.value);
    }
    return Promise.resolve(null);
  });
  useGame.setState({
    world,
    missions: [],
    error: '',
    busy: false,
    sessionPending: false,
    revision: 0,
  });
  useWindows.getState().reset();
  useVfsClipboard.getState().clear();
});
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});
function menu() {
  fireEvent.contextMenu(screen.getByTestId('desktop'), { clientX: 320, clientY: 180 });
  return screen.getByRole('menu', { name: 'Menu da área de trabalho' });
}
function select(label: string) {
  fireEvent.click(screen.getByRole('menuitem', { name: label }));
}

function press(target: Element) {
  fireEvent.pointerDown(target);
  fireEvent.click(target);
}

describe('desktop context menu', () => {
  it.each([
    ['Escolher terminal', 'terminal'],
    ['Rede', 'network'],
    ['Áudio', 'audio'],
    ['Energia', 'power'],
    ['Calendário', 'calendar'],
  ])(
    'dismisses %s outside its popup and trigger, even when propagation is stopped',
    (label, id) => {
      const outsideClick = vi.fn();
      const { container } = render(
        <>
          <Desktop onMenu={() => undefined} />
          <button
            onPointerDown={(event) => event.stopPropagation()}
            onClick={(event) => {
              event.stopPropagation();
              outsideClick();
            }}
          >
            Outra janela
          </button>
        </>,
      );
      const trigger = screen.getByRole('button', { name: label });
      const opened = () => expect(trigger).toHaveAttribute('aria-expanded', 'true');
      const closed = () => {
        expect(trigger).toHaveAttribute('aria-expanded', 'false');
        expect(container.querySelector(`.panel-popover-${id}`)).toBeNull();
      };
      press(trigger);
      opened();
      press(container.querySelector(`.panel-popover-${id}`)!);
      opened();
      fireEvent.pointerDown(trigger);
      opened();
      fireEvent.click(trigger);
      closed();
      press(trigger);
      // Another control on the same bar is outside, and its action must still run.
      press(screen.getByRole('button', { name: 'Área de trabalho 2' }));
      closed();
      expect(useWindows.getState().workspace).toBe(2);
      press(trigger);
      press(screen.getByRole('button', { name: 'Outra janela' }));
      closed();
      expect(outsideClick).toHaveBeenCalledOnce();
      press(trigger);
      // Assistive/keyboard activation need not emit pointerdown first.
      fireEvent.click(document.body);
      closed();
    },
  );

  it('keeps the audio popup open while adjusting its controls', () => {
    render(<Desktop onMenu={() => undefined} />);
    const trigger = screen.getByRole('button', { name: 'Áudio' });
    press(trigger);
    const slider = screen.getByRole('slider', { name: 'Volume da sessão' });
    press(slider);
    fireEvent.change(slider, { target: { value: '32' } });
    expect(slider).toHaveValue('32');
    expect(trigger).toHaveAttribute('aria-expanded', 'true');
  });

  it('dismisses the desktop menu outside, but preserves its submenu and the outside action', () => {
    const outsideClick = vi.fn();
    render(
      <>
        <Desktop onMenu={() => undefined} />
        <button onPointerDown={(event) => event.stopPropagation()} onClick={outsideClick}>
          Fora do desktop
        </button>
      </>,
    );
    menu();
    press(screen.getByRole('menuitem', { name: 'Criar documento' }));
    press(screen.getByRole('menu', { name: 'Criar documento' }));
    expect(screen.getByRole('menuitem', { name: 'Script Shell (.sh)' })).toBeInTheDocument();
    press(screen.getByRole('button', { name: 'Fora do desktop' }));
    expect(screen.queryByRole('menu')).toBeNull();
    expect(outsideClick).toHaveBeenCalledOnce();
    menu();
    fireEvent.click(document.body);
    expect(screen.queryByRole('menu')).toBeNull();
  });

  it('switches between bar menus and dismisses the launcher outside without reopening its trigger', () => {
    render(
      <>
        <Desktop onMenu={() => undefined} />
        <button
          onPointerDown={(event) => event.stopPropagation()}
          onClick={(event) => event.stopPropagation()}
        >
          Fora do menu
        </button>
      </>,
    );
    const network = screen.getByRole('button', { name: 'Rede' });
    const audio = screen.getByRole('button', { name: 'Áudio' });
    const launcher = screen.getByRole('button', { name: 'Abrir menu de aplicativos' });
    press(network);
    press(audio);
    expect(network).toHaveAttribute('aria-expanded', 'false');
    expect(audio).toHaveAttribute('aria-expanded', 'true');
    press(launcher);
    expect(audio).toHaveAttribute('aria-expanded', 'false');
    press(screen.getByRole('textbox', { name: 'Pesquisar aplicativos' }));
    expect(launcher).toHaveAttribute('aria-expanded', 'true');
    press(screen.getByRole('button', { name: 'Fora do menu' }));
    expect(screen.queryByRole('navigation', { name: 'Aplicativos Kali' })).toBeNull();
    press(launcher);
    press(launcher);
    expect(launcher).toHaveAttribute('aria-expanded', 'false');
    press(launcher);
    fireEvent.click(document.body);
    expect(launcher).toHaveAttribute('aria-expanded', 'false');
    press(launcher);
    press(network);
    expect(launcher).toHaveAttribute('aria-expanded', 'false');
    expect(network).toHaveAttribute('aria-expanded', 'true');
  });

  it('quotes script filenames containing spaces, quotes or shell punctuation as a single argument', () => {
    expect(scriptCommand('/home/kali/Desktop/meu script.sh')).toBe(
      "bash '/home/kali/Desktop/meu script.sh'",
    );
    expect(scriptCommand("/home/kali/Desktop/it's;$test.sh")).toBe(
      "bash '/home/kali/Desktop/it'\"'\"'s;$test.sh'",
    );
  });
  it('has the reference options in order, disables an empty clipboard, and ignores application windows', () => {
    render(<Desktop onMenu={() => undefined} />);
    expect(
      within(menu())
        .getAllByRole('menuitem')
        .map((item) => item.textContent),
    ).toEqual([
      'Criar lançador…',
      'Criar link de URL…',
      'Criar pasta…',
      'Criar documento▸',
      'Colar',
      'Abrir terminal aqui',
      'Abrir como root',
      'Abrir em nova janela',
      'Organizar ícones da área de trabalho',
      'Configurações da área de trabalho…',
      'Aplicativos▸',
    ]);
    expect(screen.getByRole('menuitem', { name: 'Colar' })).toBeDisabled();
    fireEvent.keyDown(document.activeElement!, { key: 'Escape' });
    expect(screen.queryByRole('menu')).not.toBeInTheDocument();
    act(() => useWindows.getState().open('files'));
    fireEvent.contextMenu(screen.getByRole('dialog', { name: 'Arquivos' }));
    expect(screen.queryByRole('menu')).not.toBeInTheDocument();
  });
  it('opens separate local file windows and a root window without changing the shared terminal', () => {
    render(<Desktop onMenu={() => undefined} />);
    menu();
    select('Abrir em nova janela');
    menu();
    select('Abrir em nova janela');
    menu();
    select('Abrir como root');
    expect(
      useWindows.getState().windows.map((item) => [windowApp(item.id), item.path, item.asRoot]),
    ).toEqual([
      ['files', desktopPath, false],
      ['files', desktopPath, false],
      ['files', desktopPath, true],
    ]);
    expect(world.terminal.user).toBe('kali');
  });
  it('opens a terminal in Desktop and opens settings', () => {
    render(<Desktop onMenu={() => undefined} />);
    menu();
    select('Abrir terminal aqui');
    expect(useWindows.getState().windows[0]).toMatchObject({ id: 'terminal', cwd: desktopPath });
    menu();
    select('Configurações da área de trabalho…');
    expect(useWindows.getState().windows.some((item) => item.id === 'settings')).toBe(true);
  });
  it('pastes from the virtual file clipboard and keeps cut state when paste fails', async () => {
    useVfsClipboard.getState().copy('/home/kali/Documents/origem.txt');
    render(<Desktop onMenu={() => undefined} />);
    menu();
    select('Colar');
    await waitFor(() =>
      expect(ipc).toHaveBeenCalledWith('vfs_copy', {
        source: '/home/kali/Documents/origem.txt',
        destination: desktopPath,
        asRoot: false,
      }),
    );
    await waitFor(() => expect(useGame.getState().busy).toBe(false));
    act(() => useVfsClipboard.getState().copy('/home/kali/Documents/origem.txt', true));
    ipc.mockRejectedValueOnce(new Error('destination already exists'));
    await act(async () => {
      await expect(pasteVfs(desktopPath)).rejects.toThrow('destination already exists');
    });
    expect(useVfsClipboard.getState().entry?.cut).toBe(true);
    await act(async () => {
      await pasteVfs(desktopPath);
    });
    expect(useVfsClipboard.getState().entry).toBeNull();
  });
  it('persists icon sorting and exposes copy/cut on an icon only', async () => {
    render(<Desktop onMenu={() => undefined} />);
    menu();
    select('Organizar ícones da área de trabalho');
    await waitFor(() => expect(world.settings.desktopSort).toBe('name'));
    await waitFor(() => expect(useGame.getState().busy).toBe(false));
    const icons = screen.getByTestId('desktop').querySelectorAll('[data-vfs-path]');
    expect(Array.from(icons).map((item) => item.textContent)).toEqual(['Alfa.txt', 'Zeta.txt']);
    fireEvent.contextMenu(icons[0]);
    select('Recortar');
    expect(useVfsClipboard.getState().entry).toEqual({
      path: `${desktopPath}/Alfa.txt`,
      cut: true,
    });
  });
  it.each([
    ['Criar pasta…', 'folder', 'Projetos'],
    ['Criar lançador…', 'launcher', 'Atalho'],
    ['Criar link de URL…', 'url', 'Biblioteca'],
    ['Arquivo vazio', 'file', 'Notas.txt'],
    ['Script Shell (.sh)', 'shell', 'teste.sh'],
  ])('creates %s through the transactional desktop action', async (label, kind, name) => {
    render(<Desktop onMenu={() => undefined} />);
    menu();
    if (kind === 'file' || kind === 'shell') {
      select('Criar documento');
    }
    select(label);
    fireEvent.change(screen.getByRole('textbox', { name: 'Nome' }), { target: { value: name } });
    fireEvent.click(screen.getByRole('button', { name: 'Criar' }));
    await waitFor(() =>
      expect(ipc).toHaveBeenCalledWith('desktop_create', expect.objectContaining({ kind, name })),
    );
    await waitFor(() => expect(screen.queryByRole('dialog')).not.toBeInTheDocument());
  });
  it('keeps the creation form and draft after an IPC error', async () => {
    render(<Desktop onMenu={() => undefined} />);
    menu();
    select('Criar pasta…');
    ipc.mockRejectedValueOnce(new Error('EEXIST'));
    fireEvent.change(screen.getByRole('textbox', { name: 'Nome' }), {
      target: { value: 'Já existe' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Criar' }));
    await waitFor(() =>
      expect(within(screen.getByRole('dialog')).getByRole('alert')).toHaveTextContent('EEXIST'),
    );
    expect(screen.getByRole('textbox', { name: 'Nome' })).toHaveValue('Já existe');
  });
  it('uses keyboard navigation, skips disabled items and keeps the menu on screen', () => {
    const close = vi.fn();
    render(
      <DesktopContextMenu
        x={9999}
        y={9999}
        onClose={close}
        items={[
          { label: 'Indisponível', disabled: true },
          { label: 'Documentos', children: [{ label: 'Texto' }] },
          { label: 'Último' },
        ]}
      />,
    );
    expect(document.activeElement).toHaveTextContent('Documentos');
    fireEvent.keyDown(document.activeElement!, { key: 'ArrowRight' });
    expect(document.activeElement).toHaveTextContent('Texto');
    fireEvent.keyDown(document.activeElement!, { key: 'ArrowLeft' });
    expect(document.activeElement).toHaveTextContent('Documentos');
    fireEvent.keyDown(document.activeElement!, { key: 'ArrowUp' });
    expect(document.activeElement).toHaveTextContent('Último');
    const panel = screen.getByRole('menu');
    expect(Number.parseFloat(panel.style.left)).toBeLessThan(window.innerWidth);
    expect(Number.parseFloat(panel.style.top)).toBeLessThan(window.innerHeight);
    fireEvent.pointerDown(document.body);
    expect(close).toHaveBeenCalledOnce();
  });
  it('opens desktop URLs only in the virtual browser, and catalog launchers in their own terminal', async () => {
    const link = node('Biblioteca.desktop');
    link.metadata.url = 'https://wipedia.org';
    await openVfsNode(link);
    expect(useWindows.getState().windows[0]).toMatchObject({
      id: 'browser',
      path: 'https://wipedia.org',
    });
    const shortcut = node('Admin.desktop');
    shortcut.metadata.launcherId = 'root-terminal';
    await openVfsNode(shortcut);
    expect(useWindows.getState().windows[1]).toMatchObject({ id: 'terminal', asRoot: true });
    expect(ipc).toHaveBeenCalledWith('launcher_open', { id: 'root-terminal' });
  });
});
