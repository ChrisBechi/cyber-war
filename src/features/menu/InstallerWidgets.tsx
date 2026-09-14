import { useEffect, useId, useRef, useState } from 'react';

export function InstallerList<T extends string>({
  label,
  options,
  value,
  onChange,
  onActivate,
}: {
  label: string;
  options: readonly (readonly [T, string])[];
  value: T;
  onChange: (value: T) => void;
  onActivate?: (value: T) => void;
}) {
  const id = useId();
  return (
    <>
      <span className="installer-label" id={id}>
        {label}
      </span>
      <div
        className="installer-list"
        role="listbox"
        aria-labelledby={id}
        tabIndex={0}
        onKeyDown={(event) => {
          const index = options.findIndex(([key]) => key === value);
          let next = index;
          if (event.key === 'ArrowDown') {
            next = Math.min(options.length - 1, index + 1);
          } else if (event.key === 'ArrowUp') {
            next = Math.max(0, index - 1);
          } else if (event.key === 'Home') {
            next = 0;
          } else if (event.key === 'End') {
            next = options.length - 1;
          } else if (event.key === 'Enter' && onActivate) {
            event.preventDefault();
            onActivate(value);
            return;
          } else {
            return;
          }
          event.preventDefault();
          onChange(options[next][0]);
          event.currentTarget.children[next]?.scrollIntoView?.({ block: 'nearest' });
        }}
      >
        {options.map(([key, text]) => (
          <div
            key={key}
            role="option"
            aria-selected={value === key}
            className={value === key ? 'is-selected' : ''}
            onClick={() => onChange(key)}
            onDoubleClick={() => onActivate?.(key)}
          >
            {text}
          </div>
        ))}
      </div>
    </>
  );
}

// All stages last four seconds: 3.8 s of filling, then 0.2 s visibly at 100%.
export function InstallerProgress({
  label,
  details,
  onComplete,
}: {
  label: string;
  details: readonly string[];
  onComplete: () => void;
}) {
  const [progress, setProgress] = useState(0);
  const complete = useRef(onComplete);
  complete.current = onComplete;
  useEffect(() => {
    const started = Date.now();
    const interval = window.setInterval(() => {
      setProgress(Math.min(100, (Date.now() - started) / 38));
    }, 16);
    const filled = window.setTimeout(() => setProgress(100), 3800);
    const finished = window.setTimeout(() => complete.current(), 4000);
    return () => {
      window.clearInterval(interval);
      window.clearTimeout(filled);
      window.clearTimeout(finished);
    };
  }, []);
  const detail =
    details[Math.min(details.length - 1, Math.floor((progress / 100) * details.length))];
  return (
    <div className="installer-progress">
      <div
        className="installer-progress-track"
        role="progressbar"
        aria-label={label}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={Math.floor(progress)}
      >
        <span className="installer-progress-label">{label}</span>
        <div
          className="installer-progress-fill"
          style={{ clipPath: `inset(0 ${100 - progress}% 0 0)` }}
        >
          <span className="installer-progress-label">{label}</span>
        </div>
      </div>
      <p>{detail}</p>
    </div>
  );
}
