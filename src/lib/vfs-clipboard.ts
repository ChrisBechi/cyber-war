import { create } from 'zustand';
import { emptySchema } from './api';
import { perform } from './game-store';

// This is the game's file clipboard, never the host operating system clipboard.
export const useVfsClipboard = create<{
  entry: { path: string; cut: boolean; paths?: string[] } | null;
  copy: (path: string | string[], cut?: boolean) => void;
  clear: () => void;
}>((set) => ({
  entry: null,
  copy: (path, cut = false) => {
    const paths = [...new Set(Array.isArray(path) ? path : [path])];
    set({
      entry: paths.length ? { path: paths[0], cut, ...(paths.length > 1 ? { paths } : {}) } : null,
    });
  },
  clear: () => set({ entry: null }),
}));

export async function pasteVfs(directory: string, asRoot = false) {
  let entry = useVfsClipboard.getState().entry;
  if (!entry) {
    return;
  }
  const paths = (entry.paths ?? [entry.path]).filter(
    (path, _, all) => !all.some((parent) => parent !== path && path.startsWith(`${parent}/`)),
  );
  let remaining = paths;
  for (const source of paths) {
    await perform(
      entry.cut ? 'vfs_move' : 'vfs_copy',
      {
        source,
        destination: directory,
        asRoot,
      },
      emptySchema,
    );
    if (entry.cut && useVfsClipboard.getState().entry === entry) {
      remaining = remaining.filter((path) => path !== source);
      if (remaining.length) {
        // Keep only pending items after a partially completed cut/paste.
        entry = { path: remaining[0], cut: true, paths: remaining };
        useVfsClipboard.setState({ entry });
      }
    }
  }
  if (entry.cut && useVfsClipboard.getState().entry === entry) {
    useVfsClipboard.getState().clear();
  }
}
