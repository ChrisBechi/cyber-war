import { describe, expect, it } from 'vitest';
import { vfsSchema } from './api';

const inode = {
  ino: 7,
  kind: 'file',
  content: 'shared',
  owner: 'kali',
  group: 'kali',
  mode: 420,
  modifiedAt: 1,
  metadata: {},
  nlink: 2,
};
describe('virtual inode wire projection', () => {
  it('projects shared payloads into UI paths without following symbolic links', () => {
    const vfs = vfsSchema.parse({
      formatVersion: 2,
      entries: { '/home/kali/a': 7, '/home/kali/b': 7, '/home/kali/link': 8 },
      inodes: { 7: inode, 8: { ...inode, ino: 8, kind: 'symlink', content: 'a', nlink: 1 } },
    });
    expect(vfs.nodes['/home/kali/a'].ino).toBe(vfs.nodes['/home/kali/b'].ino);
    expect(vfs.nodes['/home/kali/b'].name).toBe('b');
    expect(vfs.nodes['/home/kali/b'].parentId).toBe('/home/kali');
    expect(vfs.nodes['/home/kali/link'].content).toBe('a');
    expect(vfs.nodes['/home/kali/link'].kind).toBe('symlink');
  });
  it('accepts legacy preview nodes and character devices', () => {
    const vfs = vfsSchema.parse({
      nodes: {
        '/dev/null': {
          ...inode,
          id: '/dev/null',
          parentId: '/dev',
          name: 'null',
          kind: 'charDevice',
        },
      },
    });
    expect(vfs.nodes['/dev/null'].kind).toBe('charDevice');
  });
  it('rejects missing or mismatched canonical inodes', () => {
    expect(
      vfsSchema.safeParse({ formatVersion: 2, entries: { '/a': 9 }, inodes: { 7: inode } }).success,
    ).toBe(false);
    expect(
      vfsSchema.safeParse({
        formatVersion: 2,
        entries: { '/a': 7 },
        inodes: { 7: { ...inode, ino: 9 } },
      }).success,
    ).toBe(false);
  });
});
