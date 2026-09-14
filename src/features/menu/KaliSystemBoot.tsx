import { useRef, useState } from 'react';
import type { KeyboardEvent } from 'react';

const entries = [
  'Kali GNU/Linux',
  'Advanced options for Kali GNU/Linux',
  'UEFI Firmware Settings',
] as const;

export function KaliSystemBoot({ onContinue }: { onContinue: () => void }) {
  const [selected, setSelected] = useState(0);
  const buttons = useRef<Array<HTMLButtonElement | null>>([]);
  const handleKey = (event: KeyboardEvent<HTMLElement>) => {
    if (!['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) {
      return;
    }
    event.preventDefault();
    const next =
      event.key === 'Home'
        ? 0
        : event.key === 'End'
          ? entries.length - 1
          : (selected + (event.key === 'ArrowDown' ? 1 : -1) + entries.length) % entries.length;
    setSelected(next);
    buttons.current[next]?.focus();
  };

  return (
    <section
      className="kali-system-boot"
      aria-label="Inicialização do Kali Linux"
      onKeyDown={handleKey}
      lang="en"
    >
      <span className="sr-only">Kali Linux</span>
      <div className="kali-system-options" role="menu" aria-label="Menu de inicialização">
        {entries.map((entry, index) => (
          <button
            key={entry}
            ref={(element) => {
              buttons.current[index] = element;
            }}
            type="button"
            role="menuitem"
            autoFocus={index === 0}
            tabIndex={selected === index ? 0 : -1}
            className={selected === index ? 'is-selected' : undefined}
            onFocus={() => setSelected(index)}
            onPointerMove={() => buttons.current[index]?.focus()}
            onClick={onContinue}
          >
            {entry}
          </button>
        ))}
      </div>
    </section>
  );
}
