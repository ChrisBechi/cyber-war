import { create } from 'zustand';
import { emptySchema } from './api';
import { perform } from './game-store';

// This is the game's file clipboard, never the host operating system clipboard.
export const useVfsClipboard = create<{
  entry: { path: string; cut: boolean } | null;
  copy: (path: string, cut?: boolean) => void;
  clear: () => void;
}>((set) => ({
  entry: null,
  copy: (path, cut = false) => set({ entry: { path, cut } }),
  clear: () => set({ entry: null }),
}));

export async function pasteVfs(directory: string, asRoot = false) {
  const entry = useVfsClipboard.getState().entry;
  if (!entry) {
    return;
  }
  await perform(
    entry.cut ? 'vfs_move' : 'vfs_copy',
    {
      source: entry.path,
      destination: directory,
      asRoot,
    },
    emptySchema,
  );
  if (entry.cut && useVfsClipboard.getState().entry === entry) {
    useVfsClipboard.getState().clear();
  }
}
