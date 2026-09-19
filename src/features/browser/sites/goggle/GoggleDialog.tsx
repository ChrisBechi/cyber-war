import { useEffect, useRef } from 'react';
import type { ReactNode } from 'react';
import { GoggleIcon } from './GoggleIcon';

export function GoggleDialog({
  title,
  close,
  children,
}: {
  title: string;
  close: () => void;
  children: ReactNode;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const closeRef = useRef(close);
  closeRef.current = close;
  useEffect(() => {
    const previous = document.activeElement;
    ref.current?.focus();
    return () => {
      if (previous instanceof HTMLElement) {
        previous.focus();
      }
    };
  }, []);
  return (
    <div
      className="goggle-overlay"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) {
          close();
        }
      }}
    >
      <div
        ref={ref}
        role="dialog"
        aria-modal="true"
        aria-label={title}
        tabIndex={-1}
        className="goggle-dialog"
        onKeyDown={(event) => {
          if (event.key === 'Escape') {
            event.stopPropagation();
            closeRef.current();
          }
          if (event.key === 'Tab') {
            const items = Array.from(
              ref.current?.querySelectorAll<HTMLElement>(
                'button:not(:disabled), input, select, [tabindex="0"]',
              ) ?? [],
            );
            const first = items[0];
            const last = items[items.length - 1];
            if (
              event.shiftKey &&
              (document.activeElement === first || document.activeElement === ref.current)
            ) {
              event.preventDefault();
              last?.focus();
            } else if (!event.shiftKey && document.activeElement === last) {
              event.preventDefault();
              first?.focus();
            }
          }
        }}
      >
        <header>
          <h2>{title}</h2>
          <button type="button" className="goggle-icon-button" aria-label="Fechar" onClick={close}>
            <GoggleIcon name="close" />
          </button>
        </header>
        {children}
      </div>
    </div>
  );
}
