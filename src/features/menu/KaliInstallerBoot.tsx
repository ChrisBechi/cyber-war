import { useRef, useState } from 'react';
import type { KeyboardEvent } from 'react';

const entries = [
  'Graphical install',
  'Install',
  'Advanced options ...',
  'Accessible dark contrast installer menu ...',
  'Install with speech synthesis',
] as const;

export function KaliInstallerBoot({
  onContinue,
  onCancel,
}: {
  onContinue: () => void;
  onCancel: () => void;
}) {
  const [selected, setSelected] = useState(0);
  const buttons = useRef<Array<HTMLButtonElement | null>>([]);
  const handleKey = (event: KeyboardEvent<HTMLElement>) => {
    if (event.key === 'Escape') {
      event.preventDefault();
      onCancel();
      return;
    }
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
      className="kali-installer-boot"
      aria-label="Inicialização do instalador Kali Linux"
      onKeyDown={handleKey}
      lang="en"
    >
      <span className="sr-only">Kali Linux</span>
      <p className="kali-installer-title">Kali Linux installer menu (UEFI mode)</p>
      <div className="kali-installer-options" role="menu" aria-label="Menu do instalador">
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
            onPointerMove={() => setSelected(index)}
            onClick={onContinue}
          >
            {entry}
          </button>
        ))}
      </div>
      <div className="kali-installer-shortcuts" aria-label="Atalhos de teclado">
        <span className="kali-installer-select">
          <span>Enter:</span>
          <span>Select</span>
        </span>
        <span className="kali-installer-edit">
          <span>E:</span>
          <span>Edit Selection</span>
        </span>
        <span className="kali-installer-command">
          <span>C:</span>
          <span>GRUB Command line</span>
        </span>
      </div>
    </section>
  );
}
