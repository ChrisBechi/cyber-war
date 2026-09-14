import { describe, expect, it } from 'vitest';
import {
  capacity,
  createPartitions,
  partitionSchemes,
  validatePartitions,
} from './installer-model';
import type { SystemSetupValues } from './installer-model';

describe('virtual partition plans', () => {
  it.each(['nvme0n1', 'sda'] as const)(
    'fits every guided scheme on %s and includes the requested mount points',
    (disk) => {
      for (const method of [
        'guided-largest',
        'guided-disk',
        'guided-lvm',
        'guided-encrypted',
      ] as const) {
        for (const [partitionScheme] of partitionSchemes) {
          const values = {
            disk,
            partition: method,
            partitionScheme,
            storage: 'plain',
          } as SystemSetupValues;
          values.partitions = createPartitions(values);
          expect(validatePartitions(values)).toBe('');
          expect(values.partitions.reduce((sum, p) => sum + p.size, 0)).toBeLessThanOrEqual(
            capacity(values) + 0.001,
          );
          const mounts = values.partitions.map((p) => p.mount);
          if (partitionScheme === 'var-tmp') {
            expect(mounts).toEqual(['/boot/efi', '/', '/home', '/var', '/tmp', 'swap']);
          }
          if (partitionScheme === 'server') {
            expect(mounts).toEqual(['/boot/efi', '/', '/var', '/srv', 'swap']);
          }
        }
      }
    },
  );
  it('rejects missing roots, duplicate mounts and oversized custom partitions', () => {
    const values = {
      disk: 'sda',
      partition: 'manual',
      storage: 'plain',
      partitions: [{ size: 1, fs: 'ESP', mount: '/boot/efi' }],
    } as SystemSetupValues;
    expect(validatePartitions(values)).toContain('raiz');
    values.partitions.push({ size: 100, fs: 'ext4', mount: '/' });
    expect(validatePartitions(values)).toContain('ultrapassam');
    values.partitions[1].size = 20;
    values.partitions.push({ size: 10, fs: 'ext4', mount: '/' });
    expect(validatePartitions(values)).toContain('apenas uma');
  });
});
