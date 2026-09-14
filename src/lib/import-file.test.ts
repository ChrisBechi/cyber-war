import { beforeEach, expect, it, vi } from 'vitest';
import { importVirtualFile } from './import-file';

const perform = vi.hoisted(() => vi.fn());
vi.mock('./game-store', () => ({ perform }));
beforeEach(() => perform.mockReset());
function file(name: string, bytes: number[], type: string) {
  return {
    name,
    type,
    size: bytes.length,
    arrayBuffer: () => Promise.resolve(Uint8Array.from(bytes).buffer),
  } as File;
}
it('imports exact bytes with create-only semantics and keeps captions textual', async () => {
  await importVirtualFile(file('picture.png', [0, 255, 128], 'image/png'), '/home/kali/Pictures');
  expect(perform).toHaveBeenCalledWith(
    'vfs_import_bytes',
    expect.objectContaining({
      base64: 'AP+A',
      path: '/home/kali/Pictures/picture.png',
      expectedModified: null,
    }),
    expect.anything(),
  );
  await importVirtualFile(
    file('caption.srt', [65, 10], 'application/x-subrip'),
    '/home/kali/Videos',
  );
  expect(perform).toHaveBeenLastCalledWith(
    'vfs_write',
    expect.objectContaining({ content: 'A\n', expectedContent: null }),
    expect.anything(),
  );
});
it('rejects oversize, invalid UTF-8 text, traversal and reports IPC failure', async () => {
  await expect(
    importVirtualFile({ ...file('x.png', [], ''), size: 33554433 }, '/tmp'),
  ).rejects.toThrow('32 MiB');
  await expect(importVirtualFile(file('x.txt', [255], ''), '/tmp')).rejects.toThrow();
  await expect(importVirtualFile(file('../x.png', [], ''), '/tmp')).rejects.toThrow();
  expect(perform).not.toHaveBeenCalled();
  perform.mockRejectedValueOnce(new Error('file exists'));
  await expect(importVirtualFile(file('x.png', [0], ''), '/tmp')).rejects.toThrow('file exists');
});
