import type { VfsNode } from './api';
import { emptySchema } from './api';
import { associationForPath } from './file-associations';
import { perform } from './game-store';
import { softwareById } from './software-catalog';
import type { SoftwareEntry } from './software-catalog';
import { apps, useWindows } from './window-store';
import type { AppId } from './window-store';
import { isTrashPath } from './vfs-paths';
import { launchPackage } from './packages';

export function scriptCommand(path: string): string {
  // Adjacent quoted segments are understood by the virtual tokenizer too.
  return `bash '${path.replaceAll("'", "'\"'\"'")}'`;
}

export async function launchSoftware(entry: SoftwareEntry) {
  await perform('launcher_open', { id: entry.id }, emptySchema);
  const windows = useWindows.getState();
  if (entry.builtin === 'root-terminal' || entry.builtin === 'terminal') {
    windows.newTerminal({ asRoot: entry.builtin === 'root-terminal' });
  } else if (entry.builtin && entry.builtin in apps) {
    windows.open(entry.builtin as AppId);
  } else {
    windows.open(`tool:${entry.id}`);
  }
}

export async function openVfsNode(node: VfsNode, asRoot = false) {
  if (isTrashPath(node.id)) {
    throw new Error('Restaure o item da lixeira antes de abrir.');
  }
  const windows = useWindows.getState();
  if (node.kind === 'directory') {
    windows.newInstance('files', node.id, asRoot);
    return;
  }
  if (node.id.startsWith('/usr/share/applications/') && node.name.endsWith('.desktop')) {
    await launchPackage(node.id);
    return;
  }
  if (node.metadata.launcherId) {
    const entry = softwareById.get(node.metadata.launcherId);
    if (!entry) {
      throw new Error('Aplicativo do atalho não está no catálogo virtual.');
    }
    await launchSoftware(entry);
    return;
  }
  if (node.metadata.url) {
    // Browser only resolves game addresses through IPC; never window.open or a native opener.
    windows.open('browser', node.metadata.url);
    return;
  }
  if (node.metadata.app && node.metadata.app in apps) {
    if (node.metadata.app === 'terminal') {
      windows.newTerminal();
    } else {
      windows.open(node.metadata.app as AppId);
    }
    return;
  }
  const association = associationForPath(node.id, node);
  if (association.support === 'unsupported') {
    throw new Error(`${association.label}. O arquivo foi preservado.`);
  }
  if (association.application === 'terminal') {
    const id = windows.newTerminal({ asRoot });
    windows.open(id, node.id);
  } else if (association.application === 'editor' && asRoot) {
    windows.newInstance('editor', node.id, true);
  } else {
    windows.open(association.application, node.id);
    if (association.application === 'archive-viewer') {
      windows.update('archive-viewer', { asRoot });
    }
  }
}
