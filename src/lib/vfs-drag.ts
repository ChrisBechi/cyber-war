import type { VfsNode } from './api';
import { emptySchema } from './api';
import { associationForPath } from './file-associations';
import { perform, useGame } from './game-store';
import { softwareById } from './software-catalog';
import { apps, useWindows, windowApp } from './window-store';
import type { AppId, WindowId } from './window-store';
import { isTrashPath } from './vfs-paths';
import { clearVfsDragPreview, showVfsDragPreview } from './vfs-drag-preview';

export const vfsDragType = 'application/x-cyber-war-vfs';
const selectionType = 'application/x-cyber-war-vfs-selection';
const trashRoot = '/home/kali/.local/share/Trash';
const insideTrash = (path: string) => path === trashRoot || path.startsWith(`${trashRoot}/`);
let draggingPaths: string[] = [];
export type VfsDropTarget =
  | { kind: 'folder'; path: string; asRoot?: boolean }
  | { kind: 'trash'; asRoot?: boolean }
  | { kind: 'app'; app: AppId; windowId?: WindowId; asRoot?: boolean };

export function endVfsDrag() {
  draggingPaths = [];
  clearVfsDragPreview();
  document.removeEventListener('dragend', endVfsDrag);
  document.removeEventListener('drop', blockUnhandledDrop);
  document.removeEventListener('dragover', blockUnhandledDrag);
}

function blockUnhandledDrop(event: globalThis.DragEvent) {
  blockUnhandledDrag(event);
  endVfsDrag();
}

function blockUnhandledDrag(event: globalThis.DragEvent) {
  if (!event.defaultPrevented && event.dataTransfer && isVfsDrag(event.dataTransfer)) {
    event.preventDefault();
    event.dataTransfer.dropEffect = 'none';
  }
}

export function startVfsDrag(data: DataTransfer, paths: string[], source?: HTMLElement) {
  endVfsDrag();
  draggingPaths = [...paths];
  data.effectAllowed = 'copyMove';
  data.dropEffect = 'none';
  data.setData(vfsDragType, paths[0]);
  data.setData(selectionType, JSON.stringify(paths));
  data.setData('text/plain', paths[0]);
  document.addEventListener('dragend', endVfsDrag);
  document.addEventListener('drop', blockUnhandledDrop);
  document.addEventListener('dragover', blockUnhandledDrag);
  if (source) {
    showVfsDragPreview(data, source, paths.length);
  }
}

export function isVfsDrag(data: DataTransfer) {
  return Array.from(data.types).includes(vfsDragType);
}

export function virtualFileAddress(path: string) {
  return `file://${path.split('/').map(encodeURIComponent).join('/')}`;
}

export function dropTargetForNode(node: VfsNode, asRoot = false): VfsDropTarget | null {
  if (insideTrash(node.id)) {
    return null;
  }
  if (node.kind === 'directory') {
    return { kind: 'folder', path: node.id, asRoot };
  }
  const entry = node.metadata.launcherId ? softwareById.get(node.metadata.launcherId) : undefined;
  const app = entry?.builtin || node.metadata.app;
  if (app && app in apps) {
    return { kind: 'app', app: app as AppId, asRoot };
  }
  return null;
}

function supportsDroppedFile(node: VfsNode, app: AppId) {
  const association = associationForPath(node.id, node);
  const isText = !node.blob && ['text', 'script', 'unknown'].includes(association.kind);
  return (
    app === 'files' ||
    (node.kind === 'file' &&
      ((app === 'editor' && isText) ||
        (['browser', 'tor-browser'].includes(app) &&
          (isText || ['image', 'audio', 'video'].includes(association.kind))) ||
        (app === association.application && association.support !== 'unsupported')))
  );
}

export function openDroppedFile(node: VfsNode, target: Extract<VfsDropTarget, { kind: 'app' }>) {
  const app = target.app;
  if (!supportsDroppedFile(node, app)) {
    throw new Error(`${apps[app]?.title ?? 'Este aplicativo'} não abre ${node.name}.`);
  }
  const state = useWindows.getState();
  const id =
    target.windowId ?? (app === 'terminal' ? state.newTerminal({ asRoot: target.asRoot }) : app);
  const path = ['browser', 'tor-browser'].includes(app)
    ? virtualFileAddress(node.id)
    : app === 'files' && node.kind !== 'directory'
      ? (node.parentId ?? '/')
      : node.id;
  state.open(id, path);
  if (['editor', 'files', 'archive-viewer'].includes(windowApp(id))) {
    state.update(id, { asRoot: target.asRoot ?? false });
  }
}

function readDraggedPaths(data: DataTransfer): string[] {
  const raw: unknown = data.getData(selectionType)
    ? JSON.parse(data.getData(selectionType))
    : [data.getData(vfsDragType)];
  if (
    !Array.isArray(raw) ||
    !raw.length ||
    raw.length > 100 ||
    !raw.every((path): path is string => typeof path === 'string')
  ) {
    return [];
  }
  return raw;
}

/** Drag data is protected during dragover; keep only the current internal selection in memory. */
export function canDropVfsItems(data: DataTransfer, target: VfsDropTarget | null) {
  if (!target || !isVfsDrag(data) || useGame.getState().sessionPending) {
    return false;
  }
  try {
    return acceptsVfsDrop(draggingPaths.length ? draggingPaths : readDraggedPaths(data), target);
  } catch {
    return false;
  }
}

function acceptsVfsDrop(raw: string[], target: VfsDropTarget): boolean {
  if (!raw.length || raw.length > 100) {
    return false;
  }
  const paths = [...new Set(raw)].filter(
    (path, _, all) => !all.some((parent) => parent !== path && path.startsWith(`${parent}/`)),
  );
  const nodes = useGame.getState().world?.vfs.nodes ?? {};
  const actor = target.asRoot ? 'root' : 'kali';
  const allowed = (node: VfsNode | undefined, bits: number) =>
    !!node &&
    (actor === 'root' ||
      ((node.mode >> (actor === node.owner ? 6 : actor === node.group ? 3 : 0)) & bits) === bits);
  const accessible = (node: VfsNode): boolean => {
    let parent = node.parentId;
    while (parent) {
      const ancestor = nodes[parent];
      if (!ancestor || ancestor.kind !== 'directory' || !allowed(ancestor, 1)) {
        return false;
      }
      parent = parent === '/' ? null : ancestor.parentId;
    }
    return true;
  };
  const sources = paths.map((path) => nodes[path]);
  if (sources.some((node) => !node || insideTrash(node.id) || !accessible(node))) {
    return false;
  }
  if (target.kind === 'app') {
    const node = sources[0];
    return (
      sources.length === 1 &&
      supportsDroppedFile(node, target.app) &&
      allowed(
        node.kind === 'directory'
          ? node
          : target.app === 'files'
            ? nodes[node.parentId ?? '/']
            : node,
        node.kind === 'directory' || target.app === 'files' ? 5 : 4,
      )
    );
  }
  const names = new Set<string>();
  if (target.kind === 'folder') {
    const folder = nodes[target.path];
    if (
      !folder ||
      folder.kind !== 'directory' ||
      insideTrash(target.path) ||
      !allowed(folder, 3) ||
      !accessible(folder)
    ) {
      return false;
    }
  }
  let moving = 0;
  for (const node of sources) {
    if (node.id === '/' || !allowed(nodes[node.parentId ?? '/'], 3)) {
      return false;
    }
    if (target.kind === 'folder') {
      if (target.path === node.id || target.path.startsWith(`${node.id}/`)) {
        return false;
      }
      if (node.parentId === target.path) {
        continue;
      }
      const destination = `${target.path.replace(/\/$/, '')}/${node.name}`;
      if (nodes[destination] || names.has(node.name)) {
        return false;
      }
      names.add(node.name);
    } else if (trashRoot.startsWith(`${node.id}/`)) {
      return false;
    }
    // The backend checks the entire subtree before moving it, including trash moves.
    if (
      Object.values(nodes).some(
        (child) =>
          (child.id === node.id || child.id.startsWith(`${node.id}/`)) &&
          (!allowed(child, child.kind === 'directory' ? 5 : 4) || !accessible(child)),
      )
    ) {
      return false;
    }
    moving++;
  }
  return moving > 0;
}

export async function dropVfsItems(data: DataTransfer, target: VfsDropTarget) {
  if (!isVfsDrag(data) || useGame.getState().sessionPending) {
    return;
  }
  let paths: string[];
  try {
    paths = readDraggedPaths(data);
    if (!acceptsVfsDrop(paths, target)) {
      return;
    }
  } catch {
    return;
  }
  await transferVfsItems(paths, target);
}

export async function transferVfsItems(raw: string[], target: VfsDropTarget) {
  const paths = [...new Set(raw)].filter(
    (path, _, all) => !all.some((parent) => parent !== path && path.startsWith(`${parent}/`)),
  );
  const nodes = useGame.getState().world?.vfs.nodes ?? {};
  const sources = paths.map((path) => {
    const node = nodes[path];
    if (!node || isTrashPath(path)) {
      throw new Error(
        'O item não está disponível. Restaure itens da lixeira antes de arrastá-los.',
      );
    }
    return node;
  });
  if (target.kind === 'app') {
    if (sources.length !== 1) {
      throw new Error('Solte um arquivo por vez sobre o aplicativo.');
    }
    openDroppedFile(sources[0], target);
    return;
  }
  const moving =
    target.kind === 'folder' ? sources.filter((node) => node.parentId !== target.path) : sources;
  if (target.kind === 'folder') {
    if (isTrashPath(target.path) || nodes[target.path]?.kind !== 'directory') {
      throw new Error('A pasta de destino não está disponível.');
    }
    const destinations = new Set<string>();
    for (const node of moving) {
      if (target.path === node.id || target.path.startsWith(`${node.id}/`)) {
        throw new Error('Uma pasta não pode ser movida para dentro dela mesma.');
      }
      const destination = `${target.path.replace(/\/$/, '')}/${node.name}`;
      if (nodes[destination] || destinations.has(destination)) {
        throw new Error(`Já existe um item chamado ${node.name} no destino.`);
      }
      destinations.add(destination);
    }
  }
  let completed = 0;
  try {
    for (const node of moving) {
      await perform(
        target.kind === 'trash' ? 'vfs_remove' : 'vfs_move',
        target.kind === 'trash'
          ? { path: node.id, recursive: true, asRoot: target.asRoot ?? false }
          : { source: node.id, destination: target.path, asRoot: target.asRoot ?? false },
        emptySchema,
      );
      completed++;
    }
  } catch (error) {
    throw new Error(`${completed ? `${completed} item(ns) movido(s). ` : ''}${String(error)}`);
  }
}
