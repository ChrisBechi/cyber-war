import { emptySchema } from './api';
import { associationForPath } from './file-associations';
import { perform } from './game-store';

export async function importVirtualFile(file: File, directory: string, asRoot = false) {
  if (file.size > 32 * 1024 * 1024) {
    throw new Error('Limite de importação: 32 MiB por arquivo.');
  }
  if (
    !file.name ||
    /[\\/:]/.test(file.name) ||
    Array.from(file.name).some((char) => char.charCodeAt(0) < 32 || char.charCodeAt(0) === 127) ||
    file.name === '.' ||
    file.name === '..'
  ) {
    throw new Error('Nome de arquivo inválido.');
  }
  const path = `${directory.replace(/\/$/, '')}/${file.name}`;
  const bytes = new Uint8Array(await file.arrayBuffer());
  const association = associationForPath(path);
  if (['text', 'script'].includes(association.kind)) {
    const content = new TextDecoder('utf-8', { fatal: true }).decode(bytes);
    await perform('vfs_write', { path, content, expectedContent: null, asRoot }, emptySchema);
  } else {
    let raw = '';
    for (let i = 0; i < bytes.length; i += 8192) {
      raw += String.fromCharCode(...bytes.subarray(i, i + 8192));
    }
    const mime =
      file.type ||
      (
        {
          png: 'image/png',
          jpg: 'image/jpeg',
          jpeg: 'image/jpeg',
          wav: 'audio/wav',
          mp3: 'audio/mpeg',
          ogg: 'audio/ogg',
          webm: 'video/webm',
          mp4: 'video/mp4',
        } as Record<string, string>
      )[file.name.split('.').pop()?.toLowerCase() ?? ''] ||
      'application/octet-stream';
    await perform(
      'vfs_import_bytes',
      { path, base64: btoa(raw), mime, expectedModified: null, asRoot },
      emptySchema,
    );
  }
}
