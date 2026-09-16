import { useEffect, useRef } from 'react';
import { desktopKeyboardShortcuts } from './desktop-keyboard';
import { fileKeyboardShortcuts } from '../../lib/file-keyboard';
import './desktop-keyboard.css';

export function DesktopKeyboardHelp({ onClose }: { onClose: () => void }) {
  const previousFocus = useRef(document.activeElement);
  useEffect(
    () => () => {
      const element = previousFocus.current;
      if (element instanceof HTMLElement && element.isConnected) {
        element.focus();
      }
    },
    [],
  );
  return (
    <div className="desktop-create-backdrop">
      <section
        className="desktop-create-dialog desktop-keyboard-help"
        role="dialog"
        aria-modal="true"
        aria-labelledby="desktop-keyboard-title"
        onKeyDown={(event) => {
          if (event.key === 'Escape' || event.key === 'F1') {
            event.preventDefault();
            onClose();
          }
          if (event.key === 'Tab') {
            event.preventDefault();
          }
          event.stopPropagation();
        }}
      >
        <header>
          <h2 id="desktop-keyboard-title">Atalhos de teclado</h2>
          <button autoFocus onClick={onClose} aria-label="Fechar atalhos de teclado">
            ×
          </button>
        </header>
        <div className="desktop-keyboard-list">
          <section aria-label="Arquivos e pastas">
            <h3>Arquivos e pastas · desktop e gerenciador de arquivos</h3>
            <dl>
              {fileKeyboardShortcuts.map((item) => (
                <div key={item.keys}>
                  <dt>{item.label}</dt>
                  <dd>
                    <kbd>{item.keys}</kbd>
                  </dd>
                </div>
              ))}
            </dl>
          </section>
          {['Aplicativos', 'Janelas', 'Sistema'].map((group) => (
            <section key={group} aria-label={group}>
              <h3>{group}</h3>
              <dl>
                {desktopKeyboardShortcuts
                  .filter((shortcut) => shortcut.group === group)
                  .map((shortcut) => (
                    <div key={shortcut.action}>
                      <dt>{shortcut.label}</dt>
                      <dd>
                        <kbd>{shortcut.keys}</kbd>
                      </dd>
                    </div>
                  ))}
                {group === 'Sistema' && (
                  <div>
                    <dt>Alternar tela cheia</dt>
                    <dd>
                      <kbd>F11</kbd>
                    </dd>
                  </div>
                )}
              </dl>
            </section>
          ))}
        </div>
        <p>
          Esc fecha esta lista. Os atalhos dos aplicativos continuam disponíveis dentro de cada
          janela.
        </p>
      </section>
    </div>
  );
}
