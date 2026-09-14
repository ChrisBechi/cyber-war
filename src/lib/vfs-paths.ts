export const TRASH_FILES_PATH = '/home/kali/.local/share/Trash/files';

export function isTrashPath(path: string): boolean {
  return path === TRASH_FILES_PATH || path.startsWith(`${TRASH_FILES_PATH}/`);
}
