import { z } from 'zod';
import { request } from './api';
import type { VfsNode } from './api';
import { perform } from './game-store';
import { useWindows } from './window-store';

export const packageInspectionSchema = z.object({
  name: z.string(),
  version: z.string(),
  architecture: z.string(),
  description: z.string(),
  installedSize: z.number(),
  downloadSize: z.number(),
  dependencies: z.array(z.string()),
  origin: z.string(),
  status: z.string(),
  valid: z.boolean(),
});
export type PackageInspection = z.infer<typeof packageInspectionSchema>;
export const packagePlanSchema = z.object({ id: z.string(), summary: z.string() });
export const packageResultSchema = z.object({
  stdout: z.string(),
  stderr: z.string(),
  exitCode: z.number(),
  job: z.number().nullable(),
});
export const inspectPackage = (path: string) =>
  request('package_inspect', { path }, packageInspectionSchema);
export const planPackage = (path: string, operation: 'install' | 'remove' | 'purge') =>
  perform('package_plan', { path, operation }, packagePlanSchema);
export const confirmPackage = (id: string, accept: boolean) =>
  perform('package_confirm', { id, accept }, packageResultSchema);

export function packageDesktopEntries(
  nodes: Record<string, VfsNode>,
  installed: Record<string, { status: string }> = {},
) {
  return Object.values(nodes)
    .filter(
      (node) =>
        node.id.startsWith('/usr/share/applications/') &&
        node.name.endsWith('.desktop') &&
        node.kind === 'file',
    )
    .flatMap((node) => {
      const name = node.content
        .split('\n')
        .find((line) => line.startsWith('Name='))
        ?.slice(5);
      const command = node.content
        .split('\n')
        .find((line) => line.startsWith('Exec='))
        ?.slice(5);
      return name &&
        command &&
        nodes[`/usr/bin/${command}`] &&
        installed[node.metadata.packageOwner]?.status === 'installed'
        ? [{ name, command, path: node.id }]
        : [];
    });
}
export async function launchPackage(path: string) {
  const command = await request('package_launch', { path }, z.string());
  const windows = useWindows.getState();
  const id = windows.newTerminal();
  windows.update(id, { initialCommand: command });
}
