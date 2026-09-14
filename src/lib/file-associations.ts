import type { VfsNode } from './api';

export type FileKind =
  'text' | 'script' | 'audio' | 'video' | 'image' | 'archive' | 'document' | 'unknown';
export type FileApplication = 'editor' | 'terminal' | 'media-player' | 'image-viewer';

export type FileAssociation = {
  kind: FileKind;
  application: FileApplication;
  label: string;
  mime: string;
  support: 'editable' | 'executable-subset' | 'preview-conditional' | 'unsupported';
};

const textExtensions = new Set([
  'txt',
  'md',
  'markdown',
  'log',
  'conf',
  'cfg',
  'ini',
  'json',
  'yaml',
  'yml',
  'xml',
  'csv',
  'html',
  'htm',
  'css',
  'js',
  'jsx',
  'ts',
  'tsx',
  'py',
  'rb',
  'php',
  'c',
  'h',
  'cpp',
  'rs',
  'go',
  'java',
  'sql',
  'service',
  'desktop',
  'vtt',
  'srt',
]);
const audioExtensions = new Set(['mp3', 'ogg', 'oga', 'wav', 'flac', 'aac', 'm4a', 'opus', 'weba']);
const videoExtensions = new Set(['mp4', 'webm', 'ogv', 'mkv', 'mov', 'avi', 'm4v', '3gp']);
const imageExtensions = new Set([
  'png',
  'jpg',
  'jpeg',
  'gif',
  'webp',
  'bmp',
  'svg',
  'avif',
  'ico',
  'tif',
  'tiff',
]);
const archiveExtensions = new Set(['zip', 'tar', 'gz', 'bz2', 'xz', '7z', 'rar', 'deb', 'iso']);

export function extensionOf(path: string): string {
  const name = path.split('/').pop() ?? path;
  const dot = name.lastIndexOf('.');
  return dot > 0 ? name.slice(dot + 1).toLowerCase() : '';
}

export function associationForPath(
  path: string,
  node?: Pick<VfsNode, 'metadata'> & Partial<Pick<VfsNode, 'blob'>>,
): FileAssociation {
  // A renamed binary keeps its declared type; extensions must not override it.
  const extension = node?.blob ? '' : extensionOf(path);
  const mime = node?.blob?.mime ?? node?.metadata.mime ?? '';
  if (node?.blob && !/^(audio|video|image)\//.test(mime)) {
    return {
      kind: 'unknown',
      application: 'editor',
      label: 'Arquivo binário · abertura não suportada',
      mime,
      support: 'unsupported',
    };
  }
  if (
    !node?.blob &&
    (extension === 'sh' || extension === 'bash' || mime === 'application/x-shellscript')
  ) {
    return {
      kind: 'script',
      application: 'terminal',
      label: 'Shell script · Bash',
      mime: 'application/x-shellscript',
      support: 'executable-subset',
    };
  }
  if (audioExtensions.has(extension) || mime.startsWith('audio/')) {
    return {
      kind: 'audio',
      application: 'media-player',
      label: 'Parole Media Player · áudio',
      mime: mime || `audio/${extension || 'ogg'}`,
      support: 'preview-conditional',
    };
  }
  if (videoExtensions.has(extension) || mime.startsWith('video/')) {
    return {
      kind: 'video',
      application: 'media-player',
      label: 'Parole Media Player · vídeo',
      mime: mime || `video/${extension || 'webm'}`,
      support: 'preview-conditional',
    };
  }
  if (imageExtensions.has(extension) || mime.startsWith('image/')) {
    return {
      kind: 'image',
      application: 'image-viewer',
      label: 'Ristretto Image Viewer',
      mime: mime || `image/${extension || 'png'}`,
      support: 'preview-conditional',
    };
  }
  if (
    archiveExtensions.has(extension) ||
    mime === 'application/zip' ||
    mime === 'application/x-tar'
  ) {
    return {
      kind: 'archive',
      application: 'editor',
      label: 'Arquivo compactado · extração ainda não suportada',
      mime: mime || 'application/octet-stream',
      support: 'unsupported',
    };
  }
  if (extension === 'pdf' || mime === 'application/pdf') {
    return {
      kind: 'document',
      application: 'editor',
      label: 'PDF · visualização ainda não suportada',
      mime: 'application/pdf',
      support: 'unsupported',
    };
  }
  if (textExtensions.has(extension) || extension === 'zsh' || mime.startsWith('text/')) {
    return {
      kind: 'text',
      application: 'editor',
      label: 'HackPad · editor de texto',
      mime: mime || 'text/plain',
      support: 'editable',
    };
  }
  return {
    kind: 'unknown',
    application: 'editor',
    label: 'HackPad · arquivo desconhecido',
    mime: mime || 'application/octet-stream',
    support: 'editable', // Legacy VFS unknown files still contain UTF-8 text, not bytes.
  };
}

export function mediaSourceForPath(
  path: string,
  node?: Pick<VfsNode, 'metadata' | 'content'>,
): string | null {
  const declared = node?.metadata.mediaSource;
  if (declared) {
    // Media metadata may never turn a virtual file into an external request.
    return /^\/assets\/[a-zA-Z0-9_/-]+\.(?:png|jpe?g|webp|gif|ogg|mp3|wav|mp4|webm)$/.test(declared)
      ? declared
      : null;
  }
  if (extensionOf(path) === 'svg' && node?.content.trimStart().startsWith('<svg')) {
    return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(node.content)}`;
  }
  const name = (path.split('/').pop() ?? '').toLowerCase();
  if (name === 'cyber-war-opening.webm') {
    return '/assets/video/cyber-war-opening.webm';
  }
  if (name === 'cyber-war-opening.mp4') {
    return '/assets/video/cyber-war-opening.mp4';
  }
  if (name === 'cyber-war-opening.ogg' || name === 'cyber-war-opening.mp3') {
    return '/assets/audio/cyber-war-opening-music.ogg';
  }
  if (name === 'kali-waves.png') {
    return '/assets/kali-waves.png';
  }
  if (name === 'kali-maze.jpg') {
    return '/assets/kali-maze.jpg';
  }
  if (name === 'kali-cubes2.jpg') {
    return '/assets/kali-cubes2.jpg';
  }
  return null;
}

export function subtitlePathFor(path: string): string {
  const base = path.replace(/\.[^/.]+$/, '');
  return `${base}.vtt`;
}
