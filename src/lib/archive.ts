import { z } from 'zod';
import { perform, useGame } from './game-store';
import { request, emptySchema } from './api';

export const archiveFormats = [
  'ZIP',
  'TAR',
  'GZIP',
  'BZIP2',
  'XZ',
  'TAR_GZIP',
  'TAR_BZIP2',
  'TAR_XZ',
] as const;
export type ArchiveFormat = (typeof archiveFormats)[number];
export const archiveFormatLabel = (format: ArchiveFormat) =>
  ({
    ZIP: 'ZIP',
    TAR: 'TAR',
    GZIP: 'GZIP',
    BZIP2: 'BZIP2',
    XZ: 'XZ',
    TAR_GZIP: 'TAR.GZ',
    TAR_BZIP2: 'TAR.BZ2',
    TAR_XZ: 'TAR.XZ',
  })[format];
export const archiveEntrySchema = z.object({
  path: z.string(),
  type: z.enum(['file', 'directory', 'symlink']),
  originalSize: z.number(),
  compressedSize: z.number().nullable(),
  permissions: z.number(),
  modifiedAt: z.number(),
  owner: z.string().nullable(),
  group: z.string().nullable(),
  crc: z.number().nullable(),
  linkTarget: z.string().nullable(),
  encrypted: z.boolean(),
});
export const archiveInspectionSchema = z.object({
  path: z.string(),
  format: z.enum(archiveFormats),
  integrity: z.enum(['VALID', 'PARTIAL', 'CORRUPTED', 'ENCRYPTED', 'UNSUPPORTED']),
  encrypted: z.boolean(),
  compressedSize: z.number(),
  storageSize: z.number(),
  originalSize: z.number(),
  modifiedAt: z.number(),
  entries: z.array(archiveEntrySchema),
  createdAt: z.number().optional(),
});
export type ArchiveInspection = z.infer<typeof archiveInspectionSchema>;
export type ArchiveEntry = z.infer<typeof archiveEntrySchema>;
const responseSchema = z.object({
  inspection: archiveInspectionSchema.nullable(),
  entries: z.array(z.string()),
  error: z.string().nullable(),
});
export const archiveJobSchema = z.object({
  id: z.number(),
  pid: z.number(),
  name: z.string(),
  progress: z.number(),
  status: z.string(),
  stdout: z.string(),
  stderr: z.string(),
  exitCode: z.number(),
  response: responseSchema.nullable(),
});
export type ArchiveJob = z.infer<typeof archiveJobSchema>;
export const pollArchiveJobs = () => request('archive_jobs_tick', {}, z.array(archiveJobSchema));
export const cancelArchiveJob = (id: number) => request('archive_job_cancel', { id }, emptySchema);
export async function waitArchiveJob(id: number, onProgress?: (job: ArchiveJob) => void) {
  for (;;) {
    const job = (await pollArchiveJobs()).find((job) => job.id === id);
    if (!job) {
      throw new Error('Operação não está mais disponível.');
    }
    onProgress?.(job);
    if (job.status !== 'Running') {
      await useGame.getState().refresh();
      return job;
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
}
export async function startArchiveOperation(
  payload: ArchiveRequest,
  onProgress?: (job: ArchiveJob) => void,
) {
  const nodes = Object.values(useGame.getState().world?.vfs.nodes ?? {});
  const size = nodes
    .filter(
      (node) =>
        node.id === payload.path ||
        payload.inputs?.some((path) => node.id === path || node.id.startsWith(`${path}/`)),
    )
    .reduce(
      (total, node) =>
        total +
        Number(
          node.metadata.archiveOriginalSize ??
            node.metadata.logicalSize ??
            node.blob?.size ??
            node.content.length,
        ),
      0,
    );
  if (size < 8 * 1024 * 1024) {
    return archiveOperation(payload);
  }
  const job = await perform('archive_job_start', { request: payload }, archiveJobSchema);
  onProgress?.(job);
  const done = await waitArchiveJob(job.id, onProgress);
  if (done.exitCode || done.status === 'Cancelled') {
    throw new Error(done.stderr || 'Operação cancelada.');
  }
  if (!done.response) {
    throw new Error('Resposta da operação indisponível.');
  }
  return done.response;
}
export type ArchiveRequest = {
  operation: 'inspect' | 'create' | 'extract' | 'test' | 'decompress';
  path: string;
  format?: ArchiveFormat;
  inputs?: string[];
  password?: string;
  asRoot?: boolean;
  options?: {
    destination: string;
    overwrite: 'ask' | 'skip' | 'replace';
    selected: string[];
    decisions?: Record<string, boolean>;
  };
};
export async function archiveOperation(request: ArchiveRequest) {
  const result = await perform('archive_operation', { request }, responseSchema);
  if (result.error) {
    throw new Error(result.error);
  }
  return result;
}
export function archiveSize(value: number) {
  const units = ['B', 'KiB', 'MiB', 'GiB'];
  let i = 0;
  while (value >= 1024 && i < units.length - 1) {
    value /= 1024;
    i++;
  }
  return `${value.toLocaleString('pt-BR', { maximumFractionDigits: i ? 1 : 0 })} ${units[i]}`;
}
export function archiveChildren(entries: ArchiveEntry[], directory: string) {
  const prefix = directory ? `${directory}/` : '';
  const children = new Map<string, ArchiveEntry>();
  for (const entry of entries) {
    if (!entry.path.startsWith(prefix)) {
      continue;
    }
    const suffix = entry.path.slice(prefix.length);
    if (!suffix) {
      continue;
    }
    const name = suffix.split('/')[0];
    const path = prefix + name;
    children.set(
      path,
      suffix.includes('/')
        ? (entries.find((e) => e.path === path) ?? {
            ...entry,
            path,
            type: 'directory',
            originalSize: 0,
            compressedSize: null,
            linkTarget: null,
          })
        : entry,
    );
  }
  return [...children.values()].sort(
    (a, b) =>
      Number(b.type === 'directory') - Number(a.type === 'directory') ||
      a.path.localeCompare(b.path),
  );
}
