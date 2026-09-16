import { z } from 'zod';
import { nodeSchema, request, type BrowserPage } from '../../lib/api';
import { associationForPath } from '../../lib/file-associations';

export type BrowserLocalFile = {
  path: string;
  kind: 'text' | 'image' | 'audio' | 'video';
  text?: string;
};
export async function loadBrowserFile(
  address: string,
): Promise<{ page: BrowserPage; localFile: BrowserLocalFile }> {
  const url = new URL(address);
  if (url.protocol !== 'file:' || url.host || url.search || url.hash) {
    throw new Error('Use um caminho de arquivo do computador virtual.');
  }
  const path = decodeURIComponent(url.pathname);
  if (!path.startsWith('/') || path.includes('\\') || path.includes('\0')) {
    throw new Error('Caminho virtual inválido.');
  }
  const node = await request('vfs_stat', { path }, nodeSchema);
  if (node.kind !== 'file') {
    throw new Error('Abra pastas no gerenciador de arquivos.');
  }
  const association = associationForPath(path, node);
  if (['image', 'audio', 'video'].includes(association.kind)) {
    return {
      page: { title: node.name, body: '', action: null },
      localFile: { path, kind: association.kind as 'image' | 'audio' | 'video' },
    };
  }
  if (node.blob || !['text', 'script', 'unknown'].includes(association.kind)) {
    throw new Error('Este formato não possui prévia no navegador.');
  }
  const text = await request('vfs_read', { path }, z.string());
  return {
    page: { title: node.name, body: text, action: null },
    localFile: { path, kind: 'text', text },
  };
}
