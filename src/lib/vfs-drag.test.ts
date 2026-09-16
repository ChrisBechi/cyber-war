import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { perform, useGame } from './game-store';
import type * as Store from './game-store';
import type { VfsNode, World } from './api';
import { useWindows } from './window-store';
import {
  canDropVfsItems,
  dropVfsItems,
  endVfsDrag,
  startVfsDrag,
  transferVfsItems,
} from './vfs-drag';
import { pasteVfs, useVfsClipboard } from './vfs-clipboard';

vi.mock('./game-store', async (original) => ({
  ...(await original<typeof Store>()),
  perform: vi.fn(),
}));
const node = (id: string, kind: VfsNode['kind'] = 'file'): VfsNode => ({
  id,
  name: id.split('/').pop()!,
  parentId: id.slice(0, id.lastIndexOf('/')) || '/',
  kind,
  content: '',
  metadata: {},
  owner: 'kali',
  group: 'kali',
  mode: 0o755,
  modifiedAt: 1,
});
beforeEach(() => {
  const nodes = [
    { ...node('/', 'directory'), parentId: null },
    node('/home', 'directory'),
    node('/home/folder', 'directory'),
    node('/home/source', 'directory'),
    node('/home/a.txt'),
    node('/home/b.txt'),
    node('/home/picture.png'),
    node('/home/movie.mp4'),
  ];
  useGame.setState({
    world: { vfs: { nodes: Object.fromEntries(nodes.map((item) => [item.id, item])) } } as World,
    sessionPending: false,
  });
  useWindows.getState().reset();
  useVfsClipboard.getState().clear();
  vi.mocked(perform).mockReset().mockResolvedValue(null);
});
afterEach(endVfsDrag);

const dragData = () => {
  const entries = new Map<string, string>();
  return {
    setData: (key: string, value: string) => entries.set(key, value),
    getData: (key: string) => entries.get(key) ?? '',
    get types() {
      return [...entries.keys()];
    },
    setDragImage: vi.fn(),
  } as unknown as DataTransfer;
};

it('previews supported targets even while the browser protects drag data, then forgets the selection on cancellation', () => {
  const data = dragData();
  startVfsDrag(data, ['/home/a.txt']);
  const protectedData = { types: data.types, getData: () => '' } as unknown as DataTransfer;
  expect(canDropVfsItems(protectedData, { kind: 'folder', path: '/home/folder' })).toBe(true);
  expect(canDropVfsItems(protectedData, { kind: 'app', app: 'browser' })).toBe(true);
  expect(canDropVfsItems(protectedData, { kind: 'app', app: 'image-viewer' })).toBe(false);
  expect(canDropVfsItems(protectedData, { kind: 'folder', path: '/home' })).toBe(false);
  document.dispatchEvent(new Event('dragend'));
  expect(canDropVfsItems(protectedData, { kind: 'trash' })).toBe(false);
});

it('silently ignores incompatible apps, recursive folders, missing items and denied permissions', async () => {
  const data = dragData();
  startVfsDrag(data, ['/home/a.txt']);
  await dropVfsItems(data, { kind: 'app', app: 'image-viewer' });
  useGame.getState().world!.vfs.nodes['/home/folder'].mode = 0o555;
  expect(canDropVfsItems(data, { kind: 'folder', path: '/home/folder' })).toBe(false);
  await dropVfsItems(data, { kind: 'folder', path: '/home/folder' });
  expect(canDropVfsItems(data, { kind: 'folder', path: '/home/folder', asRoot: true })).toBe(true);
  startVfsDrag(data, ['/home/source']);
  await dropVfsItems(data, { kind: 'folder', path: '/home/source' });
  startVfsDrag(data, ['/home/missing.txt']);
  await dropVfsItems(data, { kind: 'trash' });
  expect(useWindows.getState().windows).toHaveLength(0);
  expect(perform).not.toHaveBeenCalled();
});

it('blocks the whole selection on collision or a protected child without moving any other item', async () => {
  const data = dragData();
  const nodes = useGame.getState().world!.vfs.nodes;
  nodes['/home/folder/a.txt'] = node('/home/folder/a.txt');
  startVfsDrag(data, ['/home/a.txt', '/home/b.txt']);
  expect(canDropVfsItems(data, { kind: 'folder', path: '/home/folder' })).toBe(false);
  await dropVfsItems(data, { kind: 'folder', path: '/home/folder' });
  nodes['/home/source/locked'] = { ...node('/home/source/locked'), mode: 0 };
  startVfsDrag(data, ['/home/b.txt', '/home/source']);
  expect(canDropVfsItems(data, { kind: 'trash' })).toBe(false);
  await dropVfsItems(data, { kind: 'trash' });
  expect(perform).not.toHaveBeenCalled();
});

it('uses the actual icon and label in the ghost and cleans it up on cancellation', () => {
  const source = document.createElement('button');
  source.innerHTML = '<img class="app-icon" src="/assets/kali/firefox.svg"><span>Navegador</span>';
  const data = dragData();
  const setDragImage = vi.fn();
  data.setDragImage = setDragImage;
  startVfsDrag(data, ['/home/a.txt', '/home/b.txt'], source);
  const ghost = document.querySelector('.vfs-drag-preview');
  expect(ghost?.querySelector('img')).toHaveAttribute('src', '/assets/kali/firefox.svg');
  expect(ghost?.querySelector('.vfs-drag-preview__name')).toHaveTextContent('Navegador');
  expect(ghost?.querySelector('.vfs-drag-preview__count')).toHaveTextContent('2');
  expect(setDragImage).toHaveBeenCalledWith(ghost, 20, 20);
  document.dispatchEvent(new Event('dragend'));
  expect(document.querySelector('.vfs-drag-preview')).toBeNull();
});

it('moves selected files into folders and sends them to the recoverable trash with the target permissions', async () => {
  await transferVfsItems(['/home/a.txt', '/home/b.txt'], {
    kind: 'folder',
    path: '/home/folder',
    asRoot: true,
  });
  expect(perform).toHaveBeenNthCalledWith(
    1,
    'vfs_move',
    { source: '/home/a.txt', destination: '/home/folder', asRoot: true },
    expect.anything(),
  );
  expect(perform).toHaveBeenNthCalledWith(
    2,
    'vfs_move',
    { source: '/home/b.txt', destination: '/home/folder', asRoot: true },
    expect.anything(),
  );
  await transferVfsItems(['/home/a.txt'], { kind: 'trash' });
  expect(perform).toHaveBeenLastCalledWith(
    'vfs_remove',
    { path: '/home/a.txt', recursive: true, asRoot: false },
    expect.anything(),
  );
});

it('rejects duplicate destinations and recursive moves before changing any file', async () => {
  useGame.getState().world!.vfs.nodes['/home/folder/a.txt'] = node('/home/folder/a.txt');
  await expect(
    transferVfsItems(['/home/b.txt', '/home/a.txt'], { kind: 'folder', path: '/home/folder' }),
  ).rejects.toThrow('Já existe');
  await expect(
    transferVfsItems(['/home/folder'], { kind: 'folder', path: '/home/folder' }),
  ).rejects.toThrow('dela mesma');
  expect(perform).not.toHaveBeenCalled();
});

it('opens compatible applications and browser file addresses without moving or executing text', async () => {
  await transferVfsItems(['/home/a.txt'], { kind: 'app', app: 'editor' });
  await transferVfsItems(['/home/picture.png'], { kind: 'app', app: 'image-viewer' });
  await transferVfsItems(['/home/movie.mp4'], { kind: 'app', app: 'media-player' });
  await transferVfsItems(['/home/a.txt'], { kind: 'app', app: 'browser' });
  expect(useWindows.getState().windows.map(({ id, path }) => [id, path])).toEqual([
    ['editor', '/home/a.txt'],
    ['image-viewer', '/home/picture.png'],
    ['media-player', '/home/movie.mp4'],
    ['browser', 'file:///home/a.txt'],
  ]);
  await expect(
    transferVfsItems(['/home/picture.png'], { kind: 'app', app: 'editor' }),
  ).rejects.toThrow('não abre');
  expect(perform).not.toHaveBeenCalled();
});

it('only accepts the internal drag format and serializes a multi-file selection', async () => {
  const entries = new Map<string, string>();
  const data = {
    types: [],
    setData: (key: string, value: string) => entries.set(key, value),
    getData: (key: string) => entries.get(key) ?? '',
  } as unknown as DataTransfer;
  await dropVfsItems(data, { kind: 'trash' });
  expect(perform).not.toHaveBeenCalled();
  startVfsDrag(data, ['/home/a.txt', '/home/b.txt']);
  Object.defineProperty(data, 'types', { value: [...entries.keys()] });
  await dropVfsItems(data, { kind: 'trash' });
  expect(perform).toHaveBeenCalledTimes(2);
});

it('keeps only pending cut items after a partial failure and finishes them on retry', async () => {
  useVfsClipboard.getState().copy(['/home/a.txt', '/home/b.txt'], true);
  vi.mocked(perform)
    .mockResolvedValueOnce(null)
    .mockRejectedValueOnce(new Error('permission denied'));
  await expect(pasteVfs('/home/folder')).rejects.toThrow('permission denied');
  expect(useVfsClipboard.getState().entry).toMatchObject({
    path: '/home/b.txt',
    paths: ['/home/b.txt'],
    cut: true,
  });
  await pasteVfs('/home/folder');
  expect(useVfsClipboard.getState().entry).toBeNull();
  expect(perform).toHaveBeenCalledTimes(3);
});
