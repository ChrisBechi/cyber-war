import type { VfsNode } from './api';

export type VfsSort = 'name' | 'modified' | 'kind';
export type VfsSortDirection = 'asc' | 'desc';

export const defaultVfsSort: VfsSort = 'name';
export const defaultVfsSortDirection: VfsSortDirection = 'asc';
export const defaultVfsFoldersFirst = true;

const collator = new Intl.Collator('pt-BR', { numeric: true, sensitivity: 'base' });

export function sortVfsNodes(
  nodes: VfsNode[],
  sort: VfsSort = defaultVfsSort,
  direction: VfsSortDirection = defaultVfsSortDirection,
  foldersFirst = defaultVfsFoldersFirst,
): VfsNode[] {
  const multiplier = direction === 'desc' ? -1 : 1;
  return [...nodes].sort((left, right) => {
    if (foldersFirst && left.kind !== right.kind) {
      return left.kind === 'directory' ? -1 : 1;
    }
    const comparison =
      sort === 'modified'
        ? left.modifiedAt - right.modifiedAt
        : sort === 'kind'
          ? collator.compare(left.kind, right.kind) || collator.compare(left.name, right.name)
          : collator.compare(left.name, right.name);
    return (comparison || collator.compare(left.id, right.id)) * multiplier;
  });
}
