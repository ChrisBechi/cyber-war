import { useId } from 'react';
import type { ReactNode } from 'react';
import { windowApp } from '../../lib/window-store';
import type { WindowId, BuiltinAppId } from '../../lib/window-store';
import { softwareById } from '../../lib/software-catalog';
import { KaliIcon } from './KaliIcon';

export type IconName = BuiltinAppId | 'folder' | 'file' | 'trash';

const palettes: Record<IconName, [string, string]> = {
  terminal: ['#243d4f', '#101b29'],
  files: ['#55bde8', '#2775b3'],
  folder: ['#55bde8', '#2775b3'],
  file: ['#eaf5ff', '#a8cce5'],
  editor: ['#5cbfce', '#287b99'],
  browser: ['#49bde2', '#3864c8'],
  'tor-browser': ['#9c7bd8', '#49326d'],
  messages: ['#60cbb4', '#258a88'],
  forum: ['#74a7ee', '#5261bd'],
  missions: ['#e8b75d', '#b87735'],
  codelab: ['#b28be8', '#7251b8'],
  settings: ['#8ea8bd', '#506b85'],
  saves: ['#73c1a2', '#397f77'],
  processes: ['#65b8d6', '#386ba7'],
  journey: ['#ecad77', '#b96868'],
  vigilia: ['#79e6c1', '#2b766c'],
  'media-player': ['#edc16d', '#a45d3a'],
  'image-viewer': ['#88c8a4', '#3b7e69'],
  trash: ['#dbe9f6', '#657d98'],
  'archive-viewer': ['#d3aa50', '#8a6935'],
  'package-installer': ['#dda85f', '#89552e'],
};

const glyphs: Record<IconName, ReactNode> = {
  'package-installer': <path d="m3 7 9-4 9 4v11l-9 4-9-4Zm0 0 9 4 9-4M12 11v11M7 5l10 4v5" />,
  'archive-viewer': <path d="M5 3h14v18H5ZM10 3v4h4v4h-4v4h4v4h-4" />,
  folder: <></>,
  file: <></>,
  terminal: <path d="m5 6 6 6-6 6m8 0h6" />,
  files: (
    <path d="M3 8V6a2 2 0 0 1 2-2h5l3 3h6a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8Zm0 1h18" />
  ),
  editor: (
    <>
      <path d="M14 3H5v18h14v-9M8 8h3m-3 5h3m-3 4h7" />
      <path d="m13 9 6-6 2 2-6 6-3 1Z" />
    </>
  ),
  browser: (
    <>
      <circle cx="12" cy="12" r="9" />
      <ellipse cx="12" cy="12" rx="4" ry="9" />
      <path d="M3 12h18M5 6.5h14M5 17.5h14" />
    </>
  ),
  'tor-browser': (
    <>
      <circle cx="12" cy="12" r="9" />
      <path d="M6 7c2-2 5-3 8-2m-6 5c2-2 6-2 9 0M7 15c2 2 6 3 10 1" />
      <circle cx="12" cy="12" r="2" />
    </>
  ),
  messages: (
    <>
      <path d="M5 3h14a2 2 0 0 1 2 2v11a2 2 0 0 1-2 2h-8l-5 3v-3H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2Z" />
      <path d="M7 8h10M7 13h7" />
    </>
  ),
  forum: (
    <>
      <path d="M14 3H5a2 2 0 0 0-2 2v9l4-3h7a2 2 0 0 0 2-2V5a2 2 0 0 0-2-2Zm-4 12v2a2 2 0 0 0 2 2h5l4 3V11a2 2 0 0 0-2-2M7 7h5" />
    </>
  ),
  missions: (
    <>
      <rect x="5" y="4" width="14" height="17" rx="2" />
      <path d="M9 3h6v4H9Zm-1 11 3 3 5-6" />
    </>
  ),
  codelab: (
    <>
      <path d="M8 3H6a2 2 0 0 0-2 2v4l-2 3 2 3v4a2 2 0 0 0 2 2h2m8-18h2a2 2 0 0 1 2 2v4l2 3-2 3v4a2 2 0 0 1-2 2h-2" />
      <path d="m14 7-4 10" stroke="#e4d6ff" />
    </>
  ),
  settings: (
    <>
      <path d="m9 3-1 3-3-1-2 4 2 3-2 3 2 4 3-1 1 3h6l1-3 3 1 2-4-2-3 2-3-2-4-3 1-1-3Z" />
      <circle cx="12" cy="12" r="4" />
    </>
  ),
  saves: (
    <>
      <path d="M4 3h13l4 4v14H3V3Zm3 0v7h10V3M7 21v-7h10v7" />
      <path d="M14 5v3" />
    </>
  ),
  processes: (
    <>
      <rect x="3" y="3" width="18" height="18" rx="2" />
      <path d="M7 17v-5m5 5V7m5 10v-8" />
    </>
  ),
  journey: (
    <>
      <path d="M6 18v-5a4 4 0 0 1 4-4h4a4 4 0 0 0 4-4" />
      <circle cx="6" cy="19" r="2" />
      <circle cx="18" cy="5" r="2" />
      <path d="m11 6 3 3-3 3" />
    </>
  ),
  vigilia: (
    <>
      <path d="M5 4h14l2 4-2 13H5L3 8l2-4Z" />
      <path d="M8 4v4m8-4v4M7 12h10M8 16h8" />
    </>
  ),
  'media-player': (
    <>
      <circle cx="12" cy="12" r="8" />
      <path d="m10 8 6 4-6 4V8Z" fill="currentColor" stroke="none" />
    </>
  ),
  'image-viewer': (
    <>
      <rect x="4" y="5" width="16" height="14" rx="2" />
      <circle cx="9" cy="10" r="1.5" />
      <path d="m5 17 4-4 3 3 2-2 4 3" />
    </>
  ),
  trash: (
    <>
      <path d="M6 8h12l-1 12H7L6 8Zm2-3h8l1 3H7l1-3Z" />
      <path d="M9 10v7m6-7v7" />
    </>
  ),
};

/** A fixed vector canvas keeps every symbol independent of font baselines. */
export function AppIcon({
  name: inputName,
  size = 18,
  trashFull = false,
}: {
  name: IconName | WindowId;
  size?: number;
  trashFull?: boolean;
}) {
  const name =
    inputName === 'folder' || inputName === 'file' || inputName === 'trash'
      ? inputName
      : windowApp(inputName);
  const official: Partial<Record<IconName, string>> = {
    terminal: 'utilities-terminal',
    files: 'system-file-manager',
    folder: 'folder',
    editor: 'accessories-text-editor',
    browser: 'firefox',
    trash: trashFull ? 'user-trash-full' : 'user-trash',
    settings: 'preferences-system',
  };
  const icon = name.startsWith('tool:')
    ? softwareById.get(name.slice(5))?.icon
    : official[name as IconName];
  return name === 'processes' ? (
    <svg className="app-icon" width={size} height={size} viewBox="0 0 16 16" aria-hidden="true">
      <rect x="1.5" y="2" width="13" height="10" rx="1" fill="#31383f" stroke="#aeb7be" />
      <path d="M3 10V8h3V6h3V4h4v6Z" fill="#75bb67" />
      <path d="M3 10h10M6 14h4M8 12v2" fill="none" stroke="#aeb7be" />
    </svg>
  ) : icon ? (
    <KaliIcon name={icon} size={size} />
  ) : (
    <LegacyAppIcon name={name as IconName} size={size} />
  );
}

function LegacyAppIcon({ name, size = 18 }: { name: IconName; size?: number }) {
  const gradient = useId();
  const [light, dark] = palettes[name];
  return (
    <svg
      className="app-icon"
      data-icon={name}
      width={size}
      height={size}
      viewBox="0 0 48 48"
      fill="none"
      aria-hidden="true"
      focusable="false"
    >
      <defs>
        <linearGradient id={gradient} x1="8" y1="4" x2="40" y2="44" gradientUnits="userSpaceOnUse">
          <stop stopColor={light} />
          <stop offset="1" stopColor={dark} />
        </linearGradient>
      </defs>
      {name === 'folder' ? (
        <>
          <path
            d="M4 12a4 4 0 0 1 4-4h11l5 5h16a4 4 0 0 1 4 4v19a4 4 0 0 1-4 4H8a4 4 0 0 1-4-4Z"
            fill={dark}
            stroke="#a5ddff"
            strokeOpacity=".5"
          />
          <path
            d="M4 20a4 4 0 0 1 4-4h32a4 4 0 0 1 4 4v16a4 4 0 0 1-4 4H8a4 4 0 0 1-4-4Z"
            fill={`url(#${gradient})`}
            stroke="#c6edff"
            strokeOpacity=".5"
          />
          <path d="M10 21h28" stroke="#b3e9ff" strokeOpacity=".65" strokeLinecap="round" />
        </>
      ) : name === 'file' ? (
        <g transform="translate(1 0)">
          <path
            d="M12 4h17l9 9v27a4 4 0 0 1-4 4H12a4 4 0 0 1-4-4V8a4 4 0 0 1 4-4Z"
            fill={`url(#${gradient})`}
            stroke="#ecf7ff"
            strokeOpacity=".8"
          />
          <path d="M29 4v9h9" fill="#86b8d9" stroke="#688fae" strokeLinejoin="round" />
          <path
            d="M16 21h14M16 27h14M16 33h9"
            stroke="#436d8a"
            strokeWidth="2"
            strokeLinecap="round"
          />
        </g>
      ) : (
        <>
          <rect
            x="3"
            y="3"
            width="42"
            height="42"
            rx="10"
            fill={`url(#${gradient})`}
            stroke={light}
          />
          <path d="M13 5h22" stroke="#fff" strokeOpacity=".2" strokeLinecap="round" />
          <g
            transform={`translate(${name === 'editor' ? 11 : 12} ${name === 'forum' ? 11.5 : 12})`}
            stroke="#f0f9ff"
            strokeWidth="1.7"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            {glyphs[name]}
          </g>
        </>
      )}
    </svg>
  );
}
