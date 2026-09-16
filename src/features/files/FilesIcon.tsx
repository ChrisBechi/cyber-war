// Original Flat-Remix-Blue-Dark assets. See public/assets/flat-remix/sources.json.
const symbols = {
  home: 'user-home-symbolic',
  desktop: 'user-desktop-symbolic',
  documents: 'folder-documents-symbolic',
  downloads: 'folder-download-symbolic',
  music: 'folder-music-symbolic',
  pictures: 'folder-pictures-symbolic',
  videos: 'folder-videos-symbolic',
  projects: 'folder-symbolic',
  system: 'computer-symbolic',
  trash: 'user-trash-symbolic',
  search: 'edit-find-symbolic',
  back: 'go-previous-symbolic',
  forward: 'go-next-symbolic',
  up: 'go-up-symbolic',
  refresh: 'view-refresh-symbolic',
  more: 'view-more-symbolic',
  list: 'view-list-symbolic',
  grid: 'view-grid-symbolic',
  down: 'pan-down-symbolic',
  close: 'window-close-symbolic',
} as const;

export type FilesSymbol = keyof typeof symbols;

export function FilesIcon({ name }: { name: FilesSymbol }) {
  return (
    <span
      className="files-symbol"
      style={{ maskImage: `url(/assets/flat-remix/${symbols[name]}.svg)` }}
      aria-hidden="true"
    />
  );
}

const folders: Record<string, string> = {
  Desktop: 'user-desktop',
  Documents: 'folder-blue-documents',
  Downloads: 'folder-blue-download',
  Music: 'folder-blue-music',
  Pictures: 'folder-blue-pictures',
  Videos: 'folder-blue-videos',
  Public: 'folder-blue-publicshare',
  Templates: 'folder-blue-templates',
};

export function FolderIcon({ name }: { name: string }) {
  return (
    <img
      className="files-folder-icon"
      src={`/assets/flat-remix/${folders[name] ?? 'folder-blue'}.svg`}
      width={64}
      height={64}
      alt=""
      draggable={false}
    />
  );
}
