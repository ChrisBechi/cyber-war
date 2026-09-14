import { useEffect, useState } from 'react';
import { z } from 'zod';
import { nodeSchema, request, type VfsNode } from '../../lib/api';
import { mediaSourceForPath } from '../../lib/file-associations';

const bytesSchema = z.object({ base64: z.string(), mime: z.string(), size: z.number() });

/** Object URLs belong to this mounted view, never the save or caller metadata. */
export function useMediaSource(path?: string) {
  const [state, setState] = useState<{
    path?: string;
    node: VfsNode | null;
    source: string | null;
    error: string;
  }>({ node: null, source: null, error: '' });
  useEffect(() => {
    let current = true;
    let objectUrl: string | undefined;
    setState({ path, node: null, source: null, error: '' });
    if (!path) {
      return;
    }
    void (async () => {
      const node = await request('vfs_stat', { path }, nodeSchema);
      let source: string | null;
      if (node.blob) {
        const data = await request('vfs_read_bytes', { path }, bytesSchema);
        if (!current) {
          return;
        }
        const decoded = atob(data.base64);
        if (decoded.length !== data.size || data.size > 32 * 1024 * 1024) {
          throw new Error('Conteúdo binário inválido.');
        }
        // SVG/HTML are not treated as passive raster media. Keep arbitrary bytes
        // storable, but do not publish an active document URL as a media preview.
        if (
          !/^(image\/(png|jpeg|gif|webp|bmp|avif|x-icon)|audio\/[a-z0-9.+-]+|video\/[a-z0-9.+-]+)$/.test(
            data.mime,
          )
        ) {
          throw new Error('Formato armazenado, mas prévia não suportada.');
        }
        const bytes = Uint8Array.from(decoded, (char) => char.charCodeAt(0));
        objectUrl = URL.createObjectURL(new Blob([bytes], { type: data.mime }));
        source = objectUrl;
      } else {
        // Stat is not read permission. Even bundled previews must pass VFS read.
        const content = await request('vfs_read', { path }, z.string());
        source = mediaSourceForPath(path, { ...node, content });
      }
      if (current) {
        setState({ path, node, source, error: '' });
      }
    })().catch((error: unknown) => {
      if (current) {
        setState({ path, node: null, source: null, error: String(error) });
      }
    });
    return () => {
      current = false;
      if (objectUrl) {
        URL.revokeObjectURL(objectUrl);
      }
    };
  }, [path]);
  return state.path === path ? state : { node: null, source: null, error: '' };
}
