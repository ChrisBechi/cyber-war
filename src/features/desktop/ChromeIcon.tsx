import type { ReactNode } from 'react';

const shapes: Record<string, ReactNode> = {
  back: <path d="m9 3-5 5 5 5M4 8h10" />,
  forward: <path d="m7 3 5 5-5 5M2 8h10" />,
  reload: (
    <>
      <path d="M13 6a5 5 0 1 0 0 5M13 2v4H9" />
    </>
  ),
  home: <path d="m2 7 6-5 6 5M4 6v8h3v-4h2v4h3V6" />,
  search: (
    <>
      <circle cx="6.5" cy="6.5" r="4" />
      <path d="m10 10 4 4" />
    </>
  ),
  close: <path d="m4 4 8 8m0-8-8 8" />,
  plus: <path d="M8 3v10M3 8h10" />,
  minus: <path d="M3 8h10" />,
  menu: <path d="M2 4h12M2 8h12M2 12h12" />,
  down: <path d="m3 6 5 5 5-5" />,
  lock: (
    <>
      <rect x="4" y="7" width="8" height="7" rx="1" />
      <path d="M5 7V5a3 3 0 0 1 6 0v2" />
    </>
  ),
  shield: <path d="m8 1 5 2v5c0 3-3 5-5 7-2-2-5-4-5-7V3Z" />,
  star: <path d="m8 1 2 4.5 5 .5-3.7 3.3 1.2 5L8 12l-4.5 2.3 1.2-5L1 6l5-.5Z" />,
  settings: (
    <>
      <path d="m6 2-1 2-2 1 1 3-1 3 2 1 1 2h4l1-2 2-1-1-3 1-3-2-1-1-2Z" />
      <circle cx="8" cy="8" r="2" />
    </>
  ),
  target: (
    <>
      <circle cx="8" cy="8" r="4" />
      <circle cx="8" cy="8" r="1" />
      <path d="M8 1v3m0 8v3M1 8h3m8 0h3" />
    </>
  ),
  history: (
    <>
      <circle cx="8" cy="8" r="6" />
      <path d="M8 4v4l3 2" />
    </>
  ),
  downloads: <path d="M8 1v9m-4-4 4 4 4-4M2 11v3h12v-3" />,
  extensions: (
    <path d="M2 3h4V2a2 2 0 0 1 4 0v1h4v4h-1a2 2 0 0 0 0 4h1v3h-4v-1a2 2 0 0 0-4 0v1H2Z" />
  ),
};
export function ChromeIcon({ name }: { name: keyof typeof shapes }) {
  return (
    <svg
      className="chrome-icon"
      viewBox="0 0 16 16"
      width="14"
      height="14"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.25"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {shapes[name]}
    </svg>
  );
}
