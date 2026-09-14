import { describe, expect, it } from 'vitest';
import type { VfsNode } from './api';
import { sortVfsNodes } from './vfs-order';

const node = (name: string, kind: VfsNode['kind'], modifiedAt: number): VfsNode => ({
  id: `/home/kali/${name}`,
  parentId: '/home/kali',
  name,
  kind,
  content: '',
  owner: 'kali',
  group: 'kali',
  mode: kind === 'directory' ? 0o755 : 0o644,
  modifiedAt,
  metadata: {},
});

describe('sortVfsNodes', () => {
  it('keeps folders before files and compares natural names', () => {
    const result = sortVfsNodes([
      node('file10.txt', 'file', 3),
      node('file2.txt', 'file', 2),
      node('Projects', 'directory', 1),
    ]);
    expect(result.map((item) => item.name)).toEqual(['Projects', 'file2.txt', 'file10.txt']);
  });

  it('persists the requested field and direction through a pure sort', () => {
    const result = sortVfsNodes(
      [node('old.txt', 'file', 1), node('new.txt', 'file', 2)],
      'modified',
      'desc',
      false,
    );
    expect(result.map((item) => item.name)).toEqual(['new.txt', 'old.txt']);
  });
});
