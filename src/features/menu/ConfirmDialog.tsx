import { useEffect, useRef } from 'react';

export function ConfirmDialog({
  title,
  description,
  confirmLabel,
  busy,
  error,
  onCancel,
  onConfirm,
}: {
  title: string;
  description: string;
  confirmLabel: string;
  busy: boolean;
  error?: string;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  const panel = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const previous = document.activeElement as HTMLElement | null;
    panel.current?.querySelector('button')?.focus();
    return () => previous?.focus();
  }, []);
  return (
    <div className="front-modal-backdrop">
      <div
        ref={panel}
        className="front-modal"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="confirm-title"
        aria-describedby="confirm-description"
        onKeyDown={(event) => {
          if (event.key === 'Escape' && !busy) {
            event.stopPropagation();
            onCancel();
          }
          if (event.key === 'Tab') {
            const buttons = Array.from(
              event.currentTarget.querySelectorAll<HTMLButtonElement>('button:not(:disabled)'),
            );
            event.preventDefault();
            buttons[
              (buttons.indexOf(document.activeElement as HTMLButtonElement) +
                (event.shiftKey ? -1 : 1) +
                buttons.length) %
                buttons.length
            ]?.focus();
          }
        }}
      >
        <span className="front-kicker">CONFIRMAÇÃO</span>
        <h2 id="confirm-title">{title}</h2>
        <p id="confirm-description">{description}</p>
        {error && <p role="alert">{error}</p>}
        <div className="front-modal-actions">
          <button disabled={busy} onClick={onCancel}>
            CANCELAR
          </button>
          <button className="front-primary" disabled={busy} onClick={onConfirm}>
            {busy ? 'AGUARDE…' : confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
