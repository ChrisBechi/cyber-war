export function GoggleIcon({ name }: { name: string }) {
  const paths: Record<string, string> = {
    search: 'M10.5 17a6.5 6.5 0 1 0 0-13 6.5 6.5 0 0 0 0 13Zm5-1 5 5',
    keyboard: 'M3 6h18v12H3zM6 9h1m3 0h1m3 0h1m3 0h1M6 12h1m3 0h1m3 0h1m3 0h1M7 15h10',
    mic: 'M9 6a3 3 0 0 1 6 0v6a3 3 0 0 1-6 0V6Zm-3 5v1a6 6 0 0 0 12 0v-1M12 18v3m-3 0h6',
    image: 'M3 9V5h4m10 0h4v4m0 6v4h-4M7 19H3v-4M16 12a4 4 0 1 0-8 0 4 4 0 0 0 8 0Z',
    close: 'm6 6 12 12M18 6 6 18',
    mail: 'M3 5h18v14H3zM3 6l9 7 9-7',
    drive: 'm12 3 9 16H3L12 3Zm-3 6 6 10M6 14h12',
    pin: 'M12 21s7-7 7-12A7 7 0 0 0 5 9c0 5 7 12 7 12Zm2-12a2 2 0 1 0-4 0 2 2 0 0 0 4 0Z',
    news: 'M4 4h16v16H4zM7 8h10M7 12h4m3 0h3M7 16h4m3 0h3',
    calendar: 'M4 5h16v16H4zM8 3v4m8-4v4M4 10h16m-12 4h1m6 0h1m-8 3h1',
    video: 'M3 6h12v12H3zM15 10l6-4v12l-6-4',
    person: 'M16 7a4 4 0 1 0-8 0 4 4 0 0 0 8 0ZM4 21v-2a8 8 0 0 1 16 0v2',
    arrow: 'M5 12h14m-6-6 6 6-6 6',
    backspace: 'm9 5-7 7 7 7h12V5H9Zm3 4 6 6m0-6-6 6',
  };
  return (
    <svg
      aria-hidden="true"
      viewBox="0 0 24 24"
      width="22"
      height="22"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d={paths[name] ?? paths.search} />
    </svg>
  );
}
