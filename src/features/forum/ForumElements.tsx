import { Fragment, useId } from 'react';
import type { ForumMember } from './forum-model';

const icons: Record<string, string> = {
  home: 'm3 10 9-7 9 7M5 9v12h5v-7h4v7h5V9',
  announce: 'M4 9h5l11-5v16L9 15H4V9Zm4 6 2 6h4l-2-5M20 8h2v8h-2',
  members:
    'M16 21v-3a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v3M15 3a4 4 0 0 1 0 8M22 21v-3a4 4 0 0 0-3-4M13 7a4 4 0 1 1-8 0 4 4 0 0 1 8 0Z',
  globe: 'M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0ZM3 12h18M12 3c-5 5-5 13 0 18 5-5 5-13 0-18Z',
  chat: 'M4 4h16v13H9l-5 4V4ZM8 8h8M8 12h5',
  code: 'm8 5-6 7 6 7m8-14 6 7-6 7m-3-17-2 20',
  terminal: 'M3 4h18v16H3V4Zm3 4 4 4-4 4m7 0h5',
  network: 'M9 2h6v6H9V2ZM2 16h6v6H2v-6Zm14 0h6v6h-6v-6ZM12 8v4M5 16v-4h14v4',
  game: 'M7 6h10l3 3 2 10-3 2-5-5h-4l-5 5-3-2L4 9l3-3ZM5 11h6m-3-3v6m8-3h.1m3 3h.1',
  briefcase: 'M3 7h18v14H3V7ZM8 7V3h8v4M3 12l9 3 9-3M10 15h4',
  search: 'M16 10a6 6 0 1 1-12 0 6 6 0 0 1 12 0Zm-2 5 7 7',
  arrow: 'm9 5 7 7-7 7',
  back: 'm14 5-7 7 7 7M7 12h14',
  check: 'm4 12 5 5L20 6',
  plus: 'M12 4v16M4 12h16',
  pin: 'M8 3h8l-1 6 4 4H5l4-4-1-6Zm4 10v8',
  lock: 'M6 10h12v11H6V10Zm2 0V6a4 4 0 0 1 8 0v4',
  star: 'm12 2 3 6 7 1-5 5 1 8-6-4-6 4 1-8-5-5 7-1 3-6Z',
  quote: 'M4 5h6v8H6v5H3v-7l1-6Zm11 0h6v8h-4v5h-3v-7l1-6Z',
  flag: 'M5 22V3l6-1 5 3 5-1v11l-5 1-5-3-6 1',
  reply: 'm9 4-7 7 7 7v-5h4c4 0 6 2 8 6 0-7-2-10-8-10H9V4Z',
  document: 'M5 2h9l5 5v15H5V2Zm9 0v6h5M8 12h8M8 16h8',
};

export function ForumIcon({ name, size = 16 }: { name: string; size?: number }) {
  return (
    <svg
      className="fb-icon"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d={icons[name] ?? icons.chat} />
    </svg>
  );
}

export function ForumAvatar({ member, small = false }: { member: ForumMember; small?: boolean }) {
  const id = useId();
  return (
    <div
      className={`fb-avatar fb-color-${member.color} ${small ? 'fb-avatar-small' : ''}`}
      aria-hidden="true"
    >
      <svg viewBox="0 0 160 160" fill="none">
        <defs>
          <radialGradient id={id}>
            <stop stopColor="currentColor" stopOpacity=".22" />
            <stop offset="1" stopColor="currentColor" stopOpacity="0" />
          </radialGradient>
        </defs>
        <rect width="160" height="160" fill={`url(#${id})`} />
        <path
          d="M15 35V15h20m90 0h20v20M15 125v20h20m90 0h20v-20M80 8v16M8 80h16m112 0h16m-72 56v16"
          stroke="currentColor"
          opacity=".24"
        />
        {member.avatar === 'ghost' ? (
          <>
            <path
              d="m80 25 33 29 16 54-27 27H58l-27-27 16-54L80 25Z"
              fill="#11171c"
              stroke="currentColor"
              strokeWidth="2"
            />
            <path
              d="m80 38 27 28 5 28-32 27-32-27 5-28 27-28Z"
              stroke="currentColor"
              opacity=".5"
            />
            <path
              d="m49 80 21 8m20 0 21-8M74 115l6 11 6-11"
              stroke="currentColor"
              strokeWidth="4"
            />
          </>
        ) : member.avatar === 'cube' ? (
          <>
            <path
              d="m80 27 47 27v53l-47 27-47-27V54l47-27Z"
              fill="#17202a"
              stroke="currentColor"
              strokeWidth="2"
            />
            <path
              d="m33 54 47 27 47-27M80 81v53m0-84 27 16v31l-27 15-27-15V66l27-16Z"
              stroke="currentColor"
              strokeWidth="2"
            />
            <path d="m53 66 27 16 27-16M80 82v30" stroke="currentColor" opacity=".5" />
          </>
        ) : member.avatar === 'orbit' ? (
          <>
            <circle cx="80" cy="80" r="39" stroke="currentColor" strokeWidth="2" />
            <ellipse
              cx="80"
              cy="80"
              rx="64"
              ry="22"
              transform="rotate(-40 80 80)"
              stroke="currentColor"
              strokeWidth="2"
            />
            <circle cx="80" cy="80" r="13" fill="currentColor" opacity=".8" />
            <circle cx="127" cy="46" r="5" fill="currentColor" />
            <path d="M80 22v19m0 78v19M22 80h19m78 0h19" stroke="currentColor" opacity=".6" />
          </>
        ) : (
          <>
            <path
              d="m80 24 49 28v56l-49 28-49-28V52l49-28Z"
              stroke="currentColor"
              strokeWidth="2"
            />
            <path d="M49 91h13V60h14v45h13V46h13v45h12" stroke="currentColor" strokeWidth="5" />
            <path d="M31 58h14m70 44h14M66 31v12m28 74v12" stroke="currentColor" opacity=".5" />
          </>
        )}
      </svg>
    </div>
  );
}

function Inline({ text }: { text: string }) {
  return text
    .split(/(\*\*[^*]+\*\*)/g)
    .map((part, index) =>
      part.startsWith('**') && part.endsWith('**') ? (
        <strong key={index}>{part.slice(2, -2)}</strong>
      ) : (
        <Fragment key={index}>{part}</Fragment>
      ),
    );
}

export function ForumBody({ text }: { text: string }) {
  return (
    <div className="fb-message-body">
      {text
        .split(/(```[\s\S]*?```)/g)
        .filter(Boolean)
        .map((part, index) => {
          if (part.startsWith('```')) {
            const code = part.slice(3, -3).replace(/^[\w+-]*\n/, '');
            return (
              <pre key={index}>
                <code>{code.trimEnd()}</code>
              </pre>
            );
          }
          return (
            <Fragment key={index}>
              {part
                .split(/\n\s*\n/)
                .filter((paragraph) => paragraph.trim())
                .map((paragraph, i) => (
                  <p key={i}>
                    <Inline text={paragraph.trim()} />
                  </p>
                ))}
            </Fragment>
          );
        })}
    </div>
  );
}
