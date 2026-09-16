import { useEffect, useRef } from 'react';
import type { CSSProperties } from 'react';
import './session-intro.css';

// Campaign numbering follows WorldState.session, including the tutorial at zero.
const sessionNames: Record<number, string> = {
  0: 'Prólogo',
  1: 'Script Kiddie',
  2: 'Reputation',
  3: 'Black Hat',
  4: 'Ghost',
  5: 'Exposed',
  6: 'Hunted',
};

export function SessionIntro({
  session,
  reducedMotion,
  onFinish,
}: {
  session: number;
  reducedMotion: boolean;
  onFinish: () => void;
}) {
  const surface = useRef<HTMLDivElement>(null);
  const duration = reducedMotion ? 3200 : 5600;
  const title = sessionNames[session] ?? 'A história continua';

  useEffect(() => {
    const previous = document.activeElement;
    surface.current?.focus({ preventScroll: true });
    const timer = window.setTimeout(onFinish, duration);
    return () => {
      window.clearTimeout(timer);
      if (previous instanceof HTMLElement && previous.isConnected && !previous.closest('[inert]')) {
        previous.focus({ preventScroll: true });
      } else {
        document
          .querySelector<HTMLElement>('[data-testid="desktop"]')
          ?.focus({ preventScroll: true });
      }
    };
  }, [duration, onFinish]);

  return (
    <div
      ref={surface}
      className={`session-intro${reducedMotion ? ' session-intro-reduced' : ''}`}
      style={{ '--session-intro-duration': `${duration}ms` } as CSSProperties}
      role="dialog"
      aria-modal="true"
      aria-label={`Sessão ${session}: ${title}`}
      tabIndex={-1}
      onAnimationEnd={(event) => {
        if (event.target === event.currentTarget) {
          onFinish();
        }
      }}
      onKeyDown={(event) => {
        // Keep keyboard focus on the introduction while the desktop is inert.
        if (event.key === 'Tab') {
          event.preventDefault();
        }
        if (event.key !== 'F11') {
          event.stopPropagation();
        }
      }}
    >
      <div className="session-intro-band" aria-hidden="true">
        <p className="session-intro-number">SESSÃO {session}</p>
        <h1 className="session-intro-title">{title}</h1>
      </div>
    </div>
  );
}
