import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FileManager } from './FileManager';
import { request, type VfsNode, type World } from '../../lib/api';
import type * as Api from '../../lib/api';
import { perform, useGame } from '../../lib/game-store';
import type * as Store from '../../lib/game-store';
import { useVfsClipboard } from '../../lib/vfs-clipboard';
import { endVfsDrag, startVfsDrag } from '../../lib/vfs-drag';

vi.mock('../../lib/api', async (original) => ({
  ...(await original<typeof Api>()),
  request: vi.fn(),
}));
vi.mock('../../lib/game-store', async (original) => ({
  ...(await original<typeof Store>()),
  perform: vi.fn(),
}));
const directory = '/home/kali/Documents';
let nodes: VfsNode[];
const makeNode = (name: string, kind: VfsNode['kind'] = 'file'): VfsNode => ({
  id: `${directory}/${name}`,
  parentId: directory,
  name,
  kind,
  content: 'hello',
  metadata: {},
  owner: 'kali',
  group: 'kali',
  mode: 0o755,
  modifiedAt: 1,
});
beforeEach(() => {
  nodes = [makeNode('alpha.txt'), makeNode('beta.txt'), makeNode('Pasta', 'directory')];
  useGame.setState({
    world: {
      settings: {},
      vfs: {
        nodes: Object.fromEntries(
          [
            ...['/', '/home', '/home/kali', directory].map((id) => ({
              ...makeNode(id, 'directory'),
              id,
              parentId: id === '/' ? null : id.slice(0, id.lastIndexOf('/')) || '/',
            })),
            ...nodes,
            { ...makeNode('outside.txt'), id: '/home/kali/outside.txt', parentId: '/home/kali' },
          ].map((item) => [item.id, item]),
        ),
      },
    } as World,
    sessionPending: false,
  });
  vi.mocked(request)
    .mockReset()
    .mockImplementation(() => Promise.resolve(nodes));
  vi.mocked(perform).mockReset().mockResolvedValue(null);
  useVfsClipboard.getState().clear();
});
afterEach(() => {
  cleanup();
  endVfsDrag();
});
const key = (name: string, options = {}) =>
  fireEvent.keyDown(screen.getByLabelText('Gerenciador de arquivos'), { key: name, ...options });

it('creates folders and files with keyboard shortcuts and preserves entered names after failure', async () => {
  render(<FileManager initialPath={directory} />);
  await screen.findByRole('button', { name: 'alpha.txt' });
  key('n', { ctrlKey: true, shiftKey: true });
  let dialog = screen.getByRole('dialog', { name: 'Criar pasta' });
  fireEvent.change(within(dialog).getByRole('textbox', { name: 'Nome' }), {
    target: { value: 'Projeto' },
  });
  vi.mocked(perform).mockRejectedValueOnce(new Error('permission denied'));
  fireEvent.click(within(dialog).getByRole('button', { name: 'Criar' }));
  expect(await within(dialog).findByRole('alert')).toHaveTextContent('permission denied');
  expect(within(dialog).getByRole('textbox')).toHaveValue('Projeto');
  fireEvent.click(within(dialog).getByRole('button', { name: 'Criar' }));
  await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  expect(perform).toHaveBeenLastCalledWith(
    'vfs_create_directory',
    { path: `${directory}/Projeto`, asRoot: false },
    expect.anything(),
  );
  key('n', { ctrlKey: true });
  dialog = screen.getByRole('dialog', { name: 'Criar arquivo' });
  fireEvent.click(within(dialog).getByRole('button', { name: 'Criar' }));
  await waitFor(() =>
    expect(perform).toHaveBeenLastCalledWith(
      'vfs_create_file',
      { path: `${directory}/Documento.txt`, asRoot: false },
      expect.anything(),
    ),
  );
  await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
});

it('copies, cuts and pastes the selection while preserving normal text-field editing', async () => {
  render(<FileManager initialPath={directory} />);
  fireEvent.click(await screen.findByRole('button', { name: 'alpha.txt' }));
  key('c', { ctrlKey: true });
  expect(useVfsClipboard.getState().entry).toEqual({ path: `${directory}/alpha.txt`, cut: false });
  fireEvent.click(screen.getByRole('button', { name: 'Buscar arquivos' }));
  const search = screen.getByRole('textbox', { name: 'Buscar arquivos locais' });
  fireEvent.keyDown(search, { key: 'Delete' });
  fireEvent.keyDown(search, { key: 'a', ctrlKey: true });
  expect(perform).not.toHaveBeenCalled();
  key('a', { ctrlKey: true });
  key('x', { ctrlKey: true });
  expect(useVfsClipboard.getState().entry?.paths).toHaveLength(3);
  useVfsClipboard.getState().copy('/home/kali/Downloads/new.txt', true);
  key('v', { ctrlKey: true });
  await waitFor(() =>
    expect(perform).toHaveBeenCalledWith(
      'vfs_move',
      { source: '/home/kali/Downloads/new.txt', destination: directory, asRoot: false },
      expect.anything(),
    ),
  );
});

it('renames the selected file, refuses a duplicate name, and sends Delete to the recoverable trash', async () => {
  render(<FileManager initialPath={directory} />);
  const file = await screen.findByRole('button', { name: 'alpha.txt' });
  fireEvent.click(file);
  key('F2');
  const dialog = screen.getByRole('dialog', { name: 'Renomear item' });
  fireEvent.change(within(dialog).getByRole('textbox'), { target: { value: 'Pasta' } });
  fireEvent.click(within(dialog).getByRole('button', { name: 'Renomear' }));
  expect(await within(dialog).findByRole('alert')).toHaveTextContent('Já existe');
  expect(perform).not.toHaveBeenCalled();
  fireEvent.change(within(dialog).getByRole('textbox'), { target: { value: 'renamed.txt' } });
  fireEvent.click(within(dialog).getByRole('button', { name: 'Renomear' }));
  await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  expect(perform).toHaveBeenCalledWith(
    'vfs_move',
    { source: `${directory}/alpha.txt`, destination: `${directory}/renamed.txt`, asRoot: false },
    expect.anything(),
  );
  key('Delete');
  await waitFor(() =>
    expect(perform).toHaveBeenLastCalledWith(
      'vfs_remove',
      { path: `${directory}/alpha.txt`, recursive: true, asRoot: false },
      expect.anything(),
    ),
  );
});

it('drops on a folder without bubbling to the containing directory and supports the trash sidebar', async () => {
  render(<FileManager initialPath={directory} />);
  const folder = await screen.findByRole('button', { name: 'Pasta' });
  const values = new Map<string, string>();
  const data = {
    setData: (key: string, value: string) => values.set(key, value),
    getData: (key: string) => values.get(key) ?? '',
    get types() {
      return [...values.keys()];
    },
  } as unknown as DataTransfer;
  startVfsDrag(data, [`${directory}/alpha.txt`]);
  fireEvent.dragOver(folder, { dataTransfer: data });
  expect(folder).toHaveAttribute('data-drop-active', 'true');
  fireEvent.drop(folder, { dataTransfer: data });
  await waitFor(() => expect(perform).toHaveBeenCalledTimes(1));
  expect(perform).toHaveBeenCalledWith(
    'vfs_move',
    { source: `${directory}/alpha.txt`, destination: `${directory}/Pasta`, asRoot: false },
    expect.anything(),
  );
  fireEvent.drop(screen.getByRole('button', { name: 'Lixeira' }), {
    dataTransfer: data,
  });
  await waitFor(() => expect(perform).toHaveBeenCalledTimes(2));
});

it('shows a blocked drop effect on ordinary files and inputs without falling through to the containing folder', async () => {
  render(<FileManager initialPath={directory} />);
  const file = await screen.findByRole('button', { name: 'beta.txt' });
  fireEvent.click(screen.getByRole('button', { name: 'Buscar arquivos' }));
  const values = new Map<string, string>();
  const data = {
    setData: (key: string, value: string) => values.set(key, value),
    getData: (key: string) => values.get(key) ?? '',
    get types() {
      return [...values.keys()];
    },
  } as unknown as DataTransfer;
  for (const target of [file, screen.getByRole('textbox', { name: 'Buscar arquivos locais' })]) {
    startVfsDrag(data, ['/home/kali/outside.txt']);
    fireEvent.dragOver(target, { dataTransfer: data });
    expect(data.dropEffect).toBe('none');
    expect(target).not.toHaveAttribute('data-drop-active');
    fireEvent.drop(target, { dataTransfer: data });
  }
  expect(perform).not.toHaveBeenCalled();
  expect(screen.queryByRole('alert')).toBeNull();
  startVfsDrag(data, ['/home/kali/outside.txt']);
  const folder = screen.getByRole('button', { name: 'Pasta' });
  fireEvent.dragOver(folder, { dataTransfer: data });
  expect(data.dropEffect).toBe('move');
  expect(folder).toHaveAttribute('data-drop-active', 'true');
  fireEvent.dragEnd(file, { dataTransfer: data });
  expect(folder).not.toHaveAttribute('data-drop-active');
});

it('navigates back and forward, clears the forward branch after a new destination, and refreshes', async () => {
  render(<FileManager initialPath={directory} />);
  await screen.findByRole('button', { name: 'alpha.txt' });
  expect(screen.getByRole('button', { name: 'Voltar' })).toBeDisabled();
  expect(screen.getByRole('button', { name: 'Avançar' })).toBeDisabled();
  fireEvent.doubleClick(screen.getByRole('button', { name: 'Pasta' }));
  await waitFor(() =>
    expect(request).toHaveBeenLastCalledWith(
      'vfs_list',
      { path: `${directory}/Pasta`, asRoot: false },
      expect.anything(),
    ),
  );
  fireEvent.click(screen.getByRole('button', { name: 'Voltar' }));
  await waitFor(() =>
    expect(request).toHaveBeenLastCalledWith(
      'vfs_list',
      { path: directory, asRoot: false },
      expect.anything(),
    ),
  );
  expect(screen.getByRole('button', { name: 'Avançar' })).toBeEnabled();
  fireEvent.click(screen.getByRole('button', { name: 'Avançar' }));
  await waitFor(() =>
    expect(screen.getByRole('textbox', { name: 'Caminho' })).toHaveValue('Pasta'),
  );
  fireEvent.click(screen.getByRole('button', { name: 'Voltar' }));
  fireEvent.click(screen.getByRole('button', { name: 'Início' }));
  await waitFor(() =>
    expect(request).toHaveBeenLastCalledWith(
      'vfs_list',
      { path: '/home/kali', asRoot: false },
      expect.anything(),
    ),
  );
  expect(screen.getByRole('button', { name: 'Avançar' })).toBeDisabled();
  vi.mocked(request).mockClear();
  fireEvent.click(screen.getByRole('button', { name: 'Atualizar pasta' }));
  await waitFor(() => expect(request).toHaveBeenCalledTimes(1));
});

it('creates from the compact menu and keeps selection actions in the context menu', async () => {
  render(<FileManager initialPath={directory} />);
  await screen.findByRole('button', { name: 'alpha.txt' });
  expect(screen.queryByRole('button', { name: /Nova pasta/ })).toBeNull();
  fireEvent.click(screen.getByRole('button', { name: 'Menu de arquivos' }));
  fireEvent.click(screen.getByRole('button', { name: /Nova pasta/ }));
  expect(screen.getByRole('dialog', { name: 'Criar pasta' })).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Cancelar' }));
  fireEvent.contextMenu(screen.getByRole('button', { name: 'alpha.txt' }));
  fireEvent.click(screen.getByRole('button', { name: 'Copiar Ctrl+C' }));
  expect(useVfsClipboard.getState().entry).toEqual({ path: `${directory}/alpha.txt`, cut: false });
  expect(screen.queryByRole('dialog', { name: 'Ações do item' })).toBeNull();
  fireEvent.click(screen.getByRole('button', { name: 'Exibir em lista' }));
  expect(screen.getByRole('button', { name: 'Exibir em grade' })).toBeInTheDocument();
  expect(screen.getByRole('button', { name: /alpha.txt/ })).toHaveAttribute('aria-pressed', 'true');
});
